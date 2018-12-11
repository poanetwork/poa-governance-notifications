use chrono::{DateTime, Utc};
use ethabi;
use web3::types::{Address, U256};

use response::common::{u256_to_datetime, BallotType, KeyType};

/// Describes the current state of a given ballot.
///
/// The same `QuorumStates` enum is used by all V1 contracts.
///
/// V1 Keys Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeKeys.sol#L12
///
/// V1 Threshold Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeMinThreshold.sol#L10
///
/// V1 Proxy Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeProxyAddress.sol#L10
#[derive(Clone, Copy, Debug)]
pub enum QuorumState {
    Invalid,
    InProgress,
    Accepted,
    Rejected,
}

impl From<U256> for QuorumState {
    fn from(uint: U256) -> Self {
        match uint.low_u64() {
            0 => QuorumState::Invalid,
            1 => QuorumState::InProgress,
            2 => QuorumState::Accepted,
            3 => QuorumState::Rejected,
            _ => unreachable!("unrecognized `QuorumState`: {}", uint),
        }
    }
}

#[derive(Clone, Debug)]
pub enum VotingState {
    Keys(KeysVotingState),
    Threshold(ThresholdVotingState),
    Proxy(ProxyVotingState),
}

impl From<KeysVotingState> for VotingState {
    fn from(keys_voting_state: KeysVotingState) -> Self {
        VotingState::Keys(keys_voting_state)
    }
}

impl From<ThresholdVotingState> for VotingState {
    fn from(threshold_voting_state: ThresholdVotingState) -> Self {
        VotingState::Threshold(threshold_voting_state)
    }
}

impl From<ProxyVotingState> for VotingState {
    fn from(proxy_voting_state: ProxyVotingState) -> Self {
        VotingState::Proxy(proxy_voting_state)
    }
}

impl VotingState {
    pub fn contract_name(&self) -> String {
        match self {
            VotingState::Keys(_) => "VotingToChangeKeys.sol".into(),
            VotingState::Threshold(_) => "VotingToChangeMinThreshold.sol".into(),
            VotingState::Proxy(_) => "VotingToChangeProxyAddress.sol".into(),
        }
    }

    pub fn email_text(&self) -> String {
        match self {
            VotingState::Keys(state) => state.email_text(),
            VotingState::Threshold(state) => state.email_text(),
            VotingState::Proxy(state) => state.email_text(),
        }
    }
}

/// V1 Key's Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeKeys.sol#L22
#[derive(Clone, Debug)]
pub struct KeysVotingState {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub affected_key: Address,
    pub affected_key_type: KeyType,
    pub mining_key: Address,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub ballot_type: BallotType,
    pub index: U256,
    pub min_threshold_of_voters: U256,
    pub creator: Address,
    pub memo: String
}

impl From<Vec<ethabi::Token>> for KeysVotingState {
    fn from(tokens: Vec<ethabi::Token>) -> Self {
        let start_time = {
            let uint = tokens[0].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let end_time = {
            let uint = tokens[1].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let affected_key = tokens[2].clone().to_address().unwrap();
        let affected_key_type = tokens[3].clone().to_uint().unwrap().into();
        let mining_key = tokens[4].clone().to_address().unwrap();
        let total_voters = tokens[5].clone().to_uint().unwrap();
        let progress = tokens[6].clone().to_int().unwrap();
        let is_finalized = tokens[7].clone().to_bool().unwrap();
        let quorum_state = tokens[8].clone().to_uint().unwrap().into();
        let ballot_type = tokens[9].clone().to_uint().unwrap().into();
        let index = tokens[10].clone().to_uint().unwrap();
        let min_threshold_of_voters = tokens[11].clone().to_uint().unwrap();
        let creator = tokens[12].clone().to_address().unwrap();
        let memo = tokens[13].clone().to_string().unwrap();
        KeysVotingState {
            start_time,
            end_time,
            affected_key,
            affected_key_type,
            mining_key,
            total_voters,
            progress,
            is_finalized,
            quorum_state,
            ballot_type,
            index,
            min_threshold_of_voters,
            creator,
            memo,
        }
    }
}

impl KeysVotingState {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Ballot Type: {:?}\n\
            Affected Key: {:?}\n\
            Affected Key Type: {:?}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Number of Votes Required to Make Change: {}\n\
            Mining Key: {:?}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.ballot_type,
            self.affected_key,
            self.affected_key_type,
            self.is_finalized,
            self.total_voters,
            self.min_threshold_of_voters,
            self.mining_key,
            self.creator,
            self.memo,
        )
    }
}

/// V1 Threshold Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeMinThreshold.sol#L20
#[derive(Clone, Debug)]
pub struct ThresholdVotingState {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub index: U256,
    pub min_threshold_of_voters: U256,
    pub proposed_value: U256,
    pub creator: Address,
    pub memo: String,
}

impl From<Vec<ethabi::Token>> for ThresholdVotingState {
    fn from(tokens: Vec<ethabi::Token>) -> Self {
        let start_time = {
            let uint = tokens[0].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let end_time = {
            let uint = tokens[1].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let total_voters = tokens[2].clone().to_uint().unwrap();
        let progress = tokens[3].clone().to_int().unwrap();
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let quorum_state = tokens[5].clone().to_uint().unwrap().into();
        let index = tokens[6].clone().to_uint().unwrap();
        let min_threshold_of_voters = tokens[7].clone().to_uint().unwrap();
        let proposed_value = tokens[8].clone().to_uint().unwrap();
        let creator = tokens[9].clone().to_address().unwrap();
        let memo = tokens[10].clone().to_string().unwrap();
        ThresholdVotingState {
            start_time,
            end_time,
            total_voters,
            progress,
            is_finalized,
            quorum_state,
            index,
            min_threshold_of_voters,
            proposed_value,
            creator,
            memo,
        }
    }
}

impl ThresholdVotingState {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Proposed New Min. Threshold: {}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Number of Votes Required to Make Change: {}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.proposed_value,
            self.is_finalized,
            self.total_voters,
            self.min_threshold_of_voters,
            self.creator,
            self.memo,
        )
    }
}

/// V1 Proxy Contract:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeProxyAddress.sol#L19
#[derive(Clone, Debug)]
pub struct ProxyVotingState {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub quorum_state: QuorumState,
    pub index: U256,
    pub min_threshold_of_voters: U256,
    pub proposed_value: Address,
    pub contract_type: U256,
    pub creator: Address,
    pub memo: String,
}

impl From<Vec<ethabi::Token>> for ProxyVotingState {
    fn from(tokens: Vec<ethabi::Token>) -> Self {
        let start_time = {
            let uint = tokens[0].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let end_time = {
            let uint = tokens[1].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let total_voters = tokens[2].clone().to_uint().unwrap();
        let progress = tokens[3].clone().to_int().unwrap();
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let quorum_state = tokens[5].clone().to_uint().unwrap().into();
        let index = tokens[6].clone().to_uint().unwrap();
        let min_threshold_of_voters = tokens[7].clone().to_uint().unwrap();
        let proposed_value = tokens[8].clone().to_address().unwrap();
        let contract_type = tokens[9].clone().to_uint().unwrap();
        let creator = tokens[10].clone().to_address().unwrap();
        let memo = tokens[11].clone().to_string().unwrap();
        ProxyVotingState {
            start_time,
            end_time,
            total_voters,
            progress,
            is_finalized,
            quorum_state,
            index,
            min_threshold_of_voters,
            proposed_value,
            contract_type,
            creator,
            memo,
        }
    }
}

impl ProxyVotingState {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Proposed New Proxy Address: {:?}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Number of Votes Required for Change: {}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.proposed_value,
            self.is_finalized,
            self.total_voters,
            self.min_threshold_of_voters,
            self.creator,
            self.memo,
        )
    }
}
