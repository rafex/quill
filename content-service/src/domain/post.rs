use crate::domain::unix_timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Post {
    pub id: String,
    pub topic_id: String,
    pub title: String,
    pub slug: String,
    pub body: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum PostError {
    EmptyTitle,
    EmptySlug,
    EmptyBody,
}

impl Post {
    pub fn new(
        topic_id: String,
        title: String,
        slug: String,
        body: String,
    ) -> Result<Self, PostError> {
        if title.trim().is_empty() {
            return Err(PostError::EmptyTitle);
        }
        if slug.trim().is_empty() {
            return Err(PostError::EmptySlug);
        }
        if body.trim().is_empty() {
            return Err(PostError::EmptyBody);
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            topic_id,
            title,
            slug,
            body,
            created_at: unix_timestamp(),
        })
    }
}
