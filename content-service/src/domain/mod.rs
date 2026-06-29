mod category;
mod comment;
mod post;
mod topic;

pub use category::{Category, CategoryError};
pub use comment::{Comment, CommentError};
pub use post::{Post, PostError};
pub use topic::{Topic, TopicError};

use std::time::{SystemTime, UNIX_EPOCH};

pub(crate) fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
