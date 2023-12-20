use std::{
    fmt::{Display, Formatter},
    str::FromStr,
};

use serde::{Deserialize, Serialize};

/// The main identifier type for our application shall be the Snowflake:
/// a 64-bit integer that is unique across all entities,
/// that encodes the time of creation.
///
/// The first 48 bits are the timestamp in milliseconds since the Unix epoch, and the top bit is expected to be zero.
/// This gives a range of about 4460 years, and the scheme will stop working on October 17, 6429.
///
/// The next 16 bits are reserved for ensuring uniqueness. Their meaning is not yet defined.
#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize, Default,
)]
pub struct Snowflake(i64);

impl Snowflake {
    /// Create a new Snowflake.
    ///
    /// Accepts an argument which is used to set the uniqueness bits of the snowflake.
    pub fn new(uid: u16) -> Self {
        let now = std::time::SystemTime::now();
        let since_epoch = now
            .duration_since(std::time::UNIX_EPOCH)
            .expect("Time is before Unix epoch.");
        let millis = since_epoch.as_millis();
        let sn_id = millis << 16 | uid as u128;
        let sn_id = sn_id
            .try_into()
            .expect("Time is too far in the future to fit in an i64-based Snowflake.");
        Snowflake(u64_to_i64(sn_id))
    }

    /// Replace the low bits with the provided value.
    pub fn with_low_bits(self, new_uid: u16) -> Self {
        let mut value = self.0;
        let mask = i64::MAX ^ 0x1ff; // all bits set, except for the last 16
        value = value & mask; // clears the bottom 16 bits
        value = value | (new_uid as i64); // assigns them based on the given value
        Self(value)
    }

    /// Get the time that this snowflake corresponds to.
    pub fn as_time(&self) -> std::time::SystemTime {
        let millis_since_epoch = self.0 >> 16;
        std::time::UNIX_EPOCH + std::time::Duration::from_millis(millis_since_epoch as u64)
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

#[cfg(test)]
mod test {
    use std::time::Duration;

    use super::*;

    #[test]
    fn date_is_now() {
        let sn = Snowflake::new(0);
        assert!(sn.as_time().elapsed().unwrap() < Duration::from_millis(1))
    }

    #[test]
    fn date_preserved_over_low_bits() {
        let sn = Snowflake::new(0);
        let new_sn = sn.clone().with_low_bits(u16::MAX);
        assert_eq!(sn.as_time(), new_sn.as_time());
    }
}
