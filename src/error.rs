use axum::{http::StatusCode, response::{IntoResponse, Response}, Json};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("LiveKit API error: {0}")]
    LiveKit(String),
    #[error("Not found: {0}")]
    NotFound(String),
    #[error("Bad request: {0}")]
    BadRequest(String),
    #[error("Internal error: {0}")]
    Internal(#[from] anyhow::Error),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, message) = match &self {
            AppError::NotFound(m)    => (StatusCode::NOT_FOUND, m.clone()),
            AppError::BadRequest(m)  => (StatusCode::BAD_REQUEST, m.clone()),
            AppError::LiveKit(m)     => (StatusCode::BAD_GATEWAY, m.clone()),
            AppError::Internal(e)    => (StatusCode::INTERNAL_SERVER_ERROR, e.to_string()),
        };
        (status, Json(json!({ "error": message }))).into_response()
    }
}

pub type AppResult<T> = Result<T, AppError>;