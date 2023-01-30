use serde::{Deserialize, Serialize};

use crate::post::Post;

/// A record of some state on Danbooru.
/// This could be the state of a post, a comment, a tag, or anything else.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum Record {
    /// A post has this data.
    Post(Post),
    /// A post with the given ID does not exist.
    /// It could have been deleted, or it is in the future.
    PostMissing(i64),
}
