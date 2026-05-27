mod common;

use common::*;

async fn setup_with_user() -> (axum::Router, String) {
    let app = setup_app().await;
    let token = register_and_get_token(&app, "author", "author@example.com", "password123").await;
    (app, token)
}

// ─── Category admin CRUD ──────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_category_list_empty() {
    let (app, token) = setup_with_user().await;
    let (status, body) = get(&app, "/admin/categories", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("No categories yet"));
}

#[tokio::test]
async fn test_admin_create_category() {
    let (app, token) = setup_with_user().await;
    let (status, _) = post_form(
        &app,
        "/admin/categories",
        &[("name", "Technology"), ("slug", "technology")],
        Some(&token),
    ).await;
    assert_eq!(status, 303, "expected redirect after create");

    let (status, body) = get(&app, "/admin/categories", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("Technology"));
}

#[tokio::test]
async fn test_admin_create_category_auto_slug() {
    let (app, token) = setup_with_user().await;
    let (status, _) = post_form(
        &app,
        "/admin/categories",
        &[("name", "Web Development")],
        Some(&token),
    ).await;
    assert_eq!(status, 303);

    let (status, body) = get(&app, "/admin/categories", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("Web Development"));
}

#[tokio::test]
async fn test_admin_edit_category_form() {
    let (app, token) = setup_with_user().await;
    post_form(&app, "/admin/categories", &[("name", "Rust"), ("slug", "rust")], Some(&token)).await;

    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    let id = extract_id_from_href(&body, "/admin/categories/", "/edit");

    let (status, body) = get(&app, &format!("/admin/categories/{}/edit", id), Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("Edit Category"));
    assert!(body.contains("Rust"));
}

#[tokio::test]
async fn test_admin_update_category() {
    let (app, token) = setup_with_user().await;
    post_form(&app, "/admin/categories", &[("name", "Old Name"), ("slug", "old-name")], Some(&token)).await;

    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    let id = extract_id_from_href(&body, "/admin/categories/", "/edit");

    let (status, _) = post_form(
        &app,
        &format!("/admin/categories/{}", id),
        &[("name", "New Name"), ("slug", "new-name")],
        Some(&token),
    ).await;
    assert_eq!(status, 303);

    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    assert!(body.contains("New Name"));
}

#[tokio::test]
async fn test_admin_delete_category() {
    let (app, token) = setup_with_user().await;
    post_form(&app, "/admin/categories", &[("name", "ToDelete"), ("slug", "to-delete")], Some(&token)).await;

    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    let id = extract_id_from_href(&body, "/admin/categories/", "/edit");

    let (status, _) = post_form(&app, &format!("/admin/categories/{}/delete", id), &[], Some(&token)).await;
    assert_eq!(status, 303);

    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    assert!(!body.contains("ToDelete"));
}

// ─── Tag admin ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn test_admin_tag_list_empty() {
    let (app, token) = setup_with_user().await;
    let (status, body) = get(&app, "/admin/tags", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("No tags yet"));
}

#[tokio::test]
async fn test_admin_tags_created_via_post() {
    let (app, token) = setup_with_user().await;
    post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Tagged Post"),
            ("slug", "tagged-post"),
            ("content", "Hello"),
            ("tags", "rust, web"),
        ],
        Some(&token),
    ).await;

    let (status, body) = get(&app, "/admin/tags", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("rust"));
    assert!(body.contains("web"));
}

#[tokio::test]
async fn test_admin_delete_tag() {
    let (app, token) = setup_with_user().await;
    post_form(
        &app,
        "/admin/posts",
        &[("title", "T"), ("slug", "t"), ("content", "C"), ("tags", "deleteme")],
        Some(&token),
    ).await;

    let (_, body) = get(&app, "/admin/tags", Some(&token)).await;
    let id = extract_id_from_href(&body, "/admin/tags/", "/delete");

    let (status, _) = post_form(&app, &format!("/admin/tags/{}/delete", id), &[], Some(&token)).await;
    assert_eq!(status, 303);

    let (_, body) = get(&app, "/admin/tags", Some(&token)).await;
    assert!(!body.contains("deleteme"));
}

// ─── Public taxonomy pages ────────────────────────────────────────────────────

#[tokio::test]
async fn test_category_page_not_found() {
    let app = setup_app().await;
    let (status, _) = get(&app, "/categories/nonexistent", None).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn test_tag_page_not_found() {
    let app = setup_app().await;
    let (status, _) = get(&app, "/tags/nonexistent", None).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn test_category_page_shows_posts() {
    let (app, token) = setup_with_user().await;

    // Create category then post in that category
    post_form(&app, "/admin/categories", &[("name", "Tech"), ("slug", "tech")], Some(&token)).await;
    let (_, body) = get(&app, "/admin/categories", Some(&token)).await;
    let cat_id = extract_id_from_href(&body, "/admin/categories/", "/edit");

    post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Tech Post"),
            ("slug", "tech-post"),
            ("content", "Content here"),
            ("published", "on"),
            ("category_id", &cat_id),
        ],
        Some(&token),
    ).await;

    let (status, body) = get(&app, "/categories/tech", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("Tech"));
    assert!(body.contains("Tech Post"));
}

#[tokio::test]
async fn test_tag_page_shows_posts() {
    let (app, token) = setup_with_user().await;

    post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Rust Post"),
            ("slug", "rust-post"),
            ("content", "Content"),
            ("published", "on"),
            ("tags", "rust"),
        ],
        Some(&token),
    ).await;

    let (status, body) = get(&app, "/tags/rust", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("rust"));
    assert!(body.contains("Rust Post"));
}

// ─── Post form shows category/tags ───────────────────────────────────────────

#[tokio::test]
async fn test_new_post_form_has_category_dropdown() {
    let (app, token) = setup_with_user().await;
    post_form(&app, "/admin/categories", &[("name", "Science"), ("slug", "science")], Some(&token)).await;

    let (status, body) = get(&app, "/admin/posts/new", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("Science"));
    assert!(body.contains("category_id"));
}

#[tokio::test]
async fn test_edit_post_form_shows_tags() {
    let (app, token) = setup_with_user().await;
    post_form(
        &app,
        "/admin/posts",
        &[("title", "My Post"), ("slug", "my-post"), ("content", "C"), ("tags", "alpha, beta")],
        Some(&token),
    ).await;

    let (_, body) = get(&app, "/admin/posts", Some(&token)).await;
    let post_id = extract_id_from_href(&body, "/admin/posts/", "/edit");

    let (status, body) = get(&app, &format!("/admin/posts/{}/edit", post_id), Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("alpha"));
    assert!(body.contains("beta"));
}

// ─── Helper ───────────────────────────────────────────────────────────────────

/// Find the first purely-numeric segment between `prefix` and `suffix` in `html`.
fn extract_id_from_href(html: &str, prefix: &str, suffix: &str) -> String {
    let mut cursor = 0;
    while let Some(rel) = html[cursor..].find(prefix) {
        let after_prefix = cursor + rel + prefix.len();
        let rest = &html[after_prefix..];
        if let Some(end) = rest.find(suffix) {
            let candidate = rest[..end].trim();
            if !candidate.is_empty() && candidate.chars().all(|c| c.is_ascii_digit()) {
                return candidate.to_string();
            }
        }
        cursor = after_prefix;
    }
    panic!("could not find numeric ID between '{}' and '{}' in HTML", prefix, suffix);
}