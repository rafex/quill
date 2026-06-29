use rusqlite::Connection;

pub fn open(path: &str) -> rusqlite::Result<Connection> {
    let conn = Connection::open(path)?;
    apply_pragmas(&conn)?;
    Ok(conn)
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
        "CREATE TABLE IF NOT EXISTS categories (
            id TEXT PRIMARY KEY,
            name TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS topics (
            id TEXT PRIMARY KEY,
            category_id TEXT NOT NULL REFERENCES categories(id),
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS posts (
            id TEXT PRIMARY KEY,
            topic_id TEXT NOT NULL REFERENCES topics(id),
            title TEXT NOT NULL,
            slug TEXT NOT NULL UNIQUE,
            body BLOB NOT NULL,
            body_original_length INTEGER NOT NULL,
            compression_algorithm TEXT NOT NULL,
            compression_level INTEGER NOT NULL,
            created_at TEXT NOT NULL
         );

         CREATE TABLE IF NOT EXISTS comments (
            id TEXT PRIMARY KEY,
            post_id TEXT NOT NULL REFERENCES posts(id),
            body BLOB NOT NULL,
            body_original_length INTEGER NOT NULL,
            compression_algorithm TEXT NOT NULL,
            compression_level INTEGER NOT NULL,
            created_at TEXT NOT NULL
         );

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
    )
}
