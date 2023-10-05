use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The main identifier type for our application shall be the Snowflake:
/// a 64-bit integer that is unique across all entities,
/// that encodes the time of creation.
///
/// The first 48 bits are the timestamp in milliseconds since the Unix epoch.
/// This gives a range of about 8920 years.
///
/// The next 16 bits are reserved, and are currently set to 0.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Snowflake(i64);

impl Snowflake {
    /// Create a new Snowflake.
    ///
    /// This function is not thread-safe:
    /// if two threads call it at the same time,
    /// they can get the same Snowflake.
    pub fn new() -> Self {
        let now = std::time::SystemTime::now();
        let since_epoch = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time is before Unix epoch.");
        let millis = since_epoch.as_millis();
        let sn_id = millis << 16;
        let sn_id = sn_id
            .try_into()
            .expect("Time is too far in the future to fit in an i64-based Snowflake.");
        Snowflake(u64_to_i64(sn_id))
    }
}

/// Losslessly convert an u64 to an i64, copying every bit.
/// The i64 might be negative as a result of this.
/// Has the same effect as reinterpreting the bits as an i64.
pub fn u64_to_i64(x: u64) -> i64 {
    let bytes = x.to_be_bytes();
    i64::from_be_bytes(bytes)
}

/// Losslessly convert an i64 to an u64, copying every bit.
/// Has the same effect as reinterpreting the bits as an u64.
pub fn i64_to_u64(x: i64) -> u64 {
    let bytes = x.to_be_bytes();
    u64::from_be_bytes(bytes)
}

impl From<Snowflake> for i64 {
    fn from(snowflake: Snowflake) -> Self {
        snowflake.0
    }
}

impl From<i64> for Snowflake {
    fn from(id: i64) -> Self {
        Snowflake(id)
    }
}

impl Display for Snowflake {
    fn fmt(&self, f: &mut Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", self.0)
    }
}

impl FromStr for Snowflake {
    type Err = std::num::ParseIntError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        s.parse().map(Snowflake)
    }
}