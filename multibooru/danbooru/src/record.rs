use crate::{post::Post, tag::Tag};
use common::make_entity_state;
use serde::{Deserialize, Serialize};

make_entity_state! {
    Entity, Record {
        Post: u64 => PostState,
        Tag: u64 => TagState,
    }
}

/// An enum representing the state of a post.
/// A post can either exist, in which case it has the given data,
/// or it can be missing.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum PostState {
    /// The post exists.
    Exists(Post),
    /// The post is missing.
    /// This might mean that it was deleted, or that its ID is in the future.
    Missing,
}

impl From<Post> for PostState {
    fn from(post: Post) -> Self {
        PostState::Exists(post)
    }
}

/// An enum representing the state of a tag.
/// A tag can either exist, in which case it has the given data,
/// or it can be missing.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum TagState {
    /// The tag exists.
    Exists(Tag),
    /// The tag is missing.
    Missing,
}
