use std::sync::Mutex;

use rusqlite::{params, Connection, OptionalExtension};

use crate::infrastructure::now_timestamp;

pub fn process_message<F>(
    conn: &Mutex<Connection>,
    message_id: &str,
    topic: &str,
    payload: &str,
    handler: F,
) -> Result<bool, String>
where
    F: FnOnce(&str) -> Result<(), String>,
{
    {
        let conn = conn.lock().unwrap();
        let already_processed: Option<Option<String>> = conn
            .query_row(
                "SELECT processed_at FROM inbox_messages WHERE message_id = ?1",
                params![message_id],
                |row| row.get(0),
            )
            .optional()
            .map_err(|e| e.to_string())?;

        match already_processed {
            Some(Some(_)) => return Ok(false),
            Some(None) => {}
            None => {
                conn.execute(
                    "INSERT INTO inbox_messages (message_id, topic, payload, received_at, processed_at)
                     VALUES (?1, ?2, ?3, ?4, NULL)",
                    params![message_id, topic, payload, now_timestamp()],
                )
                .map_err(|e| e.to_string())?;
            }
        }
    }

    handler(payload)?;

    let conn = conn.lock().unwrap();
    conn.execute(
        "UPDATE inbox_messages SET processed_at = ?1 WHERE message_id = ?2",
        params![now_timestamp(), message_id],
    )
    .map_err(|e| e.to_string())?;

    Ok(true)
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;
    use std::sync::atomic::{AtomicUsize, Ordering};

    fn connection() -> Mutex<Connection> {
        let path = format!(
            "{}/inbox_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        Mutex::new(conn)
    }

    #[test]
    fn processes_a_new_message() {
        let conn = connection();
        let processed = process_message(&conn, "msg-1", "forum.test", "{}", |_| Ok(()));

        assert_eq!(processed, Ok(true));
    }

    #[test]
    fn duplicate_message_is_not_reprocessed() {
        let conn = connection();
        let calls = AtomicUsize::new(0);

        for _ in 0..2 {
            let result = process_message(&conn, "msg-1", "forum.test", "{}", |_| {
                calls.fetch_add(1, Ordering::SeqCst);
                Ok(())
            });
            result.unwrap();
        }

        assert_eq!(calls.load(Ordering::SeqCst), 1);
    }
}
