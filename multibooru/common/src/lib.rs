pub mod snowflake;
pub use snowflake::Snowflake;
pub mod entity_state;
pub mod proxy_maker;
pub use proxy_maker::{with_media_proxy, with_proxy};

pub const USER_AGENT: &str = concat!(
    "multibooru-scraper/",
    env!("CARGO_PKG_VERSION"),
    ", +https://github.com/danya02/multibooru-mirror"
);
