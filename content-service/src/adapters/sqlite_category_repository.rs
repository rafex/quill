use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::Category;
use crate::ports::{CategoryRepository, RepositoryError};

pub struct SqliteCategoryRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteCategoryRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl CategoryRepository for SqliteCategoryRepository {
    fn insert(&self, category: &Category) -> Result<(), RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO categories (id, name, slug, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![category.id, category.name, category.slug, category.created_at],
        )
        .map(|_| ())
        .map_err(map_insert_error)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Category>, RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, name, slug, created_at FROM categories WHERE id = ?1",
            params![id],
            |row| {
                Ok(Category {
                    id: row.get(0)?,
                    name: row.get(1)?,
                    slug: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| RepositoryError::Unknown(e.to_string()))
    }
}

fn map_insert_error(error: rusqlite::Error) -> RepositoryError {
    if let rusqlite::Error::SqliteFailure(e, _) = &error {
        if e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_UNIQUE
            || e.extended_code == rusqlite::ffi::SQLITE_CONSTRAINT_PRIMARYKEY
        {
            return RepositoryError::Duplicate;
        }
    }
    RepositoryError::Unknown(error.to_string())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;

    fn repository() -> SqliteCategoryRepository {
        let path = format!(
            "{}/category_repo_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        SqliteCategoryRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn inserts_and_finds_category_by_id() {
        let repo = repository();
        let category = Category::new("General".to_string(), "general".to_string()).unwrap();

        repo.insert(&category).unwrap();
        let found = repo.find_by_id(&category.id).unwrap();

        assert_eq!(found, Some(category));
    }

    #[test]
    fn insert_duplicate_slug_returns_duplicate_error() {
        let repo = repository();
        let a = Category::new("General".to_string(), "general".to_string()).unwrap();
        let b = Category::new("General 2".to_string(), "general".to_string()).unwrap();

        repo.insert(&a).unwrap();
        assert_eq!(repo.insert(&b), Err(RepositoryError::Duplicate));
    }
}
