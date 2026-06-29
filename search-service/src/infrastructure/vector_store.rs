use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::infrastructure::now_timestamp;

pub struct VectorStore {
    conn: Arc<Mutex<Connection>>,
}

#[derive(Debug, PartialEq)]
pub struct SimilarMatch {
    pub id: String,
    pub ext_id: String,
    pub content_type: String,
    pub distance: f64,
}

impl VectorStore {
    pub fn new(conn: Arc<Mutex<Connection>>) -> Self {
        Self { conn }
    }

    pub fn insert(
        &self,
        ext_id: &str,
        content_type: &str,
        model: &str,
        model_version: &str,
        vector: &[f32],
    ) -> rusqlite::Result<String> {
        let id = uuid::Uuid::new_v4().to_string();
        let conn = self.conn.lock().unwrap();

        conn.execute(
            "INSERT INTO embeddings (id, ext_id, content_type, model, model_version, dimension, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7)",
            params![
                id,
                ext_id,
                content_type,
                model,
                model_version,
                vector.len() as i64,
                now_timestamp()
            ],
        )?;

        let rowid = conn.last_insert_rowid();
        conn.execute(
            "INSERT INTO vec_items (rowid, embedding) VALUES (?1, ?2)",
            params![rowid, vector_to_literal(vector)],
        )?;

        Ok(id)
    }

    pub fn query_similar(&self, vector: &[f32], k: usize) -> rusqlite::Result<Vec<SimilarMatch>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = conn.prepare(
            "SELECT e.id, e.ext_id, e.content_type, v.distance
             FROM vec_items v
             JOIN embeddings e ON e.rowid = v.rowid
             WHERE v.embedding MATCH ?1 AND k = ?2
             ORDER BY v.distance",
        )?;

        let rows = stmt.query_map(params![vector_to_literal(vector), k as i64], |row| {
            Ok(SimilarMatch {
                id: row.get(0)?,
                ext_id: row.get(1)?,
                content_type: row.get(2)?,
                distance: row.get(3)?,
            })
        })?;

        rows.collect()
    }
}

fn vector_to_literal(vector: &[f32]) -> String {
    let joined = vector
        .iter()
        .map(|v| v.to_string())
        .collect::<Vec<_>>()
        .join(",");
    format!("[{joined}]")
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::infrastructure::db;

    fn store() -> VectorStore {
        let path = format!(
            "{}/vector_store_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        VectorStore::new(Arc::new(Mutex::new(conn)))
    }

    fn vector3(x: f32, y: f32, z: f32) -> Vec<f32> {
        let mut v = vec![0.0; super::super::EMBEDDING_DIMENSION];
        v[0] = x;
        v[1] = y;
        v[2] = z;
        v
    }

    #[test]
    fn finds_closest_vector_first() {
        let store = store();
        store
            .insert("post-a", "post", "stub", "v1", &vector3(1.0, 0.0, 0.0))
            .unwrap();
        store
            .insert("post-b", "post", "stub", "v1", &vector3(0.0, 1.0, 0.0))
            .unwrap();
        store
            .insert("post-c", "post", "stub", "v1", &vector3(0.9, 0.1, 0.0))
            .unwrap();

        let results = store
            .query_similar(&vector3(1.0, 0.0, 0.0), 2)
            .unwrap();

        assert_eq!(results.len(), 2);
        assert_eq!(results[0].ext_id, "post-a");
        assert_eq!(results[0].distance, 0.0);
        assert_eq!(results[1].ext_id, "post-c");
        assert!(results[1].distance > results[0].distance);
    }

    #[test]
    fn inserted_metadata_is_persisted() {
        let store = store();
        let id = store
            .insert("post-a", "post", "stub-hash-embedding", "v1", &vector3(1.0, 0.0, 0.0))
            .unwrap();

        let results = store.query_similar(&vector3(1.0, 0.0, 0.0), 1).unwrap();
        assert_eq!(results[0].id, id);
        assert_eq!(results[0].content_type, "post");
    }
}
