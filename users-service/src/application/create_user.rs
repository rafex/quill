use std::sync::Arc;

use crate::domain::{User, UserError};
use crate::ports::{RepositoryError, UserRepository};

pub struct CreateUser {
    repo: Arc<dyn UserRepository>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateUserError {
    Invalid(UserError),
    Repository(RepositoryError),
}

impl CreateUser {
    pub fn new(repo: Arc<dyn UserRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, username: String, email: String) -> Result<User, CreateUserError> {
        let user = User::new(username, email).map_err(CreateUserError::Invalid)?;
        self.repo
            .insert(&user)
            .map_err(CreateUserError::Repository)?;
        Ok(user)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::sync::Mutex;

    struct InMemoryUserRepository {
        users: Mutex<Vec<User>>,
    }

    impl InMemoryUserRepository {
        fn new() -> Self {
            Self {
                users: Mutex::new(Vec::new()),
            }
        }
    }

    impl UserRepository for InMemoryUserRepository {
        fn insert(&self, user: &User) -> Result<(), RepositoryError> {
            let mut users = self.users.lock().unwrap();
            if users.iter().any(|u| u.email == user.email) {
                return Err(RepositoryError::Duplicate);
            }
            users.push(user.clone());
            Ok(())
        }

        fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
            let users = self.users.lock().unwrap();
            Ok(users.iter().find(|u| u.id == id).cloned())
        }
    }

    #[test]
    fn creates_user_with_valid_data() {
        let use_case = CreateUser::new(Arc::new(InMemoryUserRepository::new()));
        let user = use_case
            .execute("alice".to_string(), "alice@example.com".to_string())
            .unwrap();

        assert_eq!(user.username, "alice");
        assert_eq!(user.email, "alice@example.com");
    }

    #[test]
    fn rejects_empty_username() {
        let use_case = CreateUser::new(Arc::new(InMemoryUserRepository::new()));
        let result = use_case.execute("".to_string(), "alice@example.com".to_string());

        assert_eq!(result, Err(CreateUserError::Invalid(UserError::EmptyUsername)));
    }

    #[test]
    fn rejects_duplicate_email() {
        let use_case = CreateUser::new(Arc::new(InMemoryUserRepository::new()));
        use_case
            .execute("alice".to_string(), "alice@example.com".to_string())
            .unwrap();

        let result = use_case.execute("alice2".to_string(), "alice@example.com".to_string());

        assert_eq!(
            result,
            Err(CreateUserError::Repository(RepositoryError::Duplicate))
        );
    }
}
