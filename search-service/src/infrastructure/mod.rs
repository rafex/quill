pub mod db;
pub mod inbox_worker;
pub mod mqtt;
pub mod vector_store;

use std::time::{SystemTime, UNIX_EPOCH};

/// Matches sentence-transformers/all-MiniLM-L6-v2 so swapping the stub
/// embedding provider for the real ONNX one (TASK-SEARCH-0006) doesn't
/// require changing the vec0 table or any downstream consumer.
pub const EMBEDDING_DIMENSION: usize = 384;

pub fn now_timestamp() -> String {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
        .to_string()
}
