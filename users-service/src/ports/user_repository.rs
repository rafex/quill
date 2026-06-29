use crate::domain::User;

pub trait UserRepository: Send + Sync {
    fn insert(&self, user: &User) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError>;
}

#[derive(Debug, PartialEq, Eq)]
pub enum RepositoryError {
    Duplicate,
    Unknown(String),
}
