use std::collections::HashMap;
use std::sync::{Arc, Mutex};

use rusqlite::{params, Connection};

use crate::infrastructure::vector_store::VectorStore;
use crate::ports::EmbeddingProvider;

pub const VECTOR_WEIGHT: f64 = 0.60;
pub const BM25_WEIGHT: f64 = 0.40;

const CANDIDATE_POOL_SIZE: usize = 20;
const SNIPPET_LENGTH: usize = 160;

#[derive(Debug, PartialEq)]
pub struct SearchResult {
    pub id: String,
    pub content_type: String,
    pub title: String,
    pub snippet: String,
    pub score: f64,
}

struct Candidate {
    content_type: String,
    vector_score: f64,
    fts_score: f64,
}

pub struct HybridSearch {
    conn: Arc<Mutex<Connection>>,
    vector_store: VectorStore,
    embedding_provider: Arc<dyn EmbeddingProvider>,
}

impl HybridSearch {
    pub fn new(conn: Arc<Mutex<Connection>>, embedding_provider: Arc<dyn EmbeddingProvider>) -> Self {
        let vector_store = VectorStore::new(conn.clone());
        Self {
            conn,
            vector_store,
            embedding_provider,
        }
    }

    pub fn search(&self, query: &str, limit: usize) -> rusqlite::Result<Vec<SearchResult>> {
        let query_embedding = self.embedding_provider.embed(query);
        let mut candidates: HashMap<String, Candidate> = HashMap::new();

        for m in self
            .vector_store
            .query_similar(&query_embedding, CANDIDATE_POOL_SIZE)?
        {
            let vector_score = 1.0 / (1.0 + m.distance);
            candidates
                .entry(m.ext_id)
                .or_insert_with(|| Candidate {
                    content_type: m.content_type,
                    vector_score: 0.0,
                    fts_score: 0.0,
                })
                .vector_score = vector_score;
        }

        for (ext_id, content_type, fts_score) in self.fts_candidates(query)? {
            candidates
                .entry(ext_id)
                .or_insert_with(|| Candidate {
                    content_type,
                    vector_score: 0.0,
                    fts_score: 0.0,
                })
                .fts_score = fts_score;
        }

        let mut scored: Vec<(String, String, f64)> = candidates
            .into_iter()
            .map(|(ext_id, c)| {
                let score = combine_scores(c.vector_score, c.fts_score);
                (ext_id, c.content_type, score)
            })
            .collect();
        scored.sort_by(|a, b| b.2.partial_cmp(&a.2).unwrap());
        scored.truncate(limit);

        scored
            .into_iter()
            .map(|(ext_id, content_type, score)| self.to_search_result(ext_id, content_type, score))
            .collect()
    }

    fn fts_candidates(&self, query: &str) -> rusqlite::Result<Vec<(String, String, f64)>> {
        let conn = self.conn.lock().unwrap();
        let mut stmt = match conn.prepare(
            "SELECT ext_id, content_type, bm25(content_fts) FROM content_fts
             WHERE content_fts MATCH ?1
             ORDER BY bm25(content_fts)
             LIMIT ?2",
        ) {
            Ok(stmt) => stmt,
            Err(_) => return Ok(Vec::new()),
        };

        let rows = stmt.query_map(params![query, CANDIDATE_POOL_SIZE as i64], |row| {
            let ext_id: String = row.get(0)?;
            let content_type: String = row.get(1)?;
            let raw_bm25: f64 = row.get(2)?;
            let positive = -raw_bm25;
            Ok((ext_id, content_type, positive / (1.0 + positive)))
        });

        match rows {
            Ok(rows) => rows.collect(),
            // a malformed FTS5 query syntax shouldn't fail the whole hybrid
            // search - fall back to vector-only results.
            Err(_) => Ok(Vec::new()),
        }
    }

    fn to_search_result(
        &self,
        ext_id: String,
        content_type: String,
        score: f64,
    ) -> rusqlite::Result<SearchResult> {
        let conn = self.conn.lock().unwrap();
        let (title, body): (String, String) = conn.query_row(
            "SELECT title, body FROM content_fts WHERE ext_id = ?1 LIMIT 1",
            params![ext_id],
            |row| Ok((row.get(0)?, row.get(1)?)),
        )?;

        let snippet = body.chars().take(SNIPPET_LENGTH).collect();

        Ok(SearchResult {
            id: ext_id,
            content_type,
            title,
            snippet,
            score,
        })
    }
}

fn combine_scores(vector_score: f64, fts_score: f64) -> f64 {
    VECTOR_WEIGHT * vector_score + BM25_WEIGHT * fts_score
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::adapters::StubEmbeddingProvider;
    use crate::application::IndexContent;
    use crate::infrastructure::db;

    #[test]
    fn combine_scores_weighs_vector_and_bm25() {
        assert_eq!(combine_scores(1.0, 0.0), VECTOR_WEIGHT);
        assert_eq!(combine_scores(0.0, 1.0), BM25_WEIGHT);
        assert_eq!(combine_scores(1.0, 1.0), 1.0);
    }

    fn setup() -> (HybridSearch, Arc<Mutex<Connection>>) {
        let path = format!(
            "{}/hybrid_search_test_{}.sqlite",
            std::env::temp_dir().display(),
            uuid::Uuid::new_v4()
        );
        let conn = db::open(&path).expect("open db");
        db::init_schema(&conn).expect("init schema");
        let conn = Arc::new(Mutex::new(conn));
        let provider: Arc<dyn EmbeddingProvider> = Arc::new(StubEmbeddingProvider);
        (HybridSearch::new(conn.clone(), provider), conn)
    }

    #[test]
    fn finds_indexed_content_by_keyword() {
        let (search, conn) = setup();
        let indexer = IndexContent::new(conn, Arc::new(StubEmbeddingProvider));
        indexer
            .execute("post-1", "post", "Rust and SQLite", "rust and sqlite make a great pair for embedded storage")
            .unwrap();
        indexer
            .execute("post-2", "post", "Cooking pasta", "boil water and add salt before the pasta")
            .unwrap();

        let results = search.search("rust sqlite", 5).unwrap();

        assert!(!results.is_empty());
        assert_eq!(results[0].id, "post-1");
        assert!(results[0].score > 0.0);
    }
}
