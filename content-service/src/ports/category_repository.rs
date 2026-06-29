use crate::domain::Category;
use crate::ports::RepositoryError;

pub trait CategoryRepository: Send + Sync {
    fn insert(&self, category: &Category) -> Result<(), RepositoryError>;
    fn find_by_id(&self, id: &str) -> Result<Option<Category>, RepositoryError>;
}
