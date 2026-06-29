use std::sync::Arc;

use crate::domain::{Topic, TopicError};
use crate::ports::{RepositoryError, TopicRepository};

pub struct CreateTopic {
    repo: Arc<dyn TopicRepository>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateTopicError {
    Invalid(TopicError),
    Repository(RepositoryError),
}

impl CreateTopic {
    pub fn new(repo: Arc<dyn TopicRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(
        &self,
        category_id: String,
        title: String,
        slug: String,
    ) -> Result<Topic, CreateTopicError> {
        let topic = Topic::new(category_id, title, slug).map_err(CreateTopicError::Invalid)?;
        self.repo
            .insert(&topic)
            .map_err(CreateTopicError::Repository)?;
        Ok(topic)
    }
}
