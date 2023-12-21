use serde::{Deserialize, Serialize};

use crate::DateTime;

#[derive(Serialize, Deserialize, Clone, Debug, Hash)]
pub struct CommentRecord {
    /// This is the comment's ID on the Danbooru site
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
    /// It may not have been created yet.
    Absent,

    /// The comment with the given ID exists as follows.
    Present {
        /// The post that this comment is associated with.
        post_id: u64,
        /// The comment creation and update date. If it wasn't updated yet, they will match.
        created_at: DateTime,
        updated_at: DateTime,

        /// The user IDs of the creator and updater. If it wasn't updated yet, they will match.
        creator_id: u64,
        updater_id: u64,

        /// The text content
        body: String,

        /// The total up/down score on the comment.
        /// It doesn't seem like there is a way to separate them.
        /// Changes to this do not count as updates.
        score: i64,

        do_not_bump_post: bool,
        is_sticky: bool,
        // is_deleted is implicitly false
    },

    /// The comment with the given ID has been deleted.
    Deleted {
        /// The post that the comment would have been associated with.
        post_id: u64,

        /// The comment creation and update date. Update here implies deletion.
        created_at: DateTime,
        updated_at: DateTime,

        /// The user IDs of the creator and updater.
        /// If the user deleted the comment themselves, then they will match;
        /// otherwise, the updating user is expected to be at least a Moderator.
        creator_id: u64,
        updater_id: u64,
        // is_deleted is implicitly true
    },
}
