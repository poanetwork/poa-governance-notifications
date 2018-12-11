// Don't throw a compilation warning for the to be deprecated: 'ethereum_types::H256::low_u64'.
#![allow(deprecated)]

use chrono::{DateTime, NaiveDateTime, Utc};
use ethabi;
use web3::types::{Address, H256, U256};

use error::{Error, Result};

/// Converts a `U256` timestamp to a UTC `DateTime`.
pub fn u256_to_datetime(uint: U256) -> DateTime<Utc> {
    let timestamp = uint.low_u64() as i64;
    let naive = NaiveDateTime::from_timestamp(timestamp, 0);
    DateTime::from_utc(naive, Utc)
}

/// Identifies what type of key is being voted on by the `votingToChangeKeys.sol` contract. This
/// enum is used in the V1 and V2 Keys contracts.
///
/// V1 Keys Contract (`KeyType` is used within the contract's `votingState`):
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeKeys.sol#L11
///
/// V2 `KeyTypes` enum (used by the V2 Keys Contract's `ballotInfo`):
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/abstracts/EnumKeyTypes.sol#L5
#[derive(Clone, Debug)]
pub enum KeyType {
    InvalidKey,
    MiningKey,
    VotingKey,
    PayoutKey,
}

impl From<U256> for KeyType {
    fn from(key_type: U256) -> Self {
        match key_type.low_u64() {
            0 => KeyType::InvalidKey,
            1 => KeyType::MiningKey,
            2 => KeyType::VotingKey,
            3 => KeyType::PayoutKey,
            n => unreachable!("unrecognized `KeyType`: {}", n),
        }
    }
}

/// V1 Keys Contract (used in `BallotCreated` event and within the `votingState`):
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeKeys.sol#L10
///
/// V1 Threshold Contract (used in `BallotCreated` event):
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeMinThreshold.sol#L89
///
/// V1 Proxy Contract (used in `BallotCreated` event):
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeProxyAddress.sol#L85
///
/// Note: V1 contracts do not use the `Emission` variant.
///
/// V2 - all contracts use the same enum:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/abstracts/EnumBallotTypes.sol#L5
#[derive(Clone, Copy, Debug)]
pub enum BallotType {
    InvalidKey,
    AddKey,
    RemoveKey,
    SwapKey,
    Threshold,
    Proxy,
    Emission,
}

/// Converts a `U256` (from a V2 keys contract's voting-state) into a `BallotType`.
impl From<U256> for BallotType {
    fn from(uint: U256) -> Self {
        BallotType::from(H256::from(uint))
    }
}

/// Converts an `H256` from a `web3::types::Log`'s `topics` vector to a `BallotType`.
impl From<H256> for BallotType {
    fn from(topic: H256) -> Self {
        match topic.low_u64() {
            0 => BallotType::InvalidKey,
            1 => BallotType::AddKey,
            2 => BallotType::RemoveKey,
            3 => BallotType::SwapKey,
            4 => BallotType::Threshold,
            5 => BallotType::Proxy,
            6 => BallotType::Emission,
            n => unreachable!("unrecognized `BallotType`: {}", n),
        }
    }
}

/// A parsed `BallotCreated` event log. All V1 and V2 contracts use the same `BallotCreated` event.
///
/// V1 Keys Contract's `BallotCreated` event:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeKeys.sol#L45
///
/// V1 Threshold Contract's `BallotCreated` event:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeMinThreshold.sol#L40
///
/// V1 Proxy Contract's `BallotCreated` event:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/aa45e19ca50f7cae308c1281d950245b0c65182a/contracts/VotingToChangeMinThreshold.sol#L40
///
/// V2 - all contracts use the same `BallotCreated` event:
/// https://github.com/poanetwork/poa-network-consensus-contracts/blob/ec307069302fdf6647e8b1bdc13093960913b266/contracts/abstracts/VotingTo.sol#L30
#[derive(Clone, Copy, Debug)]
pub struct BallotCreatedLog {
    pub block_number: U256,
    pub ballot_id: U256,
    pub ballot_type: BallotType,
    pub creator: Address,
}

impl BallotCreatedLog {
    pub fn from_ethabi_log(log: ethabi::Log, block_number: U256) -> Result<Self> {
        let mut ballot_id: Option<U256> = None;
        let mut ballot_type: Option<BallotType> = None;
        let mut creator: Option<Address> = None;
        for ethabi::LogParam { name, value } in log.params {
            match name.as_ref() {
                "id" => ballot_id = value.to_uint(),
                "ballotType" => ballot_type = value.to_uint().map(BallotType::from),
                "creator" => creator = value.to_address(),
                name => unreachable!("Found unknown `BallotCreated` event log field: {}", name),
            };
        }
        let ballot_id = match ballot_id {
            Some(id) => id,
            None => return Err(Error::FailedToParseBallotCreatedLog("missing `id`".into())),
        };
        let ballot_type = match ballot_type {
            Some(ballot_type) => ballot_type,
            None => return Err(Error::FailedToParseBallotCreatedLog("missing `ballot_type`".into())),
        };
        let creator = match creator {
            Some(creator) => creator,
            None => return Err(Error::FailedToParseBallotCreatedLog("missing `creator`".into())),
        };
        Ok(BallotCreatedLog { ballot_id, ballot_type, creator, block_number })
    }
}
