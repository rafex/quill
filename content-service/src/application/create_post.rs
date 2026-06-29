use std::sync::Arc;

use crate::domain::{Post, PostError};
use crate::ports::{PostRepository, RepositoryError};

pub struct CreatePost {
    repo: Arc<dyn PostRepository>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreatePostError {
    Invalid(PostError),
    Repository(RepositoryError),
}

impl CreatePost {
    pub fn new(repo: Arc<dyn PostRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(
        &self,
        topic_id: String,
        title: String,
        slug: String,
        body: String,
    ) -> Result<Post, CreatePostError> {
        let post = Post::new(topic_id, title, slug, body).map_err(CreatePostError::Invalid)?;
        self.repo
            .insert(&post)
            .map_err(CreatePostError::Repository)?;
        Ok(post)
    }
}
