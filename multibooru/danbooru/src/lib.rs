pub mod post;
pub use post::Post;
pub mod record;
pub mod tag;
use serde::{Serialize, Deserialize};
pub use tag::Tag;
pub mod post_version;
pub use post_version::PostVersion;

#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Rating {
    #[serde(rename = "g")]
    General,
    #[serde(rename = "s")]
    Sensitive,
    #[serde(rename = "q")]
    Questionable,
    #[serde(rename = "e")]
    Explicit,
}
