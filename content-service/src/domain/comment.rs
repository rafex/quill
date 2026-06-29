use crate::domain::unix_timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Comment {
    pub id: String,
    pub post_id: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CommentError {
    EmptyBody,
}

impl Comment {
    pub fn new(post_id: String, body: String) -> Result<Self, CommentError> {
        if body.trim().is_empty() {
            return Err(CommentError::EmptyBody);
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            post_id,
            body,
            created_at: unix_timestamp(),
        })
    }
}
