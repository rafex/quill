use std::sync::Arc;

use crate::domain::User;
use crate::ports::{RepositoryError, UserRepository};

pub struct GetUserById {
    repo: Arc<dyn UserRepository>,
}

impl GetUserById {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        self.repo.find_by_id(id)
    }
}
