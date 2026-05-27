use axum::{
    Json,
    http::StatusCode,
    response::{IntoResponse, Response},
};
use serde_json::json;

#[derive(thiserror::Error, Debug)]
pub enum AppError {
    #[error("Not found")]
    NotFound,

    #[error("Unauthorized")]
    Unauthorized,

    #[error("Conflict: {0}")]
    Conflict(String),

    #[error(transparent)]
    Database(#[from] sqlx::Error),

    #[error(transparent)]
    Template(#[from] askama::Error),

    #[error("{0}")]
    Internal(String),
}

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        let (status, msg) = match &self {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m.clone()),
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Template(e) => {
                tracing::error!("Template render error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Template render failed".to_string())
            }
            AppError::Internal(m) => {
                tracing::error!("Internal error: {}", m);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        (status, msg).into_response()
    }
}

impl AppError {
    /// Convert a database error, treating UNIQUE violations as Conflict.
    pub fn from_db_create(e: sqlx::Error, conflict_msg: impl Into<String>) -> Self {
        if e.to_string().contains("UNIQUE constraint failed") {
            AppError::Conflict(conflict_msg.into())
        } else {
            AppError::Database(e)
        }
    }
}

// ─── API error (returns JSON) ─────────────────────────────────────────────────

/// Wraps `AppError` but serialises the response body as JSON.
pub struct ApiError(pub AppError);

impl IntoResponse for ApiError {
    fn into_response(self) -> Response {
        let (status, msg) = match self.0 {
            AppError::NotFound => (StatusCode::NOT_FOUND, "Not found".to_string()),
            AppError::Unauthorized => (StatusCode::UNAUTHORIZED, "Unauthorized".to_string()),
            AppError::Conflict(m) => (StatusCode::CONFLICT, m),
            AppError::Database(e) => {
                tracing::error!("Database error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Database error".to_string())
            }
            AppError::Template(e) => {
                tracing::error!("Template error: {}", e);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal error".to_string())
            }
            AppError::Internal(m) => {
                tracing::error!("Internal error: {}", m);
                (StatusCode::INTERNAL_SERVER_ERROR, "Internal server error".to_string())
            }
        };
        (status, Json(json!({ "error": msg }))).into_response()
    }
}

impl From<AppError> for ApiError {
    fn from(e: AppError) -> Self { ApiError(e) }
}

impl From<sqlx::Error> for ApiError {
    fn from(e: sqlx::Error) -> Self { ApiError(AppError::Database(e)) }
}