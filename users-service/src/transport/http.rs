use std::sync::Arc;

use axum::extract::{Path, State};
use axum::http::StatusCode;
use axum::routing::{get, post};
use axum::{Json, Router};
use serde::{Deserialize, Serialize};

use crate::application::{CreateUser, CreateUserError, GetUserById};
use crate::domain::{User, UserError};
use crate::ports::{RepositoryError, UserRepository};
use crate::transport::health;

#[derive(Clone)]
pub struct AppState {
    pub repo: Arc<dyn UserRepository>,
}

#[derive(Deserialize)]
struct CreateUserRequest {
    username: String,
    email: String,
}

#[derive(Serialize)]
struct UserResponse {
    id: String,
    username: String,
    email: String,
    created_at: String,
}

impl From<User> for UserResponse {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
            created_at: user.created_at,
        }
    }
}

#[derive(Serialize)]
struct ErrorResponse {
    error: String,
}

pub fn router(state: AppState) -> Router {
    Router::new()
        .route("/users", post(create_user))
        .route("/users/:id", get(get_user))
        .route("/health", get(health::health))
        .route("/ready", get(health::ready))
        .with_state(state)
}

async fn create_user(
    State(state): State<AppState>,
    Json(body): Json<CreateUserRequest>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = CreateUser::new(state.repo);

    match use_case.execute(body.username, body.email) {
        Ok(user) => (
            StatusCode::CREATED,
            Json(serde_json::to_value(UserResponse::from(user)).unwrap()),
        ),
        Err(CreateUserError::Invalid(reason)) => (
            StatusCode::BAD_REQUEST,
            Json(serde_json::to_value(ErrorResponse {
                error: describe_user_error(reason),
            })
            .unwrap()),
        ),
        Err(CreateUserError::Repository(RepositoryError::Duplicate)) => (
            StatusCode::CONFLICT,
            Json(serde_json::to_value(ErrorResponse {
                error: "username or email already exists".to_string(),
            })
            .unwrap()),
        ),
        Err(CreateUserError::Repository(RepositoryError::Unknown(message))) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse { error: message }).unwrap()),
        ),
    }
}

async fn get_user(
    State(state): State<AppState>,
    Path(id): Path<String>,
) -> (StatusCode, Json<serde_json::Value>) {
    let use_case = GetUserById::new(state.repo);

    match use_case.execute(&id) {
        Ok(Some(user)) => (
            StatusCode::OK,
            Json(serde_json::to_value(UserResponse::from(user)).unwrap()),
        ),
        Ok(None) => (
            StatusCode::NOT_FOUND,
            Json(serde_json::to_value(ErrorResponse {
                error: "user not found".to_string(),
            })
            .unwrap()),
        ),
        Err(RepositoryError::Unknown(message)) => (
            StatusCode::INTERNAL_SERVER_ERROR,
            Json(serde_json::to_value(ErrorResponse { error: message }).unwrap()),
        ),
        Err(RepositoryError::Duplicate) => unreachable!("find_by_id never returns Duplicate"),
    }
}

fn describe_user_error(error: UserError) -> String {
    match error {
        UserError::EmptyUsername => "username must not be empty".to_string(),
        UserError::InvalidEmail => "email is invalid".to_string(),
    }
}
