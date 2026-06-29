use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::application::{
    CreateCategory, CreateCategoryError, CreateComment, CreateCommentError, CreatePost,
    CreatePostError, CreateTopic, CreateTopicError,
};
use crate::domain::{Category, Comment, Post, Topic};
use crate::ports::{CategoryRepository, CommentRepository, PostRepository, RepositoryError, TopicRepository};
use crate::transport::health;

#[derive(Clone)]
pub struct AppState {
    pub categories: Arc<dyn CategoryRepository>,
    pub topics: Arc<dyn TopicRepository>,
    pub posts: Arc<dyn PostRepository>,
    pub comments: Arc<dyn CommentRepository>,
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

fn error_body(message: impl Into<String>) -> Json<serde_json::Value> {
    Json(serde_json::to_value(ErrorResponse { error: message.into() }).unwrap())
}

fn ok_body<T: Serialize>(value: T) -> Json<serde_json::Value> {
    Json(serde_json::to_value(value).unwrap())
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/categories", post(create_category))
        .route("/categories/:id", get(get_category))
        .route("/topics", post(create_topic))
        .route("/topics/:id", get(get_topic))
        .route("/posts", post(create_post))
        .route("/posts/:id", get(get_post))
        .route("/comments", post(create_comment))
        .route("/comments/:id", get(get_comment))
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(state)
}

#[derive(Serialize)]
struct CategoryResponse {
    id: String,
    name: String,
    slug: String,
    created_at: String,
}

impl From<Category> for CategoryResponse {
    fn from(c: Category) -> Self {
        Self { id: c.id, name: c.name, slug: c.slug, created_at: c.created_at }
    }
}

#[derive(Deserialize)]
struct CreateCategoryRequest {
    name: String,
    slug: String,
}

async fn create_category(
    State(state): State<AppState>,
    Json(body): Json<CreateCategoryRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = CreateCategory::new(state.categories);
    match use_case.execute(body.name, body.slug) {
        Ok(category) => (StatusCode::CREATED, ok_body(CategoryResponse::from(category))),
        Err(CreateCategoryError::Invalid(_)) => {
            (StatusCode::BAD_REQUEST, error_body("invalid category"))
        }
        Err(CreateCategoryError::Repository(RepositoryError::Duplicate)) => {
            (StatusCode::CONFLICT, error_body("slug already exists"))
        }
        Err(CreateCategoryError::Repository(RepositoryError::Unknown(m))) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error_body(m))
        }
    }
}

async fn get_category(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.categories.find_by_id(&id) {
        Ok(Some(category)) => (StatusCode::OK, ok_body(CategoryResponse::from(category))),
        Ok(None) => (StatusCode::NOT_FOUND, error_body("category not found")),
        Err(RepositoryError::Unknown(m)) => (StatusCode::INTERNAL_SERVER_ERROR, error_body(m)),
        Err(RepositoryError::Duplicate) => unreachable!("find_by_id never returns Duplicate"),
    }
}

#[derive(Serialize)]
struct TopicResponse {
    id: String,
    category_id: String,
    title: String,
    slug: String,
    created_at: String,
}

impl From<Topic> for TopicResponse {
    fn from(t: Topic) -> Self {
        Self { id: t.id, category_id: t.category_id, title: t.title, slug: t.slug, created_at: t.created_at }
    }
}

#[derive(Deserialize)]
struct CreateTopicRequest {
    category_id: String,
    title: String,
    slug: String,
}

async fn create_topic(
    State(state): State<AppState>,
    Json(body): Json<CreateTopicRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = CreateTopic::new(state.topics);
    match use_case.execute(body.category_id, body.title, body.slug) {
        Ok(topic) => (StatusCode::CREATED, ok_body(TopicResponse::from(topic))),
        Err(CreateTopicError::Invalid(_)) => (StatusCode::BAD_REQUEST, error_body("invalid topic")),
        Err(CreateTopicError::Repository(RepositoryError::Duplicate)) => {
            (StatusCode::CONFLICT, error_body("slug already exists"))
        }
        Err(CreateTopicError::Repository(RepositoryError::Unknown(m))) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error_body(m))
        }
    }
}

async fn get_topic(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.topics.find_by_id(&id) {
        Ok(Some(topic)) => (StatusCode::OK, ok_body(TopicResponse::from(topic))),
        Ok(None) => (StatusCode::NOT_FOUND, error_body("topic not found")),
        Err(RepositoryError::Unknown(m)) => (StatusCode::INTERNAL_SERVER_ERROR, error_body(m)),
        Err(RepositoryError::Duplicate) => unreachable!("find_by_id never returns Duplicate"),
    }
}

#[derive(Serialize)]
struct PostResponse {
    id: String,
    topic_id: String,
    title: String,
    slug: String,
    body: String,
    created_at: String,
}

impl From<Post> for PostResponse {
    fn from(p: Post) -> Self {
        Self { id: p.id, topic_id: p.topic_id, title: p.title, slug: p.slug, body: p.body, created_at: p.created_at }
    }
}

#[derive(Deserialize)]
struct CreatePostRequest {
    topic_id: String,
    title: String,
    slug: String,
    body: String,
}

async fn create_post(
    State(state): State<AppState>,
    Json(body): Json<CreatePostRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = CreatePost::new(state.posts);
    match use_case.execute(body.topic_id, body.title, body.slug, body.body) {
        Ok(post) => (StatusCode::CREATED, ok_body(PostResponse::from(post))),
        Err(CreatePostError::Invalid(_)) => (StatusCode::BAD_REQUEST, error_body("invalid post")),
        Err(CreatePostError::Repository(RepositoryError::Duplicate)) => {
            (StatusCode::CONFLICT, error_body("slug already exists"))
        }
        Err(CreatePostError::Repository(RepositoryError::Unknown(m))) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error_body(m))
        }
    }
}

async fn get_post(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.posts.find_by_id(&id) {
        Ok(Some(post)) => (StatusCode::OK, ok_body(PostResponse::from(post))),
        Ok(None) => (StatusCode::NOT_FOUND, error_body("post not found")),
        Err(RepositoryError::Unknown(m)) => (StatusCode::INTERNAL_SERVER_ERROR, error_body(m)),
        Err(RepositoryError::Duplicate) => unreachable!("find_by_id never returns Duplicate"),
    }
}

#[derive(Serialize)]
struct CommentResponse {
    id: String,
    post_id: String,
    body: String,
    created_at: String,
}

impl From<Comment> for CommentResponse {
    fn from(c: Comment) -> Self {
        Self { id: c.id, post_id: c.post_id, body: c.body, created_at: c.created_at }
    }
}

#[derive(Deserialize)]
struct CreateCommentRequest {
    post_id: String,
    body: String,
}

async fn create_comment(
    State(state): State<AppState>,
    Json(body): Json<CreateCommentRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = CreateComment::new(state.comments);
    match use_case.execute(body.post_id, body.body) {
        Ok(comment) => (StatusCode::CREATED, ok_body(CommentResponse::from(comment))),
        Err(CreateCommentError::Invalid(_)) => (StatusCode::BAD_REQUEST, error_body("invalid comment")),
        Err(CreateCommentError::Repository(RepositoryError::Duplicate)) => {
            (StatusCode::CONFLICT, error_body("duplicate comment"))
        }
        Err(CreateCommentError::Repository(RepositoryError::Unknown(m))) => {
            (StatusCode::INTERNAL_SERVER_ERROR, error_body(m))
        }
    }
}

async fn get_comment(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    match state.comments.find_by_id(&id) {
        Ok(Some(comment)) => (StatusCode::OK, ok_body(CommentResponse::from(comment))),
        Ok(None) => (StatusCode::NOT_FOUND, error_body("comment not found")),
        Err(RepositoryError::Unknown(m)) => (StatusCode::INTERNAL_SERVER_ERROR, error_body(m)),
        Err(RepositoryError::Duplicate) => unreachable!("find_by_id never returns Duplicate"),
    }
}
