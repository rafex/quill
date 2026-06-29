use crate::domain::unix_timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Topic {
    pub id: String,
    pub category_id: String,
    pub title: String,
    pub slug: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum TopicError {
    EmptyTitle,
    EmptySlug,
}

impl Topic {
    pub fn new(category_id: String, title: String, slug: String) -> Result<Self, TopicError> {
        if title.trim().is_empty() {
            return Err(TopicError::EmptyTitle);
        }
        if slug.trim().is_empty() {
            return Err(TopicError::EmptySlug);
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            category_id,
            title,
            slug,
            created_at: unix_timestamp(),
        })
    }
}
