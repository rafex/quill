use std::sync::Mutex;

use rusqlite::{params, Connection};

use crate::infrastructure::now_timestamp;
use crate::ports::EventPublisher;

pub fn publish_pending(conn: &Mutex<Connection>, publisher: &dyn EventPublisher) -> usize {
    let pending: Vec<(String, String, String)> = {
        let conn = conn.lock().unwrap();
        let mut stmt = conn
            .prepare("SELECT id, topic, payload FROM outbox_events WHERE published_at IS NULL")
            .expect("prepare outbox query");
        stmt.query_map([], |row| Ok((row.get(0)?, row.get(1)?, row.get(2)?)))
            .expect("query outbox rows")
            .collect::<Result<_, _>>()
            .expect("read outbox rows")
    };

    let mut published = 0;
    for (id, topic, payload) in pending {
        if publisher.publish(&topic, &payload).is_ok() {
            let conn = conn.lock().unwrap();
            conn.execute(
                "UPDATE outbox_events SET published_at = ?1 WHERE id = ?2",
                params![now_timestamp(), id],
            )
            .expect("mark outbox event published");
            published += 1;
        }
    }

    published
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;
    use crate::ports::PublishError;
    use std::sync::Mutex as StdMutex;

    struct RecordingPublisher {
        sent: StdMutex<Vec<(String, String)>>,
    }

    impl RecordingPublisher {
        fn new() -> Self {
            Self {
                sent: StdMutex::new(Vec::new()),
            }
        }
    }

    impl EventPublisher for RecordingPublisher {
        fn publish(&self, topic: &str, payload: &str) -> Result<(), PublishError> {
            self.sent
                .lock()
                .unwrap()
                .push((topic.to_string(), payload.to_string()));
            Ok(())
        }
    }

    fn connection() -> Mutex<Connection> {
        let path = format!(
            "{}/outbox_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        Mutex::new(conn)
    }

    #[test]
    fn publishes_pending_events_and_marks_them_published() {
        let conn = connection();
        {
            let guard = conn.lock().unwrap();
            guard
                .execute(
                    "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
                     VALUES ('evt-1', 'forum.user.created', '{}', '0', NULL)",
                    [],
                )
                .unwrap();
        }

        let publisher = RecordingPublisher::new();
        let published = publish_pending(&conn, &publisher);

        assert_eq!(published, 1);
        assert_eq!(publisher.sent.lock().unwrap().len(), 1);

        let guard = conn.lock().unwrap();
        let published_at: Option<String> = guard
            .query_row(
                "SELECT published_at FROM outbox_events WHERE id = 'evt-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert!(published_at.is_some());
    }

    #[test]
    fn does_not_republish_already_published_events() {
        let conn = connection();
        {
            let guard = conn.lock().unwrap();
            guard
                .execute(
                    "INSERT INTO outbox_events (id, topic, payload, created_at, published_at)
                     VALUES ('evt-1', 'forum.user.created', '{}', '0', '0')",
                    [],
                )
                .unwrap();
        }

        let publisher = RecordingPublisher::new();
        let published = publish_pending(&conn, &publisher);

        assert_eq!(published, 0);
        assert!(publisher.sent.lock().unwrap().is_empty());
    }
}
