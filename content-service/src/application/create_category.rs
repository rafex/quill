use std::sync::Arc;

use crate::domain::{Category, CategoryError};
use crate::ports::{CategoryRepository, RepositoryError};

pub struct CreateCategory {
    repo: Arc<dyn CategoryRepository>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum CreateCategoryError {
    Invalid(CategoryError),
    Repository(RepositoryError),
}

impl CreateCategory {
    pub fn new(repo: Arc<dyn CategoryRepository>) -> Self {
        Self { repo }
    }

    pub fn execute(&self, name: String, slug: String) -> Result<Category, CreateCategoryError> {
        let category = Category::new(name, slug).map_err(CreateCategoryError::Invalid)?;
        self.repo
            .insert(&category)
            .map_err(CreateCategoryError::Repository)?;
        Ok(category)
    }
}
