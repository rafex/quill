use crate::domain::Topic;
use crate::ports::RepositoryError;

pub trait TopicRepository: Send + Sync {
    fn insert(&self, topic: &Topic) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Topic>, RepositoryError>;
}
