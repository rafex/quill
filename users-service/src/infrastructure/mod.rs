pub mod db;
pub mod inbox_worker;
pub mod mqtt;
pub mod outbox_publisher;

use std::time::{SystemTime, UNIX_EPOCH};

pub fn now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
