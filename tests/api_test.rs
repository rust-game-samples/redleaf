mod common;

use serde_json::json;

// ─── GET /api/posts ───────────────────────────────────────────────────────────

#[tokio::test]
async fn api_list_returns_empty() {
    let app = common::setup_app().await;
    let (status, body) = common::get(&app, "/api/posts", None).await;
    assert_eq!(status, 200);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["items"].as_array().unwrap().len(), 0);
    assert_eq!(v["total"], 0);
}

#[tokio::test]
async fn api_list_returns_only_published() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a1", "a1@test.com", "pass").await;

    // published
    common::post_json(
        &app, "/api/posts",
        &json!({"title": "Pub", "slug": "pub", "content": "c", "published": true}),
        Some(&token),
    ).await;
    // draft
    common::post_json(
        &app, "/api/posts",
        &json!({"title": "Draft", "slug": "draft", "content": "c", "published": false}),
        Some(&token),
    ).await;

    let (status, body) = common::get(&app, "/api/posts", None).await;
    assert_eq!(status, 200);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    let items = v["items"].as_array().unwrap();
    assert_eq!(items.len(), 1);
    assert_eq!(items[0]["title"], "Pub");
}

// ─── GET /api/posts/{id} ──────────────────────────────────────────────────────

#[tokio::test]
async fn api_show_returns_published_post() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a2", "a2@test.com", "pass").await;

    let (_, created) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "Hello", "slug": "hello", "content": "World", "published": true}),
        Some(&token),
    ).await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = common::get(&app, &format!("/api/posts/{}", id), None).await;
    assert_eq!(status, 200);
    let v: serde_json::Value = serde_json::from_str(&body).unwrap();
    assert_eq!(v["title"], "Hello");
    assert_eq!(v["slug"], "hello");
}

#[tokio::test]
async fn api_show_draft_returns_404() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a3", "a3@test.com", "pass").await;

    let (_, created) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "Hidden", "slug": "hidden", "content": "secret", "published": false}),
        Some(&token),
    ).await;
    let id = created["id"].as_i64().unwrap();

    let (status, _) = common::get(&app, &format!("/api/posts/{}", id), None).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn api_show_nonexistent_returns_404() {
    let app = common::setup_app().await;
    let (status, _) = common::get(&app, "/api/posts/9999", None).await;
    assert_eq!(status, 404);
}

// ─── POST /api/posts ─────────────────────────────────────────────────────────

#[tokio::test]
async fn api_create_requires_auth() {
    let app = common::setup_app().await;
    let (status, body) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "T", "content": "C"}),
        None,
    ).await;
    // no auth → redirect to login
    assert_eq!(status, 303);
}

#[tokio::test]
async fn api_create_returns_201_with_post() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a4", "a4@test.com", "pass").await;

    let (status, body) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "New Post", "content": "Body text", "published": true}),
        Some(&token),
    ).await;
    assert_eq!(status, 201);
    assert_eq!(body["title"], "New Post");
    assert_eq!(body["slug"], "new-post");   // auto-generated
    assert_eq!(body["published"], true);
}

#[tokio::test]
async fn api_create_with_explicit_slug() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a5", "a5@test.com", "pass").await;

    let (status, body) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "Custom", "slug": "my-custom-slug", "content": "x"}),
        Some(&token),
    ).await;
    assert_eq!(status, 201);
    assert_eq!(body["slug"], "my-custom-slug");
}

#[tokio::test]
async fn api_create_duplicate_slug_returns_409() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a6", "a6@test.com", "pass").await;

    let payload = json!({"title": "T", "slug": "dup", "content": "x"});
    common::post_json(&app, "/api/posts", &payload, Some(&token)).await;
    let (status, body) = common::post_json(&app, "/api/posts", &payload, Some(&token)).await;
    assert_eq!(status, 409);
    assert!(body["error"].as_str().unwrap().contains("Slug"));
}

// ─── PUT /api/posts/{id} ─────────────────────────────────────────────────────

#[tokio::test]
async fn api_update_post() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a7", "a7@test.com", "pass").await;

    let (_, created) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "Old", "slug": "old", "content": "old body"}),
        Some(&token),
    ).await;
    let id = created["id"].as_i64().unwrap();

    let (status, body) = common::put_json(
        &app, &format!("/api/posts/{}", id),
        &json!({"title": "New Title", "published": true}),
        Some(&token),
    ).await;
    assert_eq!(status, 200);
    assert_eq!(body["title"], "New Title");
    assert_eq!(body["published"], true);
    assert_eq!(body["slug"], "old"); // unchanged
}

#[tokio::test]
async fn api_update_nonexistent_returns_404() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a8", "a8@test.com", "pass").await;

    let (status, _) = common::put_json(
        &app, "/api/posts/9999",
        &json!({"title": "x"}),
        Some(&token),
    ).await;
    assert_eq!(status, 404);
}

// ─── DELETE /api/posts/{id} ───────────────────────────────────────────────────

#[tokio::test]
async fn api_delete_post() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a9", "a9@test.com", "pass").await;

    let (_, created) = common::post_json(
        &app, "/api/posts",
        &json!({"title": "Bye", "slug": "bye", "content": "bye", "published": true}),
        Some(&token),
    ).await;
    let id = created["id"].as_i64().unwrap();

    let (status, _) = common::delete_req(&app, &format!("/api/posts/{}", id), Some(&token)).await;
    assert_eq!(status, 204);

    // Confirm gone
    let (get_status, _) = common::get(&app, &format!("/api/posts/{}", id), None).await;
    assert_eq!(get_status, 404);
}

#[tokio::test]
async fn api_delete_nonexistent_returns_404() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "a10", "a10@test.com", "pass").await;

    let (status, _) = common::delete_req(&app, "/api/posts/9999", Some(&token)).await;
    assert_eq!(status, 404);
}