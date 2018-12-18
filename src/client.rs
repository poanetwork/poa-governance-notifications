use std::u64;

use jsonrpc_core as json_rpc;
use serde_json as json;
use web3::types::{Address, BlockNumber, Filter, FilterBuilder, U256};

use crate::config::{ContractType, PoaContract};
use crate::error::{Error, Result};
use crate::response::common::BallotCreatedLog;
use crate::response::v1::{KeysVotingState, ProxyVotingState, ThresholdVotingState, VotingState};
use crate::response::v2::{
    BallotInfo, EmissionBallotInfo, KeysBallotInfo, ProxyBallotInfo, ThresholdBallotInfo,
};

#[derive(Debug)]
pub enum RpcMethod {
    CallContractFunction,
    GetLogs,
    GetLastMinedBlockNumber,
}

impl Into<String> for RpcMethod {
    fn into(self) -> String {
        let s = match self {
            RpcMethod::CallContractFunction => "eth_call",
            RpcMethod::GetLogs => "eth_getLogs",
            RpcMethod::GetLastMinedBlockNumber => "eth_blockNumber",
        };
        s.into()
    }
}

#[derive(Debug)]
pub struct RpcClient {
    endpoint: String,
    client: reqwest::Client,
}

impl RpcClient {
    pub fn new(endpoint: String) -> Self {
        let client = reqwest::Client::new();
        RpcClient { endpoint, client }
    }

    fn build_request(
        &self,
        method: RpcMethod,
        params: Vec<json::Value>,
    ) -> Result<reqwest::Request> {
        let method_call = json_rpc::types::request::MethodCall {
            jsonrpc: Some(json_rpc::types::version::Version::V2),
            method: method.into(),
            params: Some(json_rpc::types::Params::Array(params)),
            id: json_rpc::types::id::Id::Num(1),
        };
        let request_data: json_rpc::types::request::Call = method_call.into();
        self.client
            .post(&self.endpoint)
            .json(&request_data)
            .build()
            .map_err(|e| Error::FailedToBuildRequest(e))
    }

    fn send(&self, req: reqwest::Request) -> Result<json::Value> {
        let resp: json_rpc::types::response::Response = self
            .client
            .execute(req)
            .map_err(|e| Error::RequestFailed(e))?
            .json()
            .unwrap();
        if let json_rpc::types::response::Response::Single(resp_status) = resp {
            match resp_status {
                json_rpc::types::response::Output::Success(resp) => return Ok(resp.result),
                json_rpc::types::response::Output::Failure(e) => {
                    return Err(Error::JsonRpcResponseFailure(e))
                }
            };
        }
        unreachable!("Recieved multiple responses for single request");
    }

    pub fn get_last_mined_block_number(&self) -> Result<u64> {
        let req = self.build_request(RpcMethod::GetLastMinedBlockNumber, vec![])?;
        if let json::Value::String(s) = self.send(req)? {
            let s = s.trim_left_matches("0x");
            let block_number = u64::from_str_radix(s, 16).unwrap();
            return Ok(block_number);
        }
        unreachable!("Received a non-string response from `eth_blockNumber` call");
    }

    fn get_logs(&self, filter: Filter) -> Result<Vec<web3::types::Log>> {
        let params = vec![json::to_value(filter).unwrap()];
        let req = self.build_request(RpcMethod::GetLogs, params)?;
        let result = self.send(req)?;
        Ok(json::from_value(result).unwrap())
    }

    /// V1 and V2
    pub fn get_ballot_created_logs(
        &self,
        contract: &PoaContract,
        start: BlockNumber,
        stop: BlockNumber,
    ) -> Result<Vec<BallotCreatedLog>> {
        let event = contract.event("BallotCreated");
        let event_sig = event.signature();
        let filter = FilterBuilder::default()
            .topics(Some(vec![event_sig]), None, None, None)
            .address(vec![contract.addr])
            .from_block(start)
            .to_block(stop)
            .build();
        self.get_logs(filter)?
            .into_iter()
            .map(|web3_log| {
                let web3::types::Log {
                    topics,
                    data,
                    block_number,
                    ..
                } = web3_log;
                let raw_log = ethabi::RawLog::from((topics, data.0));
                let ethabi_log = event
                    .parse_log(raw_log)
                    .map_err(|e| Error::FailedToParseRawLogToLog(e))?;
                BallotCreatedLog::from_ethabi_log(ethabi_log, block_number.unwrap())
            })
            .collect()
    }

    /// V1
    pub fn get_voting_state(&self, contract: &PoaContract, ballot_id: U256) -> Result<VotingState> {
        let function = contract.function("votingState");
        let tokens = vec![ethabi::Token::Uint(ballot_id)];
        let encoded_input = function.encode_input(&tokens).unwrap();
        let function_call_request = web3::types::CallRequest {
            to: contract.addr,
            data: Some(encoded_input.into()),
            from: None,
            gas: None,
            gas_price: None,
            value: None,
        };
        let rpc_method_params = vec![
            json::to_value(function_call_request).unwrap(),
            json::to_value(BlockNumber::Latest).unwrap(),
        ];
        let req = self.build_request(RpcMethod::CallContractFunction, rpc_method_params)?;
        if let json::Value::String(s) = self.send(req)? {
            let s = s.trim_left_matches("0x");
            let bytes = hex::decode(s).unwrap();
            let outputs = function.decode_output(&bytes).unwrap();
            let voting_state: VotingState = match contract.kind {
                ContractType::Keys => KeysVotingState::from(outputs).into(),
                ContractType::Threshold => ThresholdVotingState::from(outputs).into(),
                ContractType::Proxy => ProxyVotingState::from(outputs).into(),
                ContractType::Emission => return Err(Error::EmissionFundsV1ContractDoesNotExist),
            };
            return Ok(voting_state);
        }
        unreachable!("received non-string JSON response from `votingState`");
    }

    /// V2
    // TODO: When V2 contracts have been published and ballots have begun, test that calling
    // `.getBallotInfo()` with `Address::zero()` for the `votingKey` works (we don't care if
    // `votingKey` has voted yet).
    pub fn get_ballot_info(&self, contract: &PoaContract, ballot_id: U256) -> Result<BallotInfo> {
        // pub fn get_ballot_info(&self, contract: &PoaContract, ballot_id: U256, voting_key: Option<Address>) -> Result<BallotInfo> {
        let function = contract.function("getBallotInfo");
        /*
        let mut tokens = vec![ethabi::Token::Uint(ballot_id)];
        if function.inputs.len() == 2 {
            if let Some(voting_key) = voting_key {
                tokens.push(ethabi::Token::Address(voting_key));
            }
        }
        */
        let mut tokens = vec![ethabi::Token::Uint(ballot_id)];
        if function.inputs.len() == 2 {
            tokens.push(ethabi::Token::Address(Address::zero()));
        }

        let encoded_input = function.encode_input(&tokens).unwrap();
        let function_call_request = web3::types::CallRequest {
            to: contract.addr,
            data: Some(encoded_input.into()),
            from: None,
            gas: None,
            gas_price: None,
            value: None,
        };
        let rpc_method_params = vec![
            json::to_value(function_call_request).unwrap(),
            json::to_value(BlockNumber::Latest).unwrap(),
        ];
        let req = self.build_request(RpcMethod::CallContractFunction, rpc_method_params)?;
        if let json::Value::String(s) = self.send(req)? {
            let s = s.trim_left_matches("0x");
            let bytes = hex::decode(s).unwrap();
            let outputs = function.decode_output(&bytes).unwrap();
            let ballot_info: BallotInfo = match contract.kind {
                ContractType::Keys => KeysBallotInfo::from(outputs).into(),
                ContractType::Threshold => ThresholdBallotInfo::from(outputs).into(),
                ContractType::Proxy => ProxyBallotInfo::from(outputs).into(),
                ContractType::Emission => EmissionBallotInfo::from(outputs).into(),
            };
            return Ok(ballot_info);
        }
        unreachable!("received non-string JSON response from `getBallotInfo`");
    }
}

#[cfg(test)]
mod tests {
    use std::env;
    use std::str::FromStr;

    use web3::types::{Address, BlockNumber, U256};

    use super::RpcClient;
    use crate::config::{ContractType, ContractVersion, Network, PoaContract};
    use crate::response::v1::VotingState;
    use crate::response::v2::BallotInfo;
    use crate::tests::{
        setup, SOKOL_NETWORK, V1_CONTRACT_TYPES, V1_VERSION, V2_CONTRACT_TYPES, V2_VERSION,
        XDAI_NETWORK,
    };

    #[test]
    fn test_get_last_mined_block_core() {
        setup();

        let rpc_url = env::var("CORE_RPC_ENDPOINT")
            .expect("Missing env-var: `CORE_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let res = client.get_last_mined_block_number();
        assert!(res.is_ok());

        // As of writing this test, the last mined block number on the Core POA chain was 6245220.
        let last_mined_block_number = res.unwrap();
        assert!(last_mined_block_number >= 6245220);
    }

    #[test]
    fn test_get_last_mined_block_sokol() {
        setup();

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let res = client.get_last_mined_block_number();
        assert!(res.is_ok());

        // As of writing this test, the last mined block number on the Sokol POA chain was 6107511.
        let last_mined_block_number = res.unwrap();
        assert!(last_mined_block_number >= 6107511);
    }

    #[test]
    fn test_get_last_mined_block_xdai() {
        setup();

        let rpc_url = env::var("XDAI_RPC_ENDPOINT")
            .expect("Missing env-var: `XDAI_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let res = client.get_last_mined_block_number();
        assert!(res.is_ok());

        // As of writing this test, the last mined block number on the xDai chain was 1199366.
        let last_mined_block_number = res.unwrap();
        assert!(last_mined_block_number >= 1199366);
    }

    #[test]
    fn test_get_ballot_created_logs_for_keys_v1_contract() {
        setup();

        let contract = PoaContract::read(ContractType::Keys, SOKOL_NETWORK, V1_VERSION)
            .unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);

        let res = client.get_ballot_created_logs(
            &contract,
            BlockNumber::Earliest,
            BlockNumber::Latest,
        );
        assert!(res.is_ok());

        // As of writing this test, there were 69 VotingToChangeKeys V1 ballots on the Sokol chain.
        let ballot_created_logs = res.unwrap();
        assert!(ballot_created_logs.len() >= 69);
    }

    #[test]
    fn test_get_ballot_created_logs_for_keys_v2_contract() {
        setup();

        let contract = PoaContract::read(ContractType::Keys, SOKOL_NETWORK, V2_VERSION)
            .unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);

        let res = client.get_ballot_created_logs(
            &contract,
            BlockNumber::Earliest,
            BlockNumber::Latest,
        );
        assert!(res.is_ok());

        // As of writing this test, there were 2 VotingToChangeKeys V2 ballots on the Sokol chain.
        let ballot_created_logs = res.unwrap();
        assert!(ballot_created_logs.len() >= 2);
    }

    #[test]
    fn test_get_voting_state_for_threshold_v1() {
        setup();

        let contract = PoaContract::read(ContractType::Threshold, SOKOL_NETWORK, V1_VERSION)
                .unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
                .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let ballot_id = U256::from(0);
        let res = client.get_voting_state(&contract, ballot_id);
        assert!(res.is_ok());

        if let VotingState::Threshold(threshold_voting_state) = res.unwrap() {
            let expected_creator = Address::from_str("82e4e61e7f5139ff0a4157a5bc687ef42294c248").unwrap();
            let expected_proposed_value = U256::from(4);
            assert_eq!(threshold_voting_state.creator, expected_creator);
            assert_eq!(threshold_voting_state.proposed_value, expected_proposed_value);
        }
    }

    #[test]
    fn test_get_ballot_info_for_emission_v2() {
        setup();

        let contract = PoaContract::read(ContractType::Emission, Network::Sokol, ContractVersion::V2)
            .unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let ballot_id = U256::from(0);
        let res = client.get_ballot_info(&contract, ballot_id);
        assert!(res.is_ok());

        if let BallotInfo::Emission(emission_ballot_info) = res.unwrap() {
            let expected_creator = Address::from_str("82e4e61e7f5139ff0a4157a5bc687ef42294c248").unwrap();
            let expected_receiver = Address::from_str("9a1f6adb5bd804b5d8bd21dd7aeb44edecbaa313").unwrap();
            let expected_amount = U256::from_dec_str("10441000000000000000000").unwrap();
            assert_eq!(emission_ballot_info.creator, expected_creator);
            assert_eq!(emission_ballot_info.receiver, expected_receiver);
            assert_eq!(emission_ballot_info.amount, expected_amount);
        }
    }

    #[test]
    fn test_get_voting_state_for_all_v1_contracts() {
        setup();

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let ballot_id = U256::from(0);

        for contract_type in V1_CONTRACT_TYPES.iter() {
            let contract = match PoaContract::read(*contract_type, SOKOL_NETWORK, V1_VERSION) {
                Ok(contract) => contract,
                Err(e) => panic!("Failed to load contract: {:?}", e),
            };
            let res = client.get_voting_state(&contract, ballot_id);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_get_ballot_info_for_all_v2_contracts() {
        setup();

        let rpc_url = env::var("SOKOL_RPC_ENDPOINT")
            .expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let ballot_id = U256::from(0);

        for contract_type in V2_CONTRACT_TYPES.iter() {
            let contract = match PoaContract::read(*contract_type, SOKOL_NETWORK, V2_VERSION) {
                Ok(contract) => contract,
                Err(e) => panic!("Failed to load contract: {:?}", e),
            };
            let res = client.get_ballot_info(&contract, ballot_id);
            assert!(res.is_ok());
        }
    }

    #[test]
    fn test_get_ballot_info_for_v2_contracts_on_xdai_network() {
        setup();

        let rpc_url = env::var("XDAI_RPC_ENDPOINT")
            .expect("Missing env-var: `XDAI_RPC_ENDPOINT`");

        let client = RpcClient::new(rpc_url);
        let ballot_id = U256::from(0);

        for contract_type in V2_CONTRACT_TYPES.iter() {
            let contract = match PoaContract::read(*contract_type, XDAI_NETWORK, V2_VERSION) {
                Ok(contract) => contract,
                Err(e) => panic!("Failed to load contract: {:?}", e),
            };
            let res = client.get_ballot_info(&contract, ballot_id);
            assert!(res.is_ok());
        }
    }
}
