use crate::domain::Post;
use crate::ports::RepositoryError;

pub trait PostRepository: Send + Sync {
    fn insert(&self, post: &Post) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Post>, RepositoryError>;
}
