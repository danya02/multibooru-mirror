pub mod snowflake;
pub use snowflake::Snowflake;
pub mod record;
pub mod record_types;

pub type DateTime = chrono::DateTime<chrono::Utc>;

pub const USER_AGENT: &'static str = concat!(
    "multibooru-mirror/",
    env!("CARGO_PKG_VERSION"),
    ", +https://github.com/danya02/multibooru-mirror"
);
