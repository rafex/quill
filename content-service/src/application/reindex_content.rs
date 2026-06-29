use std::sync::Arc;

use crate::ports::{CommentRepository, PostRepository, RepositoryError};

pub struct ReindexContent {
    posts: Arc<dyn PostRepository>,
    comments: Arc<dyn CommentRepository>,
}

impl ReindexContent {
    pub fn new(posts: Arc<dyn PostRepository>, comments: Arc<dyn CommentRepository>) -> Self {
        Self { posts, comments }
    }

    /// Returns (posts_republished, comments_republished).
    pub fn execute(&self) -> Result<(usize, usize), RepositoryError> {
        let posts = self.posts.republish_all()?;
        let comments = self.comments.republish_all()?;
        Ok((posts, comments))
    }
}
