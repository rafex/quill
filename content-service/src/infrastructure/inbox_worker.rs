use std::sync::Mutex;
use std::time::Duration;

use rusqlite::{params, Connection, OptionalExtension};

use crate::infrastructure::now_timestamp;

/// Fixed backoff between retries; the last delay repeats if more
/// attempts remain. Chosen to be short enough not to stall a single
/// worker process for long, not to model a "real" SLA.
pub const RETRY_BACKOFF: &[Duration] = &[
    Duration::from_millis(200),
    Duration::from_millis(500),
    Duration::from_millis(1000),
];

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

/// Retries the same message against `handler` using `RETRY_BACKOFF`
/// before giving up. Safe to retry: `process_message` only re-runs the
/// handler on a message already marked received-but-not-processed, it
/// never re-inserts or double-counts the inbox row.
pub fn process_with_retry<F>(
    conn: &Mutex<Connection>,
    message_id: &str,
    topic: &str,
    payload: &str,
    handler: F,
) -> Result<bool, String>
where
    F: Fn(&str) -> Result<(), String>,
{
    let mut attempt = 0;
    loop {
        let result = process_message(conn, message_id, topic, payload, &handler);
        match result {
            Err(error) if attempt < RETRY_BACKOFF.len() => {
                eprintln!(
                    "attempt {} for {message_id} failed: {error}; retrying in {:?}",
                    attempt + 1,
                    RETRY_BACKOFF[attempt]
                );
                std::thread::sleep(RETRY_BACKOFF[attempt]);
                attempt += 1;
            }
            other => return other,
        }
    }
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

    #[test]
    fn process_with_retry_succeeds_after_transient_failures() {
        let conn = connection();
        let calls = AtomicUsize::new(0);

        let result = process_with_retry(&conn, "msg-1", "forum.test", "{}", |_| {
            let attempt = calls.fetch_add(1, Ordering::SeqCst);
            if attempt < 2 {
                Err("transient failure".to_string())
            } else {
                Ok(())
            }
        });

        assert_eq!(result, Ok(true));
        assert_eq!(calls.load(Ordering::SeqCst), 3);
    }

    #[test]
    fn process_with_retry_gives_up_after_exhausting_backoff() {
        let conn = connection();
        let calls = AtomicUsize::new(0);

        let result = process_with_retry(&conn, "msg-1", "forum.test", "{}", |_| {
            calls.fetch_add(1, Ordering::SeqCst);
            Err("permanent failure".to_string())
        });

        assert_eq!(result, Err("permanent failure".to_string()));
        assert_eq!(calls.load(Ordering::SeqCst), RETRY_BACKOFF.len() + 1);
    }
}
