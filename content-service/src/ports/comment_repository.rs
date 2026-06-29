use crate::domain::Comment;
use crate::ports::RepositoryError;

pub trait CommentRepository: Send + Sync {
    fn insert(&self, comment: &Comment) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Comment>, RepositoryError>;

    /// Writes a fresh outbox event for every existing comment (each with
    /// its own event_id) without touching the `comments` table. Used to
    /// drive a reindex without re-running creation.
    fn republish_all(&self) -> Result<usize, RepositoryError>;
}
