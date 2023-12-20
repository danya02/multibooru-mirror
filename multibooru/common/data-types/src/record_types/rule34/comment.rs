use serde::{Deserialize, Serialize};

use crate::DateTime;

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct CommentRecord {
    /// This is the comment's ID on the Rule34 site
    pub id: u64,

    pub state: CommentState,
}

impl CommentRecord {
    pub fn entity_id(&self) -> u64 {
        self.id
    }
}

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub enum CommentState {
    /// The comment with the given ID does not exist.
    /// It may not have been created yet, or been deleted.
    Absent,

    /// The comment with the given ID exists as follows.
    Present {
        /// The post that this comment is associated with.
        post_id: u64,
        /// The comment author's user ID.
        /// This is provided in API responses but not in the website.
        author_id: Option<u64>,
        /// The comment author's username.
        author_name: String,
        /// The comment creation date.
        /// In API responses, this is not accurate;
        /// instead it just returns the server time at the moment the response was rendered.
        created_at: DateTime,
        /// The comment's contents.
        text: String,
        /// The comment's score, based on upvotes and downvotes.
        /// It is not provided in API responses.
        score: Option<i64>,
        /// Whether the comment has been reported.
        /// This is not provided in API responses.
        is_reported: Option<bool>,
    },
}
