use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection, OptionalExtension};
use serde_json::json;

use crate::domain::User;
use crate::infrastructure::now_timestamp;
use crate::ports::{RepositoryError, UserRepository};

pub const USER_CREATED_TOPIC: &str = "forum.user.created";

pub struct SqliteUserRepository {
    conn: Arc<Mutex<Connection>>,
}

impl SqliteUserRepository {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }
}

impl UserRepository for SqliteUserRepository {
    fn insert(&self, user: &User) -> Result<(), RepositoryError> {
        let mut conn = self.conn.lock().unwrap();
        let tx = conn
            .transaction()
            .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.execute(
            "INSERT INTO users (id, username, email, created_at) VALUES (?1, ?2, ?3, ?4)",
            params![user.id, user.username, user.email, user.created_at],
        )
        .map_err(map_insert_error)?;

        let payload = json!({
            "id": user.id,
            "username": user.username,
            "email": user.email,
            "created_at": user.created_at,
        })
        .to_string();

        tx.execute(
            "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
             VALUES (?1, ?2, ?3, ?4, NULL)",
            params![
                uuid::Uuid::new_v4().to_string(),
                USER_CREATED_TOPIC,
                payload,
                now_timestamp()
            ],
        )
        .map_err(|e| RepositoryError::Unknown(e.to_string()))?;

        tx.commit().map_err(|e| RepositoryError::Unknown(e.to_string()))
    }

    fn find_by_id(&self, id: &str) -> Result<Option<User>, RepositoryError> {
        let conn = self.conn.lock().unwrap();
        conn.query_row(
            "SELECT id, username, email, created_at FROM users WHERE id = ?1",
            params![id],
            |row| {
                Ok(User {
                    id: row.get(0)?,
                    username: row.get(1)?,
                    email: row.get(2)?,
                    created_at: row.get(3)?,
                })
            },
        )
        .optional()
        .map_err(|e| RepositoryError::Unknown(e.to_string()))
    }
}

fn map_insert_error(error: rusqlite::Error) -> RepositoryError {
    match &error {
        rusqlite::Error::SqliteFailure(e, _)
            if e.code == rusqlite::ErrorCode::ConstraintViolation =>
        {
            RepositoryError::Duplicate
        }
        _ => RepositoryError::Unknown(error.to_string()),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;

    fn repository() -> SqliteUserRepository {
        let path = format!(
            "{}/users_repo_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        SqliteUserRepository::new(Arc::new(Mutex::new(conn)))
    }

    #[test]
    fn inserts_and_finds_user_by_id() {
        let repo = repository();
        let user = User::new("alice".to_string(), "alice@example.com".to_string()).unwrap();

        repo.insert(&user).unwrap();
        let found = repo.find_by_id(&user.id).unwrap();

        assert_eq!(found, Some(user));
    }

    #[test]
    fn find_by_id_returns_none_when_missing() {
        let repo = repository();
        assert_eq!(repo.find_by_id("missing").unwrap(), None);
    }

    #[test]
    fn insert_duplicate_email_returns_duplicate_error() {
        let repo = repository();
        let user_a = User::new("alice".to_string(), "alice@example.com".to_string()).unwrap();
        let user_b = User::new("alice2".to_string(), "alice@example.com".to_string()).unwrap();

        repo.insert(&user_a).unwrap();
        let result = repo.insert(&user_b);

        assert_eq!(result, Err(RepositoryError::Duplicate));
    }

    #[test]
    fn insert_writes_pending_outbox_event_in_same_transaction() {
        let repo = repository();
        let user = User::new("alice".to_string(), "alice@example.com".to_string()).unwrap();

        repo.insert(&user).unwrap();

        let conn = repo.conn.lock().unwrap();
        let (topic, payload, published_at): (String, String, Option<String>) = conn
            .query_row(
                "SELECT topic, payload, published_at FROM outbox_events WHERE topic = ?1",
                params![USER_CREATED_TOPIC],
                |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)),
            )
            .unwrap();

        assert_eq!(topic, USER_CREATED_TOPIC);
        assert!(payload.contains(&user.id));
        assert_eq!(published_at, None);
    }
}
