use std::sync::Arc;

use axum::extract::{Query, State};
use axum::http::StatusCode;
use axum::routing::get;
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::application::{HybridSearch, SearchResult};
use crate::transport::health;

#[derive(Clone)]
pub struct AppState {
    pub search: Arc<HybridSearch>,
}

#[derive(Deserialize)]
struct SearchQuery {
    q: String,
    limit: Option<usize>,
}

#[derive(Serialize)]
struct SearchResultResponse {
    id: String,
    #[serde(rename = "type")]
    content_type: String,
    title: String,
    snippet: String,
    score: f64,
}

impl From<SearchResult> for SearchResultResponse {
    fn from(r: SearchResult) -> Self {
        Self {
            id: r.id,
            content_type: r.content_type,
            title: r.title,
            snippet: r.snippet,
            score: r.score,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/search", get(search))
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(state)
}

async fn search(
    State(state): State<AppState>,
    Query(query): Query<SearchQuery>,
) -> (StatusCode, Json<serde_json::Value>) {
    let limit = query.limit.unwrap_or(10);

    match state.search.search(&query.q, limit) {
        Ok(results) => {
            let body: Vec<SearchResultResponse> =
                results.into_iter().map(SearchResultResponse::from).collect();
            (StatusCode::OK, Json(serde_json::to_value(body).unwrap()))
        }
        Err(error) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse { error: error.to_string() }).unwrap()),
        ),
    }
}
