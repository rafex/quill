use crate::domain::Post;
use crate::ports::RepositoryError;

pub trait PostRepository: Send + Sync {
    fn insert(&self, post: &Post) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Post>, RepositoryError>;

    /// Writes a fresh outbox event for every existing post (each with its
    /// own event_id) without touching the `posts` table. Used to drive a
    /// reindex without re-running creation.
    fn republish_all(&self) -> Result<usize, RepositoryError>;
}
