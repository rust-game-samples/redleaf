mod common;

#[tokio::test]
async fn home_page_returns_200() {
    let app = common::setup_app().await;
    let (status, body) = common::get(&app, "/", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("RedLeaf CMS"));
}

#[tokio::test]
async fn post_list_returns_200_when_empty() {
    let app = common::setup_app().await;
    let (status, body) = common::get(&app, "/posts", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("Posts"), "should render posts page");
}

#[tokio::test]
async fn post_list_shows_only_published_posts() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "pub1", "pub1@test.com", "pass").await;

    // Create published post
    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Public Post"), ("slug", "public-post"), ("content", "visible"), ("published", "on")],
        Some(&token),
    )
    .await;

    // Create draft
    common::post_form(
        &app,
        "/admin/posts",
        &[("title", "Hidden Draft"), ("slug", "hidden-draft"), ("content", "hidden")],
        Some(&token),
    )
    .await;

    let (status, body) = common::get(&app, "/posts", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("Public Post"), "published post should be visible");
    assert!(!body.contains("Hidden Draft"), "draft should not be visible");
}

#[tokio::test]
async fn single_post_renders_markdown() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "pub2", "pub2@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Markdown Post"),
            ("slug", "markdown-post"),
            ("content", "## Hello\n\nThis is **bold** text."),
            ("published", "on"),
        ],
        Some(&token),
    )
    .await;

    let (status, body) = common::get(&app, "/posts/1", None).await;
    assert_eq!(status, 200);
    assert!(body.contains("<h2>Hello</h2>"), "should render Markdown headings as HTML");
    assert!(body.contains("<strong>bold</strong>"), "should render bold text");
    assert!(body.contains("Markdown Post"), "should show post title");
}

#[tokio::test]
async fn single_post_not_found_returns_404() {
    let app = common::setup_app().await;
    let (status, _) = common::get(&app, "/posts/9999", None).await;
    assert_eq!(status, 404);
}

#[tokio::test]
async fn post_excerpt_shown_in_list() {
    let app = common::setup_app().await;
    let token = common::register_and_get_token(&app, "pub3", "pub3@test.com", "pass").await;

    common::post_form(
        &app,
        "/admin/posts",
        &[
            ("title", "Post With Excerpt"),
            ("slug", "post-excerpt"),
            ("content", "Full content here."),
            ("excerpt", "Short summary"),
            ("published", "on"),
        ],
        Some(&token),
    )
    .await;

    let (_, body) = common::get(&app, "/posts", None).await;
    assert!(body.contains("Short summary"), "excerpt should appear in post list");
}