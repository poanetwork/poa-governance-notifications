use chrono::{DateTime, Utc};
use ethabi;
use web3::types::{Address, U256};

use response::common::{u256_to_datetime, BallotType, KeyType};

#[derive(Clone, Debug)]
pub enum BallotInfo {
    Keys(KeysBallotInfo),
    Threshold(ThresholdBallotInfo),
    Proxy(ProxyBallotInfo),
    Emission(EmissionBallotInfo),
}

impl From<KeysBallotInfo> for BallotInfo {
    fn from(keys_ballot_info: KeysBallotInfo) -> Self {
        BallotInfo::Keys(keys_ballot_info)
    }
}

impl From<ThresholdBallotInfo> for BallotInfo {
    fn from(threshold_ballot_info: ThresholdBallotInfo) -> Self {
        BallotInfo::Threshold(threshold_ballot_info)
    }
}

impl From<ProxyBallotInfo> for BallotInfo {
    fn from(proxy_ballot_info: ProxyBallotInfo) -> Self {
        BallotInfo::Proxy(proxy_ballot_info)
    }
}

impl From<EmissionBallotInfo> for BallotInfo {
    fn from(emission_ballot_info: EmissionBallotInfo) -> Self {
        BallotInfo::Emission(emission_ballot_info)
    }
}

impl BallotInfo {
    pub fn contract_name(&self) -> String { 
        match self {
            BallotInfo::Keys(_) => "VotingToChangeKeys.sol".into(),
            BallotInfo::Threshold(_) => "VotingToChangeMinThreshold.sol".into(),
            BallotInfo::Proxy(_) => "VotingToChangeProxyAddress.sol".into(),
            BallotInfo::Emission(_) => "VotingToManageEmissionFunds.sol".into(),
        }
    }

    pub fn email_text(&self) -> String {    
        match self {
            BallotInfo::Keys(info) => info.email_text(),
            BallotInfo::Threshold(info) => info.email_text(),
            BallotInfo::Proxy(info) => info.email_text(),
            BallotInfo::Emission(info) => info.email_text(),
        }
    }
}

/// Returned by the V2 Keys contract's `.getBallotInfo()` function:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/VotingToChangeKeys.sol#L7
#[derive(Clone, Debug)]
pub struct KeysBallotInfo {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub affected_key: Address,
    pub affected_key_type: KeyType,
    pub new_voting_key: Address,
    pub new_payout_key: Address,
    pub mining_key: Address,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub ballot_type: BallotType,
    pub creator: Address,
    pub memo: String,
    pub can_be_finalized_now: bool,
}

impl From<Vec<ethabi::Token>> for KeysBallotInfo {
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
        let new_voting_key = tokens[4].clone().to_address().unwrap();
        let new_payout_key = tokens[5].clone().to_address().unwrap();
        let mining_key = tokens[6].clone().to_address().unwrap();
        let total_voters = tokens[7].clone().to_uint().unwrap();
        let progress = tokens[8].clone().to_int().unwrap();
        let is_finalized = tokens[9].clone().to_bool().unwrap();
        let ballot_type = tokens[10].clone().to_uint().unwrap().into();
        let creator = tokens[11].clone().to_address().unwrap();
        let memo = tokens[12].clone().to_string().unwrap();
        let can_be_finalized_now = tokens[13].clone().to_bool().unwrap();
        KeysBallotInfo {
            start_time,
            end_time,
            affected_key,
            affected_key_type,
            new_voting_key,
            new_payout_key,
            mining_key,
            total_voters,
            progress,
            is_finalized,
            ballot_type,
            creator,
            memo,
            can_be_finalized_now,
        }
    }
}

impl KeysBallotInfo {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Ballot Type: {:?}\n\
            Affected Key: {:?}\n\
            Affected Key Type: {:?}\n\
            New Voting Key: {:?}\n\
            New Payout Key: {:?}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Mining Key: {:?}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.ballot_type,
            self.affected_key,
            self.affected_key_type,
            self.new_voting_key,
            self.new_payout_key,
            self.is_finalized,
            self.total_voters,
            self.mining_key,
            self.creator,
            self.memo,
        )
    }
}

/// Returned by the V2 Threshold Contract's `.getBallotInfo()` function:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/VotingToChangeMinThreshold.sol#L30
#[derive(Clone, Debug)]
pub struct ThresholdBallotInfo {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub proposed_value: U256,
    pub creator: Address,
    pub memo: String,
    pub can_be_finalized_now: bool,
    // pub already_voted: bool,
}

impl From<Vec<ethabi::Token>> for ThresholdBallotInfo {
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
        let progress = tokens[3].clone().to_uint().unwrap();
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let proposed_value = tokens[5].clone().to_uint().unwrap();
        let creator = tokens[6].clone().to_address().unwrap();
        let memo = tokens[7].clone().to_string().unwrap();
        let can_be_finalized_now = tokens[8].clone().to_bool().unwrap();
        // let already_voted = tokens[9].clone().to_bool().unwrap();
        ThresholdBallotInfo {
            start_time,
            end_time,
            total_voters,
            progress,
            is_finalized,
            proposed_value,
            creator,
            memo,
            can_be_finalized_now,
            // already_voted,
        }
    }
}

impl ThresholdBallotInfo {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Proposed New Min. Threshold: {}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.proposed_value,
            self.is_finalized,
            self.total_voters,
            self.creator,
            self.memo,
        )
    }
}

/// Returned by the V2 Proxy Contract's `.getBallotInfo()` function:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/VotingToChangeProxyAddress.sol#L30
#[derive(Clone, Debug)]
pub struct ProxyBallotInfo {
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub total_voters: U256,
    pub progress: U256,
    pub is_finalized: bool,
    pub proposed_value: Address,
    pub contract_type: U256,
    pub creator: Address,
    pub memo: String,
    pub can_be_finalized_now: bool,
    // pub already_voted: bool,
}

impl From<Vec<ethabi::Token>> for ProxyBallotInfo {
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
        let progress = tokens[3].clone().to_uint().unwrap();
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let proposed_value = tokens[5].clone().to_address().unwrap();
        let contract_type = tokens[6].clone().to_uint().unwrap();
        let creator = tokens[7].clone().to_address().unwrap();
        let memo = tokens[8].clone().to_string().unwrap();
        let can_be_finalized_now = tokens[9].clone().to_bool().unwrap();
        // let already_voted = tokens[10].clone().to_bool().unwrap();
        ProxyBallotInfo {
            start_time,
            end_time,
            total_voters,
            progress,
            is_finalized,
            proposed_value,
            contract_type,
            creator,
            memo,
            can_be_finalized_now,
            // already_voted,
        }
    }
}

impl ProxyBallotInfo {
    fn email_text(&self) -> String {
        format!(
            "Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Proposed New Proxy Address: {:?}\n\
            Voting has Finished: {}\n\
            Number of Votes Made: {}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.start_time,
            self.end_time,
            self.proposed_value,
            self.is_finalized,
            self.total_voters,
            self.creator,
            self.memo,
        )
    }
}

/// Returned by the V2 Emission Contract's `.getBallotInfo()` function:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/VotingToManageEmissionFunds.sol#L126
#[derive(Clone, Debug)]
pub struct EmissionBallotInfo {
    pub creation_time: DateTime<Utc>,
    pub start_time: DateTime<Utc>,
    pub end_time: DateTime<Utc>,
    pub is_canceled: bool,
    pub is_finalized: bool,
    pub creator: Address,
    pub memo: String,
    pub ammount: U256,
    pub burn_votes: U256,
    pub freeze_votes: U256,
    pub send_votes: U256,
    pub receiver: Address,
}

impl From<Vec<ethabi::Token>> for EmissionBallotInfo {
    fn from(tokens: Vec<ethabi::Token>) -> Self {
        let creation_time = {
            let uint = tokens[0].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let start_time = {
            let uint = tokens[1].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let end_time = {
            let uint = tokens[2].clone().to_uint().unwrap();
            u256_to_datetime(uint)
        };
        let is_canceled = tokens[3].clone().to_bool().unwrap();
        let is_finalized = tokens[4].clone().to_bool().unwrap();
        let creator = tokens[5].clone().to_address().unwrap();
        let memo = tokens[6].clone().to_string().unwrap();
        let ammount = tokens[7].clone().to_uint().unwrap();
        let burn_votes = tokens[8].clone().to_uint().unwrap();
        let freeze_votes = tokens[9].clone().to_uint().unwrap();
        let send_votes = tokens[10].clone().to_uint().unwrap();
        let receiver = tokens[11].clone().to_address().unwrap();
        EmissionBallotInfo {
            creation_time,
            start_time,
            end_time,
            is_canceled,
            is_finalized,
            creator,
            memo,
            ammount,
            burn_votes,
            freeze_votes,
            send_votes,
            receiver,
        }
    }
}

impl EmissionBallotInfo {
    fn email_text(&self) -> String {
        format!(
            "Creation Time: {}\n\
            Voting Start Time: {}\n\
            Voting End Time: {}\n\
            Ammount: {}\n\
            Burn Votes: {}\n\
            Freeze Votes: {}\n\
            Send Votes: {}\n\
            Receiver: {:?}\n\
            Voting was Canceled: {}\n\
            Voting has Finished: {}\n\
            Ballot Creator: {:?}\n\
            Memo: {}\n",
            self.creation_time,
            self.start_time,
            self.end_time,
            self.ammount,
            self.burn_votes,
            self.freeze_votes,
            self.send_votes,
            self.receiver,
            self.is_canceled,
            self.is_finalized,
            self.creator,
            self.memo,
        )
    }
}
