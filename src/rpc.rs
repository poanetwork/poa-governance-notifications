use std::convert::TryFrom;
use std::i64;
use std::thread;
use std::time::Duration;
use std::u64;

use chrono::{DateTime, Utc};
use ethabi::{Event, Token};
use ethereum_types::{Address, H256, U256};
use hex;
use jsonrpc_core as rpc;
use reqwest;
use serde_json as json;
use web3::types::{BlockNumber, Bytes, CallRequest, Filter, FilterBuilder, Log};

use config::{Config, ContractType, PoaContract, StartBlock};
use utils::{hex_string_to_u64, u256_to_datetime};

const JSONRPC_VERSION: rpc::Version = rpc::Version::V2;
const REQUEST_ID: rpc::Id = rpc::Id::Num(1);

#[derive(Clone, Copy, Debug)]
pub enum BallotType {
    InvalidKey,
    AddKey,
    RemoveKey,
    SwapKey,
    ChangeMinThreshold,
    ChangeProxyAddress
}

// Used when converting from an element in a Log's "topics" vector.
impl From<H256> for BallotType {
    fn from(topic: H256) -> Self {
        match topic.low_u64() {
            0 => BallotType::InvalidKey,
            1 => BallotType::AddKey,
            2 => BallotType::RemoveKey,
            3 => BallotType::SwapKey,
            4 => BallotType::ChangeMinThreshold,
            5 => BallotType::ChangeProxyAddress,
            _ => unreachable!()
        }
    }
}

// Used when converting from an output-token returned from a contract's
// `votingState` function.
impl From<U256> for BallotType {
    fn from(output: U256) -> Self {
        match output.as_u64() {
            0 => BallotType::InvalidKey,
            1 => BallotType::AddKey,
            2 => BallotType::RemoveKey,
            3 => BallotType::SwapKey,
            4 => BallotType::ChangeMinThreshold,
            5 => BallotType::ChangeProxyAddress,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum KeyType {
    Invalid,
    MiningKey,
    VotingKey,
    PayoutKey
}

impl From<U256> for KeyType {
    fn from(key_type: U256) -> Self {
        match key_type.as_u64() {
            0 => KeyType::Invalid,
            1 => KeyType::MiningKey,
            2 => KeyType::VotingKey,
            3 => KeyType::PayoutKey,
            _ => unreachable!()
        }
    }
}

#[derive(Clone, Copy, Debug)]
pub enum QuorumState {
    Invalid,
    InProgress,
    Accepted,
    Rejected
}

impl From<U256> for QuorumState {
    fn from(quorum_state: U256) -> Self {
        match quorum_state.as_u64() {
            0 => QuorumState::Invalid,
            1 => QuorumState::InProgress,
            2 => QuorumState::Accepted,
            3 => QuorumState::Rejected,
            _ => unreachable!()
        }
    }
}

#[derive(Debug)]
pub struct BallotCreatedLog {
    pub block_number: u64,
    pub ballot_id: u64,
    pub ballot_type: BallotType
}

impl From<Log> for BallotCreatedLog {
    fn from(log: Log) -> Self {
        BallotCreatedLog {
            block_number: log.block_number.unwrap().as_u64(),
            ballot_id: log.topics[1].low_u64(),
            ballot_type: log.topics[2].into()
        }
    }
}

#[derive(Debug)]
pub enum VotingData {
    Keys(KeysVotingData),
    Threshold(ThresholdVotingData),
    Proxy(ProxyVotingData)
}

impl VotingData {
    pub fn keys(tokens: Vec<Token>) -> Self {
        let voting_data: KeysVotingData = tokens.into();
        VotingData::Keys(voting_data)
    }

    pub fn threshold(tokens: Vec<Token>) -> Self {
        let voting_data: ThresholdVotingData = tokens.into();
        VotingData::Threshold(voting_data)
    }

    pub fn proxy(tokens: Vec<Token>) -> Self {
        let voting_data: ProxyVotingData = tokens.into();
        VotingData::Proxy(voting_data)
    }

    pub fn start_time(&self) -> DateTime<Utc> {
        match *self {
            VotingData::Keys(ref inner) => inner.start_time,
            VotingData::Threshold(ref inner) => inner.start_time,
            VotingData::Proxy(ref inner) => inner.start_time
        }
    }

    pub fn end_time(&self) -> DateTime<Utc> {
        match *self {
            VotingData::Keys(ref inner) => inner.end_time,
            VotingData::Threshold(ref inner) => inner.end_time,
            VotingData::Proxy(ref inner) => inner.end_time
        }
    }

    pub fn memo(&self) -> String {
        match *self {
            VotingData::Keys(ref inner) => inner.memo.clone(),
            VotingData::Threshold(ref inner) => inner.memo.clone(),
            VotingData::Proxy(ref inner) => inner.memo.clone()
        }
    }
}

#[derive(Debug)]
pub struct KeysVotingData {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub affected_key: Address,
    pub affected_key_type: KeyType,
    pub mining_key: Address,
    pub total_voters: u64,
    pub progress: i64,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub ballot_type: BallotType,
    pub index: u64,
    pub min_threshold_of_voters: u64,
    pub creator: Address,
    pub memo: String
}

impl From<Vec<Token>> for KeysVotingData {
    fn from(tokens: Vec<Token>) -> Self {
        let start_time = tokens[0].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let end_time = tokens[1].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let affected_key = tokens[2].clone().to_address().unwrap();
        let affected_key_type = tokens[3].clone().to_uint().unwrap().into();
        let mining_key = tokens[4].clone().to_address().unwrap();
        let total_voters = tokens[5].clone().to_uint().unwrap().as_u64();
        let progress = match tokens[6].clone().to_int().unwrap().low_u64() {
            lsb if lsb <= total_voters => lsb as i64,
            lsb => i64::try_from(u64::MAX - lsb + 1).unwrap()
        };
        let is_finalized = tokens[7].clone().to_bool().unwrap();
        let quorum_state = tokens[8].clone().to_uint().unwrap().into();
        let ballot_type = tokens[9].clone().to_uint().unwrap().into();
        let index = tokens[10].clone().to_uint().unwrap().as_u64();
        let min_threshold_of_voters = tokens[11].clone().to_uint().unwrap().as_u64();
        let creator = tokens[12].clone().to_address().unwrap();
        let memo = tokens[13].clone().to_string().unwrap();

        KeysVotingData {
            start_time, end_time,
            affected_key, affected_key_type,
            mining_key, total_voters,
            progress, is_finalized,
            quorum_state, ballot_type,
            index, min_threshold_of_voters,
            creator, memo
        }
    }
}

#[derive(Debug)]
pub struct ThresholdVotingData {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: u64,
    pub progress: i64,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub index: u64,
    pub proposed_value: u64,
    pub min_threshold_of_voters: u64,
    pub creator: Address,
    pub memo: String
}

impl From<Vec<Token>> for ThresholdVotingData {
    fn from(tokens: Vec<Token>) -> Self {
        let start_time = tokens[0].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let end_time = tokens[1].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let total_voters = tokens[2].clone().to_uint().unwrap().as_u64();
        let progress = match tokens[3].clone().to_int().unwrap().low_u64() {
            lsb if lsb <= total_voters => lsb as i64,
            lsb => i64::try_from(u64::MAX - lsb + 1).unwrap()
        };
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let quorum_state = tokens[5].clone().to_uint().unwrap().into();
        let index = tokens[6].clone().to_uint().unwrap().as_u64();
        let proposed_value = tokens[7].clone().to_uint().unwrap().as_u64();
        let min_threshold_of_voters = tokens[8].clone().to_uint().unwrap().as_u64();
        let creator = tokens[9].clone().to_address().unwrap();
        let memo = tokens[10].clone().to_string().unwrap();

        ThresholdVotingData {
            start_time, end_time,
            total_voters, progress,
            is_finalized, quorum_state,
            index, proposed_value,
            min_threshold_of_voters, creator,
            memo
        }
    }
}

#[derive(Debug)]
pub struct ProxyVotingData {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: u64,
    pub progress: i64,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub index: u64,
    pub min_threshold_of_voters: u64,
    pub proposed_value: Address,
    pub contract_type: u64,
    pub creator: Address,
    pub memo: String
}

impl From<Vec<Token>> for ProxyVotingData {
    fn from(tokens: Vec<Token>) -> Self {
        let start_time = tokens[0].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let end_time = tokens[1].clone().to_uint()
            .map(|uint| u256_to_datetime(uint))
            .unwrap();

        let total_voters = tokens[2].clone().to_uint().unwrap().as_u64();
        let progress = match tokens[3].clone().to_int().unwrap().low_u64() {
            lsb if lsb <= total_voters => lsb as i64,
            lsb => i64::try_from(u64::MAX - lsb + 1).unwrap()
        };
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let quorum_state = tokens[5].clone().to_uint().unwrap().into();
        let index = tokens[6].clone().to_uint().unwrap().as_u64();
        let min_threshold_of_voters = tokens[7].clone().to_uint().unwrap().as_u64();
        let proposed_value = tokens[8].clone().to_address().unwrap();
        let contract_type = tokens[9].clone().to_uint().unwrap().as_u64();
        let creator = tokens[10].clone().to_address().unwrap();
        let memo = tokens[11].clone().to_string().unwrap();

        ProxyVotingData {
            start_time, end_time,
            total_voters, progress,
            is_finalized, quorum_state,
            index, min_threshold_of_voters,
            proposed_value, contract_type,
            creator, memo
        }
    }
}

pub struct BlockchainIter<'a> {
    client: &'a RpcClient,
    start_block: u64,
    stop_block: u64,
    on_first_iteration: bool,
    avg_block_time: Duration
}

impl<'a> BlockchainIter<'a> {
    pub fn new(client: &'a RpcClient, config: &Config) -> Self {
        let latest = client.latest_block_number().unwrap();

        let start_block = match config.start_block {
            StartBlock::Earliest => 0,
            StartBlock::Latest => latest,
            StartBlock::Number(block_number) => block_number,
            StartBlock::Tail(tail) => latest - tail
        };

        if start_block > latest {
            panic!(
                "Provided start-block ({}) exceeds latest mined block number ({})",
                start_block,
                latest
            );
        }

        BlockchainIter {
            client,
            start_block,
            stop_block: latest,
            on_first_iteration: true,
            avg_block_time: config.avg_block_time
        }
    }
}

impl<'a> Iterator for BlockchainIter<'a> {
    type Item = (BlockNumber, BlockNumber);

    fn next(&mut self) -> Option<Self::Item> {
        if self.on_first_iteration {
            self.on_first_iteration = false;
        } else {
            self.start_block = self.stop_block + 1;
            while self.start_block >= self.stop_block {
                thread::sleep(self.avg_block_time);
                self.stop_block = self.client.latest_block_number().unwrap();
            }
        }

        Some((
            BlockNumber::Number(self.start_block),
            BlockNumber::Number(self.stop_block)
        ))
    }
}

#[derive(Debug)]
pub enum Method {
    CallContractFunction,
    GetLogs,
    LastMinedBlockNumber
}

impl From<Method> for String {
    fn from(method: Method) -> Self {
        let s = match method {
            Method::CallContractFunction => "eth_call",
            Method::GetLogs => "eth_getLogs",
            Method::LastMinedBlockNumber => "eth_blockNumber"
        };
        s.into()
    }
}

pub struct EventFilter;

impl EventFilter {
    pub fn new(ev: Event, addr: Address, from: BlockNumber, to: BlockNumber) -> Filter {
        let topic = vec![ev.signature()];
        FilterBuilder::default()
            .topics(Some(topic), None, None, None)
            .address(vec![addr])
            .from_block(from)
            .to_block(to)
            .build()
    }
}

#[derive(Debug)]
pub struct RpcClient {
    endpoint: String,
    client: reqwest::Client
}

impl RpcClient {
    pub fn new(endpoint: &str) -> Self {
        let endpoint = endpoint.into();
        let client = reqwest::Client::new();
        RpcClient { endpoint, client }
    }

    fn build_request(&self, method: Method, params: Vec<json::Value>) -> reqwest::Request {
        let jsonrpc = Some(JSONRPC_VERSION);
        let id = REQUEST_ID.clone();
        let method = method.into();
        let params = Some(rpc::Params::Array(params));
        let method_call = rpc::MethodCall { jsonrpc, method, params, id };
        let body = rpc::Call::MethodCall(method_call);
        self.client.post(&self.endpoint).json(&body).build().unwrap()
    }

    fn send(&self, req: reqwest::Request) -> reqwest::Result<json::Value> {
        let resp: rpc::Response = self.client.execute(req)?.json().unwrap();
        if let rpc::Response::Single(resp_status) = resp {
            if let rpc::Output::Success(resp) = resp_status {
                return Ok(resp.result);
            }
        }
        unreachable!();
    }

    fn latest_block_number(&self) -> reqwest::Result<u64> {
        let req = self.build_request(Method::LastMinedBlockNumber, vec![]);
        if let json::Value::String(hex) = self.send(req)? {
            return Ok(hex_string_to_u64(&hex).unwrap());
        }
        unreachable!();
    }

    fn get_logs(&self, filter: Filter) -> reqwest::Result<Vec<Log>> {
        let params = vec![json::to_value(filter).unwrap()];
        let req = self.build_request(Method::GetLogs, params);
        let result = self.send(req)?;
        Ok(json::from_value(result).unwrap())
    }

    pub fn get_ballot_created_logs(
        &self,
        contract: &PoaContract,
        start: BlockNumber,
        stop: BlockNumber
    ) -> reqwest::Result<Vec<BallotCreatedLog>>
    {
        let event = contract.event("BallotCreated");
        let filter = EventFilter::new(event, contract.addr, start, stop);
        let logs = self.get_logs(filter)?;
        let ballot_created_logs: Vec<BallotCreatedLog> = logs.into_iter()
            .map(|log| log.into())
            .collect();
        Ok(ballot_created_logs)
    }

    pub fn get_voting_state(&self, contract: &PoaContract, ballot_id: u64) -> reqwest::Result<VotingData> {
        let function = contract.function("votingState");
        let tokens = vec![Token::Uint(U256::from(ballot_id))];
        let encoded_input: Bytes = function.encode_input(&tokens).unwrap().into();

        let call = CallRequest {
            to: contract.addr,
            data: Some(encoded_input),
            from: None,
            gas: None,
            gas_price: None,
            value: None
        };

        let params = vec![
            json::to_value(call).unwrap(),
            json::to_value(BlockNumber::Latest).unwrap()
        ];

        let req = self.build_request(Method::CallContractFunction, params);
        let result = self.send(req)?;

        if let json::Value::String(hex) = result {
            let bytes = hex::decode(hex.trim_left_matches("0x")).unwrap();
            let outputs = function.decode_output(&bytes).unwrap();
            let voting_data = match contract.kind {
                ContractType::Keys => VotingData::keys(outputs.into()),
                ContractType::Threshold => VotingData::threshold(outputs.into()),
                ContractType::Proxy => VotingData::proxy(outputs.into())
            };
            return Ok(voting_data);
        }

        unreachable!();
    }
}
