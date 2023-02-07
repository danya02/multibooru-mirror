use crate::{post::Post, tag::Tag, post_version::PostVersion};
use common::make_entity_state;
use serde::{Deserialize, Serialize};

make_entity_state! {
    Entity, Record {
        Post: i64 => PostState,
        Tag: i64 => TagState,
        PostVersion: i64 => PostVersion,
    }
}

impl Entity {
    /// Get the ID of the type of entity this is.
    /// This is used for database lookups and storage,
    /// so this must not change between versions.
    /// This should also match between different boorus:
    /// for example, a post on Danbooru and a post on Gelbooru
    /// should have the same type ID.
    pub fn type_id(&self) -> u32 {
        match self {
            Entity::Post(_) => 1,
            Entity::Tag(_) => 2,
            Entity::PostVersion(_) => 3,
        }
    }

    /// Get the booru ID of this entity.
    pub fn booru_id(&self) -> i64 {
        match self {
            Entity::Post(id) => *id,
            Entity::Tag(id) => *id,
            Entity::PostVersion(id) => *id,
        }
    }
}

/// An enum representing the state of a post.
/// A post can either exist, in which case it has the given data,
/// or it can be missing.
#[derive(Debug, Clone, PartialEq, Hash, Serialize, Deserialize)]
pub enum PostState {
    /// The post exists.
    Exists(Post),
    /// The post is missing, but it might exist in the future.
    /// This usually means the post's ID is past the last known post ID.
    MissingTemporarily,
    /// The post is missing, and it is expected to never exist anymore.
    /// This usually means that the post's ID is in the past,
    /// but the post was not found --
    /// it was probably deleted.
    MissingPermanently,
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
    /// The tag is missing, but it might exist in the future.
    MissingTemporarily,
    /// The tag is missing, and it is expected to never exist anymore.
    MissingPermanently,
}
