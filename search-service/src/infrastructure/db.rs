use rusqlite::ffi::sqlite3_auto_extension;
use rusqlite::Connection;

use crate::infrastructure::EMBEDDING_DIMENSION;

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    register_vec_extension();
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    Ok(conn)
}

fn register_vec_extension() {
    unsafe {
        sqlite3_auto_extension(Some(std::mem::transmute(
            sqlite_vec::sqlite3_vec_init as *const (),
        )));
    }
}

fn apply_pragmas(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "PRAGMA journal_mode=WAL;
         PRAGMA synchronous=NORMAL;
         PRAGMA foreign_keys=ON;
         PRAGMA busy_timeout=5000;
         PRAGMA wal_autocheckpoint=1000;",
    )
}

pub fn init_schema(conn: &Connection) -> rusqlite::Result<()> {
    conn.execute_batch(
        "CREATE VIRTUAL TABLE IF NOT EXISTS content_fts USING fts5(
            ext_id UNINDEXED,
            content_type UNINDEXED,
            title,
            body
         );

         CREATE TABLE IF NOT EXISTS embeddings (
            rowid INTEGER PRIMARY KEY,
            id TEXT NOT NULL UNIQUE,
            ext_id TEXT NOT NULL,
            content_type TEXT NOT NULL,
            model TEXT NOT NULL,
            model_version TEXT NOT NULL,
            dimension INTEGER NOT NULL,
            created_at TEXT NOT NULL
         );

         CREATE INDEX IF NOT EXISTS idx_embeddings_ext_id ON embeddings(ext_id);

         CREATE TABLE IF NOT EXISTS inbox_messages (
            message_id TEXT PRIMARY KEY,
            topic TEXT NOT NULL,
            payload TEXT NOT NULL,
            received_at TEXT NOT NULL,
            processed_at TEXT
         );

         CREATE TABLE IF NOT EXISTS outbox_events (
            id TEXT PRIMARY KEY,
            topic TEXT NOT NULL,
            payload TEXT NOT NULL,
            created_at TEXT NOT NULL,
            published_at TEXT
         );",
    )?;

    conn.execute_batch(&format!(
        "CREATE VIRTUAL TABLE IF NOT EXISTS vec_items USING vec0(embedding float[{EMBEDDING_DIMENSION}]);"
    ))
}
