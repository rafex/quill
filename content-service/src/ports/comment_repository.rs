use crate::domain::Comment;
use crate::ports::RepositoryError;

pub trait CommentRepository: Send + Sync {
    fn insert(&self, comment: &Comment) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Comment>, RepositoryError>;
}
