use std::sync::Arc;

use crate::domain::{Comment, CommentError};
use crate::ports::{CommentRepository, RepositoryError};

pub struct CreateComment {
    repo: Arc<dyn CommentRepository>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateCommentError {
    Invalid(CommentError),
    Repository(RepositoryError),
}

impl CreateComment {
    pub fn new(repo: Arc<dyn CommentRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, post_id: String, body: String) -> Result<Comment, CreateCommentError> {
        let comment = Comment::new(post_id, body).map_err(CreateCommentError::Invalid)?;
        self.repo
            .insert(&comment)
            .map_err(CreateCommentError::Repository)?;
        Ok(comment)
    }
}
