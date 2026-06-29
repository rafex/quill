use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use crate::domain::Comment;
use crate::infrastructure::compression::{self, ALGORITHM, LEVEL};
use crate::infrastructure::now_timestamp;
use crate::ports::{CommentRepository, RepositoryError};

pub const COMMENT_CREATED_TOPIC: &str = "forum.comment.created";

pub struct SqliteCommentRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteCommentRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl CommentRepository for SqliteCommentRepository {
    fn insert(&self, comment: &Comment) -> Result<(), RepositoryError> {
        let compressed = compression::compress(&comment.body);

        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.execute(
            "INSERT INTO comments (id, post_id, body, body_original_length, compression_algorithm, compression_level, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                comment.id,
                comment.post_id,
                compressed.data,
                compressed.original_length as i64,
                ALGORITHM,
                LEVEL,
                comment.created_at
            ],
        )
        .map_err(map_insert_error)?;

        let event_id = uuid::Uuid::new_v4().to_string();
        let payload = comment_created_payload(comment, &event_id);

        tx.execute(
            "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![event_id, COMMENT_CREATED_TOPIC, payload, now_timestamp()],
        )
        .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.commit().map_err(|e| RepositoryError::Unknown(e.to_string()))
    }

    fn find_by_id(&self, id: &str) -> Result<Option<Comment>, RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, post_id, body, created_at FROM comments WHERE id = ?1",
            params![id],
            |row| {
                let body: Vec<u8> = row.get(2)?;
                Ok(Comment {
                    id: row.get(0)?,
                    post_id: row.get(1)?,
                    body: compression::decompress(&body),
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| RepositoryError::Unknown(e.to_string()))
    }

    fn republish_all(&self) -> Result<usize, RepositoryError> {
        let mut conn = self.conn.lock().unwrap();
        let comments: Vec<Comment> = {
            let mut stmt = conn
                .prepare("SELECT id, post_id, body, created_at FROM comments")
                .map_err(|e| RepositoryError::Unknown(e.to_string()))?;
            let rows = stmt
                .query_map([], |row| {
                    let body: Vec<u8> = row.get(2)?;
                    Ok(Comment {
                        id: row.get(0)?,
                        post_id: row.get(1)?,
                        body: compression::decompress(&body),
                        created_at: row.get(3)?,
                    })
                })
                .map_err(|e| RepositoryError::Unknown(e.to_string()))?
                .collect::<Result<_, _>>()
                .map_err(|e| RepositoryError::Unknown(e.to_string()))?;
            rows
        };

        let tx = conn
            .transaction()
            .map_err(|e| RepositoryError::Unknown(e.to_string()))?;
        for comment in &comments {
            let event_id = uuid::Uuid::new_v4().to_string();
            let payload = comment_created_payload(comment, &event_id);
            tx.execute(
                "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
                 VALUES (?1, ?2, ?3, ?4, NULL)",
                params![event_id, COMMENT_CREATED_TOPIC, payload, now_timestamp()],
            )
            .map_err(|e| RepositoryError::Unknown(e.to_string()))?;
        }
        tx.commit().map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        Ok(comments.len())
    }
}

/// `event_id` identifies this specific emission (distinct from the
/// comment's own `id`) so a reindex emission isn't deduped away by the
/// consumer's inbox as if it were the original creation event.
fn comment_created_payload(comment: &Comment, event_id: &str) -> String {
    json!({
        "event_id": event_id,
        "id": comment.id,
        "post_id": comment.post_id,
        "body": comment.body,
        "created_at": comment.created_at,
    })
    .to_string()
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

    fn repository() -> (SqliteCommentRepository, String) {
        let path = format!(
            "{}/comment_repo_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        let post_id = seed_post(&conn);
        (SqliteCommentRepository::new(Arc::new(Mutex::new(conn))), post_id)
    }

    fn seed_post(conn: &Connection) -> String {
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

        let post = crate::domain::Post::new(
            topic.id,
            "Hello".to_string(),
            "hello-post".to_string(),
            "body".to_string(),
        )
        .unwrap();
        conn.execute(
            "INSERT INTO posts (id, topic_id, title, slug, body, body_original_length, compression_algorithm, compression_level, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                post.id,
                post.topic_id,
                post.title,
                post.slug,
                post.body.as_bytes(),
                post.body.len() as i64,
                "none",
                0,
                post.created_at
            ],
        )
        .unwrap();

        post.id
    }

    #[test]
    fn inserts_and_finds_comment_with_decompressed_body() {
        let (repo, post_id) = repository();
        let comment = Comment::new(post_id, "great post ".repeat(20)).unwrap();

        repo.insert(&comment).unwrap();

        assert_eq!(repo.find_by_id(&comment.id).unwrap(), Some(comment));
    }

    #[test]
    fn insert_writes_pending_comment_created_outbox_event() {
        let (repo, post_id) = repository();
        let comment = Comment::new(post_id, "nice".to_string()).unwrap();

        repo.insert(&comment).unwrap();

        let conn = repo.conn.lock().unwrap();
        let (topic, published_at): (String, Option<String>) = conn
            .query_row(
                "SELECT topic, published_at FROM outbox_events WHERE topic = ?1",
                params![COMMENT_CREATED_TOPIC],
                |row| Ok((row.get(0)?, row.get(1)?)),
            )
            .unwrap();

        assert_eq!(topic, COMMENT_CREATED_TOPIC);
        assert_eq!(published_at, None);
    }

    #[test]
    fn republish_all_writes_a_fresh_outbox_event_per_comment() {
        let (repo, post_id) = repository();
        let comment = Comment::new(post_id, "nice".to_string()).unwrap();
        repo.insert(&comment).unwrap();

        let republished = repo.republish_all().unwrap();
        assert_eq!(republished, 1);

        let conn = repo.conn.lock().unwrap();
        let distinct_event_ids: i64 = conn
            .query_row(
                "SELECT count(DISTINCT id) FROM outbox_events WHERE topic = ?1",
                params![COMMENT_CREATED_TOPIC],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(distinct_event_ids, 2);
    }
}
