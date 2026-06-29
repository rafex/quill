use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use crate::domain::Post;
use crate::infrastructure::compression::{self, ALGORITHM, LEVEL};
use crate::infrastructure::now_timestamp;
use crate::ports::{PostRepository, RepositoryError};

pub const POST_CREATED_TOPIC: &str = "forum.post.created";

pub struct SqlitePostRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqlitePostRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl PostRepository for SqlitePostRepository {
    fn insert(&self, post: &Post) -> Result<(), RepositoryError> {
        let compressed = compression::compress(&post.body);

        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.execute(
            "INSERT INTO posts (id, topic_id, title, slug, body, body_original_length, compression_algorithm, compression_level, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                post.id,
                post.topic_id,
                post.title,
                post.slug,
                compressed.data,
                compressed.original_length as i64,
                ALGORITHM,
                LEVEL,
                post.created_at
            ],
        )
        .map_err(map_insert_error)?;

        let payload = json!({
            "id": post.id,
            "topic_id": post.topic_id,
            "title": post.title,
            "slug": post.slug,
            "body": post.body,
            "created_at": post.created_at,
        })
        .to_string();

        tx.execute(
            "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![
                uuid::Uuid::new_v4().to_string(),
                POST_CREATED_TOPIC,
                payload,
                now_timestamp()
            ],
        )
        .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.commit().map_err(|e| RepositoryError::Unknown(e.to_string()))
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Post>, RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, topic_id, title, slug, body, created_at FROM posts WHERE id = ?1",
            params![id],
            |row| {
                let body: Vec<u8> = row.get(4)?;
                Ok(Post {
                    id: row.get(0)?,
                    topic_id: row.get(1)?,
                    title: row.get(2)?,
                    slug: row.get(3)?,
                    body: compression::decompress(&body),
                    created_at: row.get(5)?,
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

    fn repository() -> (SqlitePostRepository, String) {
        let path = format!(
            "{}/post_repo_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        let topic_id = seed_topic(&conn);
        (SqlitePostRepository::new(Arc::new(Mutex::new(conn))), topic_id)
    }

    fn seed_topic(conn: &Connection) -> String {
        let category = crate::domain::Category::new("General".to_string(), "general".to_string())
            .unwrap();
        conn.execute(
            "INSERT INTO categories (id, name, slug, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![category.id, category.name, category.slug, category.created_at],
        )
        .unwrap();

        let topic =
            crate::domain::Topic::new(category.id, "Hello".to_string(), "hello-topic".to_string())
                .unwrap();
        conn.execute(
            "INSERT INTO topics (id, category_id, title, slug, created_at) VALUES (?1, ?2, ?3, ?4, ?5)",
            params![topic.id, topic.category_id, topic.title, topic.slug, topic.created_at],
        )
        .unwrap();

        topic.id
    }

    #[test]
    fn inserts_and_finds_post_with_decompressed_body() {
        let (repo, topic_id) = repository();
        let post = Post::new(
            topic_id,
            "Hello".to_string(),
            "hello".to_string(),
            "this is the post body ".repeat(20),
        )
        .unwrap();

        repo.insert(&post).unwrap();
        let found = repo.find_by_id(&post.id).unwrap();

        assert_eq!(found, Some(post));
    }

    #[test]
    fn insert_writes_pending_post_created_outbox_event() {
        let (repo, topic_id) = repository();
        let post = Post::new(
            topic_id,
            "Hello".to_string(),
            "hello".to_string(),
            "body".to_string(),
        )
        .unwrap();

        repo.insert(&post).unwrap();

        let conn = repo.conn.lock().unwrap();
        let (topic, payload, published_at): (String, String, Option<String>) = conn
            .query_row(
                "SELECT topic, payload, published_at FROM outbox_events WHERE topic = ?1",
                params![POST_CREATED_TOPIC],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(topic, POST_CREATED_TOPIC);
        assert!(payload.contains(&post.id));
        assert_eq!(published_at, None);
    }

    #[test]
    fn body_is_actually_compressed_on_disk() {
        let (repo, topic_id) = repository();
        let body = "x".repeat(2000);
        let post = Post::new(
            topic_id,
            "Hello".to_string(),
            "hello".to_string(),
            body.clone(),
        )
        .unwrap();

        repo.insert(&post).unwrap();

        let conn = repo.conn.lock().unwrap();
        let (stored_len, original_len): (i64, i64) = conn
            .query_row(
                "SELECT length(body), body_original_length FROM posts WHERE id = ?1",
                params![post.id],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(original_len as usize, body.len());
        assert!((stored_len as usize) < body.len());
    }
}
