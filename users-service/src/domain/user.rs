use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct User {
    pub id: String,
    pub username: String,
    pub email: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum UserError {
    EmptyUsername,
    InvalidEmail,
}

impl User {
    pub fn new(username: String, email: String) -> Result<Self, UserError> {
        if username.trim().is_empty() {
            return Err(UserError::EmptyUsername);
        }
        if !email.contains('@') || email.trim().is_empty() {
            return Err(UserError::InvalidEmail);
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            username,
            email,
            created_at: unix_timestamp(),
        })
    }
}

fn unix_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
