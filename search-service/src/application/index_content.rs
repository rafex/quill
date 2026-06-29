use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::infrastructure::vector_store::VectorStore;
use crate::ports::EmbeddingProvider;

pub struct IndexContent {
    conn: Arc<Mutex<Connection>>,
    vector_store: VectorStore,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

#[derive(Debug)]
pub enum IndexContentError {
    Sqlite(String),
    UnexpectedEmbeddingDimension { expected: usize, actual: usize },
}

impl std::fmt::Display for IndexContentError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Sqlite(message) => write!(f, "sqlite error: {message}"),
            Self::UnexpectedEmbeddingDimension { expected, actual } => write!(
                f,
                "embedding provider returned {actual} dimensions, expected {expected}"
            ),
        }
    }
}

impl IndexContent {
    pub fn new(conn: Arc<Mutex<Connection>>, embedding_provider: Arc<dyn EmbeddingProvider>) -> Self {
        let vector_store = VectorStore::new(conn.clone());
        Self {
            conn,
            vector_store,
            embedding_provider,
        }
    }

    pub fn execute(
        &self,
        ext_id: &str,
        content_type: &str,
        title: &str,
        body: &str,
    ) -> Result<(), IndexContentError> {
        let embedding = self.embedding_provider.embed(body);
        let expected = self.embedding_provider.dimension();
        if embedding.len() != expected {
            return Err(IndexContentError::UnexpectedEmbeddingDimension {
                expected,
                actual: embedding.len(),
            });
        }

        self.vector_store
            .insert(
                ext_id,
                content_type,
                self.embedding_provider.model_name(),
                self.embedding_provider.model_version(),
                &embedding,
            )
            .map_err(|e| IndexContentError::Sqlite(e.to_string()))?;

        let conn = self.conn.lock().unwrap();
        conn.execute(
            "INSERT INTO content_fts (ext_id, content_type, title, body) VALUES (?1, ?2, ?3, ?4)",
            params![ext_id, content_type, title, body],
        )
        .map_err(|e| IndexContentError::Sqlite(e.to_string()))?;

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::StubEmbeddingProvider;
    use crate::infrastructure::db;

    fn index_content() -> IndexContent {
        let path = format!(
            "{}/index_content_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        let conn = Arc::new(Mutex::new(conn));
        IndexContent::new(conn, Arc::new(StubEmbeddingProvider))
    }

    #[test]
    fn indexes_content_into_fts_and_vector_store() {
        let use_case = index_content();
        use_case
            .execute("post-1", "post", "Hello", "hello world this is a post about rust")
            .unwrap();

        let conn = use_case.conn.lock().unwrap();
        let fts_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM content_fts WHERE ext_id = 'post-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(fts_count, 1);

        let embedding_count: i64 = conn
            .query_row(
                "SELECT count(*) FROM embeddings WHERE ext_id = 'post-1'",
                [],
                |row| row.get(0),
            )
            .unwrap();
        assert_eq!(embedding_count, 1);
    }
}
