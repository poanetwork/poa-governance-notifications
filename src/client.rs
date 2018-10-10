use std::u64;

use ethabi;
use hex;
use jsonrpc_core as json_rpc;
use reqwest;
use serde_json as json;
use web3;
use web3::types::{Address, BlockNumber, Filter, FilterBuilder, U256};

use config::{ContractType, PoaContract};
use error::{Error, Result};
use response::{v1, v2};
use response::common::BallotCreatedLog;

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
    ) -> Result<reqwest::Request>
    {
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
        let resp: json_rpc::types::response::Response = self.client
            .execute(req)
            .map_err(|e| Error::RequestFailed(e))?
            .json()
            .unwrap();
        if let json_rpc::types::response::Response::Single(resp_status) = resp {
            match resp_status {
                json_rpc::types::response::Output::Success(resp) => return Ok(resp.result),
                json_rpc::types::response::Output::Failure(e) => return Err(Error::JsonRpcResponseFailure(e)),
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
    ) -> Result<Vec<BallotCreatedLog>>
    {
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
                let web3::types::Log {topics, data, block_number, .. } = web3_log;
                let raw_log = ethabi::RawLog::from((topics, data.0));
                let ethabi_log = event.parse_log(raw_log)
                    .map_err(|e| Error::FailedToParseRawLogToLog(e))?;
                BallotCreatedLog::from_ethabi_log(ethabi_log, block_number.unwrap())
            })
            .collect()
    }

    /// V1
    pub fn get_voting_state(&self, contract: &PoaContract, ballot_id: U256) -> Result<v1::VotingState> {
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
            let voting_state: v1::VotingState = match contract.kind {
                ContractType::Keys => v1::KeysVotingState::from(outputs).into(),
                ContractType::Threshold => v1::ThresholdVotingState::from(outputs).into(),
                ContractType::Proxy => v1::ProxyVotingState::from(outputs).into(),
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
    pub fn get_ballot_info(&self, contract: &PoaContract, ballot_id: U256) -> Result<v2::BallotInfo> {
    // pub fn get_ballot_info(&self, contract: &PoaContract, ballot_id: U256, voting_key: Option<Address>) -> Result<v2::BallotInfo> {
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
            let ballot_info: v2::BallotInfo = match contract.kind {
                ContractType::Keys => v2::KeysBallotInfo::from(outputs).into(),
                ContractType::Threshold => v2::ThresholdBallotInfo::from(outputs).into(),
                ContractType::Proxy => v2::ProxyBallotInfo::from(outputs).into(),
                ContractType::Emission => v2::EmissionBallotInfo::from(outputs).into(),
            };
            return Ok(ballot_info);
        }
        unreachable!("received non-string JSON response from `getBallotInfo`");
    }
}

#[cfg(test)]
mod tests {
    use std::env;

    use web3::types::BlockNumber;

    use super::super::tests::setup;
    use super::RpcClient;
    use config::{ContractType, ContractVersion, Network, PoaContract};

    #[test]
    fn test_get_last_mined_block() {
        setup();

        let sokol_url = env::var("SOKOL_RPC_ENDPOINT").expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");
        let client = RpcClient::new(sokol_url);
        let res = client.get_last_mined_block_number();
        println!("\nsokol last mined block number => {:?}", res);
        assert!(res.is_ok());

        let core_url = env::var("CORE_RPC_ENDPOINT").expect("Missing env-var: `CORE_RPC_ENDPOINT`");
        let client = RpcClient::new(core_url);
        let res = client.get_last_mined_block_number();
        println!("core last mined block number => {:?}", res);
        assert!(res.is_ok());
    }

    #[test]
    fn test_get_ballot_created_logs() {
        setup();
        let contract = PoaContract::read(
            ContractType::Keys,
            Network::Sokol,
            ContractVersion::V1,
        ).unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));
        let endpoint = env::var("SOKOL_RPC_ENDPOINT").expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");
        let client = RpcClient::new(endpoint);
        let res = client.get_ballot_created_logs(
            &contract,
            BlockNumber::Earliest,
            BlockNumber::Latest,
        );
        println!("{:#?}", res);
        assert!(res.is_ok());
    }

    // TODO: uncomment this test once V2 ballots are created.
    /*
    #[test]
    fn test_get_ballot_created_logs_v2() {
        setup();
        let contract = PoaContract::read(
            ContractType::Keys,
            Network::Core,
            ContractVersion::V2
        ).unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));
        let rpc_url = env::var("CORE_RPC_ENDPOINT").expect("Missing env-var: `CORE_RPC_ENDPOINT`");
        let client = RpcClient::new(rpc_url);
        let res = client.get_ballot_created_logs(
            &contract,
            BlockNumber::Earliest,
            BlockNumber::Latest,
        );
        println!("RES => {:#?}", res);
        assert!(res.is_ok());
    }
    */

    #[test]
    fn test_get_voting_state() {
        setup();
        let contract = PoaContract::read(
            ContractType::Threshold,
            Network::Sokol,
            ContractVersion::V1
        ).unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));
        let sokol_url = env::var("SOKOL_RPC_ENDPOINT").expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");
        let client = RpcClient::new(sokol_url);
        let res = client.get_voting_state(&contract, 2.into());
        println!("{:#?}", res);
        assert!(res.is_ok());
    }

    // TODO: uncomment this test once V2 ballots are created.
    /*
    #[test]
    fn test_get_ballot_info() {
        setup();
        let contract = PoaContract::read(
            ContractType::Emission,
            Network::Core,
            ContractVersion::V2
        ).unwrap_or_else(|e| panic!("Failed to load contract: {:?}", e));
        let sokol_url = env::var("CORE_RPC_ENDPOINT").expect("Missing env-var: `SOKOL_RPC_ENDPOINT`");
        let client = RpcClient::new(sokol_url);
        let ballot_id
        let res = client.get_ballot_info(&contract, 2.into());
        println!("{:#?}", res);
        assert!(res.is_ok());
    }
    */
}
