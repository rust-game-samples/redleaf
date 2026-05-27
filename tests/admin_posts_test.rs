mod common;

// ─── Auth guard ───────────────────────────────────────────────────────────────

#[tokio::test]
async fn admin_routes_require_auth() {
    let app = common::setup_app().await;

    for (method, uri) in [
        ("GET", "/admin"),
        ("GET", "/admin/posts"),
        ("GET", "/admin/posts/new"),
    ] {
        let (status, _) = if method == "GET" {
            common::get(&app, uri, None).await
        } else {
            common::post_form(&app, uri, &[], None).await
        };
        assert_eq!(status, 401, "{method} {uri} should require auth");
    }
}

// ─── Dashboard ────────────────────────────────────────────────────────────────

#[tokio::test]
async fn dashboard_shows_post_counts() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "admin", "admin@test.com", "pass").await;

    let (status, body) = common::get(&app, "/admin", Some(&token)).await;

    assert_eq!(status, 200);
    assert!(body.contains("Total Posts"), "dashboard should show Total Posts");
    assert!(body.contains("Published"), "dashboard should show Published count");
    assert!(body.contains("Drafts"), "dashboard should show Drafts count");
}

// ─── Create post ─────────────────────────────────────────────────────────────

#[tokio::test]
async fn create_post_redirects_to_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u1", "u1@test.com", "pass").await;

    let (status, _) = common::post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Hello World"),
            ("slug", "hello-world"),
            ("content", "## Welcome\n\nFirst post."),
            ("published", "on"),
        ],
        Some(&token),
    )
    .await;

    assert_eq!(status, 303);
}

#[tokio::test]
async fn created_post_appears_in_admin_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u2", "u2@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "My Test Post"),
            ("slug", "my-test-post"),
            ("content", "Content here."),
            ("published", "on"),
        ],
        Some(&token),
    )
    .await;

    let (status, body) = common::get(&app, "/admin/posts", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("My Test Post"), "post should appear in list");
}

#[tokio::test]
async fn draft_post_not_visible_in_public_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u3", "u3@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Secret Draft"),
            ("slug", "secret-draft"),
            ("content", "Not yet published."),
            // no "published" field → draft
        ],
        Some(&token),
    )
    .await;

    let (_, body) = common::get(&app, "/posts", None).await;
    assert!(!body.contains("Secret Draft"), "draft should not appear in public list");
}

#[tokio::test]
async fn duplicate_slug_returns_error_page() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u4", "u4@test.com", "pass").await;

    let fields = &[
        ("title", "Dup"),
        ("slug", "dup-slug"),
        ("content", "x"),
        ("published", "on"),
    ];
    common::post_form(&app, "/admin/posts", fields, Some(&token)).await;

    // Second post with same slug
    let (status, body) = common::post_form(&app, "/admin/posts", fields, Some(&token)).await;
    assert_ne!(status, 303, "should not redirect on duplicate slug");
    assert!(body.contains("Slug already exists"), "should show error message");
}

// ─── Edit / Update ────────────────────────────────────────────────────────────

#[tokio::test]
async fn edit_form_prefills_existing_values() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u5", "u5@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Original Title"), ("slug", "orig"), ("content", "orig content"), ("published", "on")],
        Some(&token),
    )
    .await;

    // Post ID is 1 since it's the first post in a fresh DB
    let (status, body) = common::get(&app, "/admin/posts/1/edit", Some(&token)).await;
    assert_eq!(status, 200);
    assert!(body.contains("Original Title"), "edit form should prefill title");
    assert!(body.contains("orig"), "edit form should prefill slug");
}

#[tokio::test]
async fn update_post_redirects_to_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u6", "u6@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Old"), ("slug", "old"), ("content", "old")],
        Some(&token),
    )
    .await;

    let (status, _) = common::post_form(
        &app,
        "/admin/posts/1",
        &[("title", "New Title"), ("slug", "new-slug"), ("content", "new content"), ("published", "on")],
        Some(&token),
    )
    .await;

    assert_eq!(status, 303);
}

#[tokio::test]
async fn updated_title_appears_in_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u7", "u7@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Before Update"), ("slug", "before"), ("content", "x"), ("published", "on")],
        Some(&token),
    )
    .await;

    common::post_form(
        &app,
        "/admin/posts/1",
        &[("title", "After Update"), ("slug", "after"), ("content", "x"), ("published", "on")],
        Some(&token),
    )
    .await;

    let (_, body) = common::get(&app, "/admin/posts", Some(&token)).await;
    assert!(body.contains("After Update"), "updated title should appear");
    assert!(!body.contains("Before Update"), "old title should be gone");
}

// ─── Toggle published ─────────────────────────────────────────────────────────

#[tokio::test]
async fn toggle_publishes_a_draft() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u8", "u8@test.com", "pass").await;

    // Create draft
    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Draft Post"), ("slug", "draft-post"), ("content", "draft")],
        Some(&token),
    )
    .await;

    // Should NOT be in public list yet
    let (_, public_before) = common::get(&app, "/posts", None).await;
    assert!(!public_before.contains("Draft Post"));

    // Toggle → publish
    let (status, _) = common::post_form(&app, "/admin/posts/1/toggle", &[], Some(&token)).await;
    assert_eq!(status, 303);

    // Now should be in public list
    let (_, public_after) = common::get(&app, "/posts", None).await;
    assert!(public_after.contains("Draft Post"), "post should be published after toggle");
}

// ─── Delete ───────────────────────────────────────────────────────────────────

#[tokio::test]
async fn delete_post_redirects_and_removes_it() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u9", "u9@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "To Delete"), ("slug", "to-delete"), ("content", "bye"), ("published", "on")],
        Some(&token),
    )
    .await;

    let (status, _) = common::post_form(&app, "/admin/posts/1/delete", &[], Some(&token)).await;
    assert_eq!(status, 303);

    let (_, body) = common::get(&app, "/admin/posts", Some(&token)).await;
    assert!(!body.contains("To Delete"), "deleted post should not appear in list");
}

#[tokio::test]
async fn delete_nonexistent_post_still_redirects() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "u10", "u10@test.com", "pass").await;

    let (status, _) = common::post_form(&app, "/admin/posts/9999/delete", &[], Some(&token)).await;
    assert_eq!(status, 303, "deleting non-existent post should still redirect");
}