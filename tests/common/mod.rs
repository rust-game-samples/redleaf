#![allow(dead_code)]

use axum::{body::Body, http::Request, Router};
use http_body_util::BodyExt;
use serde_json::{json, Value};
use sqlx::sqlite::SqlitePoolOptions;
use tower::ServiceExt;

// ─── App setup ───────────────────────────────────────────────────────────────

pub async fn setup_app() -> Router {
    let pool = SqlitePoolOptions::new()
        .max_connections(1)
        .connect("sqlite::memory:")
        .await
        .expect("in-memory SQLite failed");

    sqlx::migrate!("./migrations")
        .run(&pool)
        .await
        .expect("migration failed");

    redleaf::build_app(pool)
}

// ─── Request helpers ─────────────────────────────────────────────────────────

pub async fn get(app: &Router, uri: &str, token: Option<&str>) -> (u16, String) {
    let mut req = Request::builder().method("GET").uri(uri);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    send(app, req.body(Body::empty()).unwrap()).await
}

pub async fn post_json(app: &Router, uri: &str, body: &Value, token: Option<&str>) -> (u16, Value) {
    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let (status, body) = send(app, req.body(Body::from(body.to_string())).unwrap()).await;
    let json = serde_json::from_str(&body).unwrap_or(Value::Null);
    (status, json)
}

pub async fn put_json(app: &Router, uri: &str, body: &Value, token: Option<&str>) -> (u16, Value) {
    let mut req = Request::builder()
        .method("PUT")
        .uri(uri)
        .header("Content-Type", "application/json");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    let (status, body) = send(app, req.body(Body::from(body.to_string())).unwrap()).await;
    let json = serde_json::from_str(&body).unwrap_or(Value::Null);
    (status, json)
}

pub async fn delete_req(app: &Router, uri: &str, token: Option<&str>) -> (u16, String) {
    let mut req = Request::builder().method("DELETE").uri(uri);
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    send(app, req.body(Body::empty()).unwrap()).await
}

pub async fn post_form(app: &Router, uri: &str, fields: &[(&str, &str)], token: Option<&str>) -> (u16, String) {
    let encoded = fields
        .iter()
        .map(|(k, v)| format!("{}={}", urlenccode(k), urlenccode(v)))
        .collect::<Vec<_>>()
        .join("&");

    let mut req = Request::builder()
        .method("POST")
        .uri(uri)
        .header("Content-Type", "application/x-www-form-urlencoded");
    if let Some(t) = token {
        req = req.header("Authorization", format!("Bearer {t}"));
    }
    send(app, req.body(Body::from(encoded)).unwrap()).await
}

async fn send(app: &Router, req: Request<Body>) -> (u16, String) {
    let response = app.clone().oneshot(req).await.unwrap();
    let status = response.status().as_u16();
    let bytes = response.into_body().collect().await.unwrap().to_bytes();
    let body = String::from_utf8_lossy(&bytes).into_owned();
    (status, body)
}

fn urlenccode(s: &str) -> String {
    s.chars()
        .flat_map(|c| match c {
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => {
                vec![c]
            }
            ' ' => vec!['+'],
            c => format!("%{:02X}", c as u32).chars().collect(),
        })
        .collect()
}

// ─── Auth helpers ─────────────────────────────────────────────────────────────

pub async fn register(app: &Router, username: &str, email: &str, password: &str) -> (u16, Value) {
    post_json(
        app,
        "/auth/register",
        &json!({"username": username, "email": email, "password": password}),
        None,
    )
    .await
}

/// Register a user and return their JWT token.
pub async fn register_and_get_token(app: &Router, username: &str, email: &str, password: &str) -> String {
    let (status, body) = register(app, username, email, password).await;
    assert_eq!(status, 201, "register failed: {body}");
    body["token"].as_str().unwrap().to_string()
}

pub async fn login(app: &Router, email: &str, password: &str) -> (u16, Value) {
    post_json(
        app,
        "/auth/login",
        &json!({"email": email, "password": password}),
        None,
    )
    .await
}