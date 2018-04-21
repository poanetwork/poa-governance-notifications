use std::convert::TryFrom;
use std::i64;
use std::num::ParseIntError;
use std::u64;

use chrono::{DateTime, NaiveDateTime, Utc};
use ethereum_types::U256;

pub fn hex_string_to_u64(hex: &str) -> Result<u64, ParseIntError> {
    let hex = hex.trim_left_matches("0x");
    u64::from_str_radix(hex, 16)
}

pub fn u256_to_datetime(uint: U256) -> DateTime<Utc> {
    let n_secs = i64::try_from(uint.as_u64()).unwrap();
    let timestamp = NaiveDateTime::from_timestamp(n_secs, 0);
    DateTime::from_utc(timestamp, Utc)
}
