use crate::domain::unix_timestamp;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Category {
    pub id: String,
    pub name: String,
    pub slug: String,
    pub created_at: String,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CategoryError {
    EmptyName,
    EmptySlug,
}

impl Category {
    pub fn new(name: String, slug: String) -> Result<Self, CategoryError> {
        if name.trim().is_empty() {
            return Err(CategoryError::EmptyName);
        }
        if slug.trim().is_empty() {
            return Err(CategoryError::EmptySlug);
        }

        Ok(Self {
            id: uuid::Uuid::new_v4().to_string(),
            name,
            slug,
            created_at: unix_timestamp(),
        })
    }
}
