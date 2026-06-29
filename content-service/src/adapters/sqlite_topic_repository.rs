use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};

use crate::domain::Topic;
use crate::ports::{RepositoryError, TopicRepository};

pub struct SqliteTopicRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteTopicRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl TopicRepository for SqliteTopicRepository {
    fn insert(&self, topic: &Topic) -> Result<(), RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO topics (id, category_id, title, slug, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![topic.id, topic.category_id, topic.title, topic.slug, topic.created_at],
        )
        .map(|_| ())
        .map_err(map_insert_error)
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Topic>, RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, category_id, title, slug, created_at FROM topics WHERE id = ?1",
            params![id],
            |row| {
                Ok(Topic {
                    id: row.get(0)?,
                    category_id: row.get(1)?,
                    title: row.get(2)?,
                    slug: row.get(3)?,
                    created_at: row.get(4)?,
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
    use crate::adapters::SqliteCategoryRepository;
    use crate::infrastructure::db;
    use crate::ports::CategoryRepository;
    use crate::domain::Category;

    fn repository() -> (SqliteTopicRepository, Arc<Mutex<Connection>>) {
        let path = format!(
            "{}/topic_repo_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        let conn = Arc::new(Mutex::new(conn));
        (SqliteTopicRepository::new(conn.clone()), conn)
    }

    #[test]
    fn inserts_and_finds_topic_by_id() {
        let (repo, conn) = repository();
        let category_repo = SqliteCategoryRepository::new(conn);
        let category = Category::new("General".to_string(), "general".to_string()).unwrap();
        category_repo.insert(&category).unwrap();

        let topic = Topic::new(category.id, "Hello".to_string(), "hello".to_string()).unwrap();
        repo.insert(&topic).unwrap();

        assert_eq!(repo.find_by_id(&topic.id).unwrap(), Some(topic));
    }
}
