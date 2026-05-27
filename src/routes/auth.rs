use axum::{
    Json, Router,
    extract::State,
    http::StatusCode,
    routing::post,
};
use serde::Serialize;

use crate::{
    auth::generate_token,
    db::DbPool,
    models::{
        User,
        user::{CreateUser, LoginUser},
    },
};

#[derive(Serialize)]
pub struct AuthResponse {
    token: String,
    user: User,
}

pub fn auth_routes() -> Router<DbPool> {
    Router::new()
        .route("/register", post(register))
        .route("/login", post(login))
}

async fn register(
    State(pool): State<DbPool>,
    Json(payload): Json<CreateUser>,
) -> Result<(StatusCode, Json<AuthResponse>), StatusCode> {
    let user = User::create(&pool, payload).await.map_err(|e| {
        if e.to_string().contains("UNIQUE constraint failed") {
            StatusCode::CONFLICT
        } else {
            tracing::error!("Registration failed: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        }
    })?;

    let token = generate_token(&user).map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok((StatusCode::CREATED, Json(AuthResponse { token, user })))
}

async fn login(
    State(pool): State<DbPool>,
    Json(payload): Json<LoginUser>,
) -> Result<Json<AuthResponse>, StatusCode> {
    let user = User::authenticate(&pool, payload)
        .await
        .map_err(|e| {
            tracing::error!("Authentication error: {}", e);
            StatusCode::INTERNAL_SERVER_ERROR
        })?
        .ok_or(StatusCode::UNAUTHORIZED)?;

    let token = generate_token(&user).map_err(|e| {
        tracing::error!("Token generation failed: {}", e);
        StatusCode::INTERNAL_SERVER_ERROR
    })?;

    Ok(Json(AuthResponse { token, user }))
}