mod common;

use serde_json::json;

#[tokio::test]
async fn register_returns_201_with_token_and_user() {
    let app = common::setup_app().await;
    let (status, body) = common::register(&app, "alice", "alice@example.com", "pass123").await;

    assert_eq!(status, 201);
    assert!(body["token"].as_str().is_some(), "token should be present");
    assert_eq!(body["user"]["username"], "alice");
    assert_eq!(body["user"]["email"], "alice@example.com");
    assert!(
        body["user"].get("password_hash").is_none(),
        "password_hash must not be serialized"
    );
}

#[tokio::test]
async fn register_duplicate_email_returns_409() {
    let app = common::setup_app().await;

    common::register(&app, "bob", "bob@example.com", "pass123").await;
    let (status, _) = common::register(&app, "bob2", "bob@example.com", "pass123").await;

    assert_eq!(status, 409);
}

#[tokio::test]
async fn register_duplicate_username_returns_409() {
    let app = common::setup_app().await;

    common::register(&app, "carol", "carol@example.com", "pass123").await;
    let (status, _) = common::register(&app, "carol", "carol2@example.com", "pass123").await;

    assert_eq!(status, 409);
}

#[tokio::test]
async fn login_with_correct_credentials_returns_200_with_token() {
    let app = common::setup_app().await;
    common::register(&app, "dave", "dave@example.com", "secret").await;

    let (status, body) = common::login(&app, "dave@example.com", "secret").await;

    assert_eq!(status, 200);
    assert!(body["token"].as_str().is_some());
    assert_eq!(body["user"]["email"], "dave@example.com");
}

#[tokio::test]
async fn login_with_wrong_password_returns_401() {
    let app = common::setup_app().await;
    common::register(&app, "eve", "eve@example.com", "correct").await;

    let (status, _) = common::login(&app, "eve@example.com", "wrong").await;

    assert_eq!(status, 401);
}

#[tokio::test]
async fn login_with_unknown_email_returns_401() {
    let app = common::setup_app().await;

    let (status, _) = common::post_json(
        &app,
        "/auth/login",
        &json!({"email": "nobody@example.com", "password": "pass"}),
        None,
    )
    .await;

    assert_eq!(status, 401);
}