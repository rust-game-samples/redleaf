use askama::Template;
use axum::{
    extract::{Extension, Form, Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::{Deserialize, Deserializer};

fn empty_string_as_none<'de, D>(d: D) -> Result<Option<i64>, D::Error>
where
    D: Deserializer<'de>,
{
    let s = Option::<String>::deserialize(d)?;
    match s.as_deref() {
        None | Some("") => Ok(None),
        Some(v) => v.parse::<i64>().map(Some).map_err(serde::de::Error::custom),
    }
}

use crate::{
    auth::{generate_token, Claims},
    db::DbPool,
    errors::AppError,
    filters,
    middleware::has_capability,
    models::{
        Category, CategoryWithCount,
        Comment, CommentWithPost,
        Media,
        NavMenu, NavMenuItem, LOCATIONS,
        Page, CreatePage, UpdatePage,
        Post, PostWithAuthor,
        PostMeta,
        PostRevision,
        Setting,
        Tag, TagWithCount,
        Widget, WidgetArea,
        post::{CreatePost, UpdatePost},
        user::{CreateUser, LoginUser, UpdateProfile},
        User,
    },
    util::{render, slugify, Pagination, PER_PAGE},
};

// ─── Comment templates ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/comments/list.html")]
struct CommentListTemplate {
    comments: Vec<CommentWithPost>,
    pending_count: i64,
}

// ─── User templates ───────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/users/list.html")]
struct UserListTemplate {
    users: Vec<User>,
    roles: Vec<String>,
}

impl UserListTemplate {
    fn role_selected(&self, role: &str, current: &str) -> bool {
        role == current
    }
}

#[derive(Template)]
#[template(path = "admin/profile.html")]
struct ProfileTemplate {
    username: String,
    email: String,
    display_name: String,
    bio: String,
    website: String,
    avatar_url: String,
    saved: Option<String>,
    error: Option<String>,
    pw_saved: Option<String>,
    pw_error: Option<String>,
}

// ─── Login / register templates ──────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/login.html")]
struct LoginTemplate {
    error: Option<String>,
}

#[derive(Template)]
#[template(path = "admin/register.html")]
struct RegisterTemplate {
    error: Option<String>,
    prefill_username: String,
    prefill_email: String,
}

// ─── Settings template ───────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/settings.html")]
struct SettingsTemplate {
    post_url_type: String,
    site_name: String,
    site_description: String,
    logo_url: String,
    active_theme: String,
    robots_txt: String,
    saved: Option<String>,
}

// ─── Category templates ───────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/categories/list.html")]
struct CategoryListTemplate {
    categories: Vec<CategoryWithCount>,
}

#[derive(Template)]
#[template(path = "admin/categories/form.html")]
struct CategoryFormTemplate {
    heading: String,
    action: String,
    name: String,
    slug: String,
    error: Option<String>,
}

// ─── Tag templates ────────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/tags/list.html")]
struct TagListTemplate {
    tags: Vec<TagWithCount>,
}

// ─── Page templates ──────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/pages/list.html")]
struct PageListTemplate {
    pages: Vec<Page>,
}

#[derive(Template)]
#[template(path = "admin/pages/form.html")]
struct PageFormTemplate {
    heading: String,
    action: String,
    title: String,
    slug: String,
    content: String,
    status: String,
    parent_id: Option<i64>,
    all_pages: Vec<Page>,
    error: Option<String>,
    media_images: Vec<Media>,
}

// ─── Revision template ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/posts/revisions.html")]
struct RevisionListTemplate {
    post_id: i64,
    post_title: String,
    revisions: Vec<PostRevision>,
}

// ─── Media templates ──────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/media/list.html")]
struct MediaListTemplate {
    media: Vec<Media>,
    error: Option<String>,
}

// ─── Template structs ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/dashboard.html")]
struct DashboardTemplate {
    total: i64,
    published: i64,
    drafts: i64,
}

#[derive(Template)]
#[template(path = "admin/posts/list.html")]
struct PostListTemplate {
    posts: Vec<PostWithAuthor>,
    paging: Pagination,
}

#[derive(Template)]
#[template(path = "admin/posts/form.html")]
struct PostFormTemplate {
    heading: String,
    action: String,
    post_id: Option<i64>,
    title: String,
    slug: String,
    content: String,
    excerpt: String,
    published: bool,
    sticky: bool,
    scheduled_at: String,
    error: Option<String>,
    categories: Vec<Category>,
    selected_category_id: Option<i64>,
    tag_names: String,
    media_images: Vec<Media>,
    featured_image_id: Option<i64>,
    featured_image_url: Option<String>,
    meta_fields: Vec<(String, String)>,
    seo_title: String,
    seo_description: String,
}

impl PostFormTemplate {
    fn new(action: impl Into<String>, heading: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            action: action.into(),
            post_id: None,
            title: String::new(),
            slug: String::new(),
            content: String::new(),
            excerpt: String::new(),
            published: false,
            sticky: false,
            scheduled_at: String::new(),
            error: None,
            categories: vec![],
            selected_category_id: None,
            tag_names: String::new(),
            media_images: vec![],
            featured_image_id: None,
            featured_image_url: None,
            meta_fields: vec![],
            seo_title: String::new(),
            seo_description: String::new(),
        }
    }

    fn from_post(post: &Post) -> Self {
        Self {
            heading: "Edit Post".into(),
            action: format!("/admin/posts/{}", post.id),
            post_id: Some(post.id),
            title: post.title.clone(),
            slug: post.slug.clone(),
            content: post.content.clone(),
            excerpt: post.excerpt.clone().unwrap_or_default(),
            published: post.published,
            sticky: post.sticky,
            scheduled_at: post.scheduled_at
                .map(|dt| dt.format("%Y-%m-%dT%H:%M").to_string())
                .unwrap_or_default(),
            error: None,
            categories: vec![],
            selected_category_id: post.category_id,
            tag_names: String::new(),
            media_images: vec![],
            featured_image_id: post.featured_image_id,
            featured_image_url: None,
            meta_fields: vec![],
            seo_title: post.seo_title.clone(),
            seo_description: post.seo_description.clone(),
        }
    }
}

// ─── Form input ──────────────────────────────────────────────────────────────

#[derive(Deserialize, Default)]
struct PageQuery {
    page: Option<i64>,
}

#[derive(Debug, Deserialize)]
struct PostForm {
    title: String,
    slug: String,
    content: String,
    excerpt: Option<String>,
    published: Option<String>,
    sticky: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    category_id: Option<i64>,
    tags: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    featured_image_id: Option<i64>,
    scheduled_at: Option<String>,
    #[serde(rename = "meta_key[]", default)]
    meta_keys: Vec<String>,
    #[serde(rename = "meta_value[]", default)]
    meta_values: Vec<String>,
    #[serde(default)]
    seo_title: String,
    #[serde(default)]
    seo_description: String,
}

#[derive(Debug, Deserialize)]
struct CategoryForm {
    name: String,
    slug: Option<String>,
}

// ─── Router ──────────────────────────────────────────────────────────────────

pub fn admin_login_routes() -> Router<DbPool> {
    Router::new()
        .route("/admin/login", get(login_page).post(login_submit))
        .route("/admin/register", get(register_page).post(register_submit))
        .route("/admin/logout", post(logout))
}

pub fn admin_routes() -> Router<DbPool> {
    Router::new()
        .route("/", get(dashboard))
        .route("/posts", get(list_posts).post(create_post))
        .route("/posts/new", get(new_post_form))
        .route("/posts/{id}", post(update_post))
        .route("/posts/{id}/edit", get(edit_post_form))
        .route("/posts/{id}/delete", post(delete_post))
        .route("/posts/{id}/toggle", post(toggle_published))
        .route("/categories", get(list_categories).post(create_category))
        .route("/categories/new", get(new_category_form))
        .route("/categories/{id}/edit", get(edit_category_form))
        .route("/categories/{id}", post(update_category))
        .route("/categories/{id}/delete", post(delete_category))
        .route("/tags", get(list_tags))
        .route("/tags/{id}/delete", post(delete_tag))
        .route("/settings", get(settings_page).post(settings_save))
        .route("/settings/robots", post(save_robots_txt))
        .route("/media", get(list_media))
        .route("/media/upload", post(upload_media))
        .route("/media/{id}/delete", post(delete_media_handler))
        .route("/pages", get(list_pages).post(create_page))
        .route("/pages/new", get(new_page_form))
        .route("/pages/{id}/edit", get(edit_page_form))
        .route("/pages/{id}", post(update_page_handler))
        .route("/pages/{id}/delete", post(delete_page_handler))
        .route("/posts/{id}/revisions", get(list_revisions))
        .route("/posts/{id}/revisions/{rev_id}/restore", post(restore_revision))
        .route("/widgets", get(list_widgets).post(create_widget))
        .route("/widgets/reorder", post(reorder_widgets))
        .route("/widgets/areas", post(create_widget_area))
        .route("/widgets/areas/{id}/delete", post(delete_widget_area))
        .route("/widgets/{id}", post(update_widget))
        .route("/widgets/{id}/delete", post(delete_widget))
        .route("/menus", get(list_menus))
        .route("/menus/new", get(new_menu_form).post(create_menu))
        .route("/menus/{id}/edit", get(edit_menu_form).post(update_menu))
        .route("/menus/{id}/delete", post(delete_menu))
        .route("/menus/{id}/items", post(create_menu_item))
        .route("/menus/{id}/items/reorder", post(reorder_menu_items))
        .route("/menus/{id}/items/{item_id}", post(update_menu_item))
        .route("/menus/{id}/items/{item_id}/delete", post(delete_menu_item))
        .route("/users", get(list_users))
        .route("/users/{id}/role", post(update_user_role))
        .route("/profile", get(edit_profile_form).post(update_profile))
        .route("/profile/password", post(change_password))
        .route("/comments", get(list_comments))
        .route("/comments/{id}/approve", post(approve_comment))
        .route("/comments/{id}/reject", post(reject_comment))
        .route("/comments/{id}/spam", post(spam_comment))
}

// ─── Login / logout handlers ─────────────────────────────────────────────────

async fn login_page() -> Result<Response, AppError> {
    render(LoginTemplate { error: None })
}

async fn login_submit(
    State(pool): State<DbPool>,
    Form(payload): Form<LoginUser>,
) -> Result<Response, AppError> {
    let user = match User::authenticate(&pool, payload)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        Some(u) => u,
        None => {
            return render(LoginTemplate {
                error: Some("Invalid email or password.".to_string()),
            });
        }
    };

    let token = generate_token(&user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap())
}

async fn logout() -> Response {
    Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin/login")
        .header(
            header::SET_COOKIE,
            "session=; HttpOnly; Path=/; SameSite=Strict; Max-Age=0",
        )
        .body(axum::body::Body::empty())
        .unwrap()
}

async fn register_page() -> Result<Response, AppError> {
    render(RegisterTemplate {
        error: None,
        prefill_username: String::new(),
        prefill_email: String::new(),
    })
}

async fn register_submit(
    State(pool): State<DbPool>,
    Form(payload): Form<CreateUser>,
) -> Result<Response, AppError> {
    let prefill_username = payload.username.clone();
    let prefill_email = payload.email.clone();

    let user = match User::create(&pool, payload).await {
        Ok(u) => u,
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Username or email is already taken.".to_string()
            } else {
                return Err(AppError::Internal(e.to_string()));
            };
            return render(RegisterTemplate {
                error: Some(msg),
                prefill_username,
                prefill_email,
            });
        }
    };

    let token = generate_token(&user)
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let cookie = format!(
        "session={}; HttpOnly; Path=/; SameSite=Strict; Max-Age=604800",
        token
    );
    Ok(Response::builder()
        .status(StatusCode::SEE_OTHER)
        .header(header::LOCATION, "/admin")
        .header(header::SET_COOKIE, cookie)
        .body(axum::body::Body::empty())
        .unwrap())
}

// ─── Helpers ─────────────────────────────────────────────────────────────────

async fn resolve_tags(pool: &DbPool, input: Option<&str>) -> Result<Vec<i64>, AppError> {
    let Some(s) = input else { return Ok(vec![]) };
    let mut ids = Vec::new();
    for name in s.split(',').map(str::trim).filter(|s| !s.is_empty()) {
        let tag = Tag::find_or_create(pool, name).await?;
        ids.push(tag.id);
    }
    Ok(ids)
}

fn form_to_create(form: &PostForm) -> CreatePost {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };
    let scheduled_at = form.scheduled_at
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok())
        .map(|ndt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc));
    CreatePost {
        title: form.title.clone(),
        slug,
        content: form.content.clone(),
        excerpt: form.excerpt.clone().filter(|s| !s.trim().is_empty()),
        published: form.published.is_some(),
        sticky: form.sticky.is_some(),
        author_id: None,
        category_id: form.category_id,
        featured_image_id: form.featured_image_id,
        scheduled_at,
        seo_title: form.seo_title.trim().to_string(),
        seo_description: form.seo_description.trim().to_string(),
    }
}

fn meta_pairs_from_form(form: &PostForm) -> Vec<(String, String)> {
    form.meta_keys
        .iter()
        .zip(form.meta_values.iter())
        .filter(|(k, _)| !k.trim().is_empty())
        .map(|(k, v)| (k.clone(), v.clone()))
        .collect()
}

// ─── Handlers ────────────────────────────────────────────────────────────────

async fn dashboard(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (total, published) = tokio::join!(
        Post::count_all(&pool),
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM posts WHERE published = 1")
            .fetch_one(&pool),
    );
    let total = total?;
    let published = published?;
    render(DashboardTemplate { total, published, drafts: total - published })
}

async fn list_posts(
    State(pool): State<DbPool>,
    Query(q): Query<PageQuery>,
) -> Result<Response, AppError> {
    let page = q.page.unwrap_or(1).max(1);
    let (posts, total) = tokio::join!(
        Post::find_admin_paginated(&pool, page, PER_PAGE),
        Post::count_all(&pool),
    );
    let paging = Pagination::new(page, total?, PER_PAGE, "/admin/posts");
    render(PostListTemplate { posts: posts?, paging })
}

async fn new_post_form(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (categories, media_images) = tokio::join!(
        Category::find_all(&pool),
        Media::find_all(&pool),
    );
    let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
    tmpl.categories = categories?;
    tmpl.media_images = media_images?.into_iter().filter(|m| m.is_image()).collect();
    render(tmpl)
}

async fn create_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let meta_pairs = meta_pairs_from_form(&form);
    let tag_input = form.tags.clone();
    let mut payload = form_to_create(&form);
    payload.author_id = Some(claims.sub);

    match Post::create(&pool, payload).await {
        Ok(post) => {
            let tag_ids = resolve_tags(&pool, tag_input.as_deref()).await?;
            Post::set_tags(&pool, post.id, &tag_ids).await?;
            PostMeta::replace_all(&pool, post.id, &meta_pairs).await?;
            Ok(Redirect::to("/admin/posts").into_response())
        }
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Slug already exists. Please choose a different slug.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            let (categories, media_images) = tokio::join!(
                Category::find_all(&pool), Media::find_all(&pool)
            );
            let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
            tmpl.error = Some(msg);
            tmpl.categories = categories?;
            tmpl.media_images = media_images?.into_iter().filter(|m| m.is_image()).collect();
            render(tmpl)
        }
    }
}

async fn edit_post_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let (post_res, categories, media_images, tags_res, meta_res) = tokio::join!(
        Post::find_by_id(&pool, id),
        Category::find_all(&pool),
        Media::find_all(&pool),
        Tag::find_by_post(&pool, id),
        PostMeta::get_all(&pool, id),
    );
    let post = post_res?.ok_or(AppError::NotFound)?;
    let tag_names = tags_res?.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ");
    let meta_fields = meta_res?.into_iter().map(|m| (m.meta_key, m.meta_value)).collect();
    let images: Vec<Media> = media_images?.into_iter().filter(|m| m.is_image()).collect();
    let featured_image_url = post.featured_image_id
        .and_then(|fid| images.iter().find(|m| m.id == fid).map(|m| m.url.clone()));
    let mut tmpl = PostFormTemplate::from_post(&post);
    tmpl.categories = categories?;
    tmpl.tag_names = tag_names;
    tmpl.media_images = images;
    tmpl.featured_image_url = featured_image_url;
    tmpl.meta_fields = meta_fields;
    render(tmpl)
}

async fn update_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };

    // Save revision before update
    if let Ok(Some(cur)) = Post::find_by_id(&pool, id).await {
        PostRevision::save(&pool, id, &cur.title, &cur.content, cur.excerpt.as_deref(), Some(claims.sub)).await?;
    }

    let scheduled_at = form.scheduled_at
        .as_deref()
        .filter(|s| !s.trim().is_empty())
        .and_then(|s| chrono::NaiveDateTime::parse_from_str(s, "%Y-%m-%dT%H:%M").ok())
        .map(|ndt| chrono::DateTime::<chrono::Utc>::from_naive_utc_and_offset(ndt, chrono::Utc));

    let payload = UpdatePost {
        title: Some(form.title.clone()),
        slug: Some(slug),
        content: Some(form.content.clone()),
        excerpt: Some(form.excerpt.clone().filter(|s| !s.trim().is_empty())),
        published: Some(form.published.is_some()),
        sticky: Some(form.sticky.is_some()),
        category_id: Some(form.category_id),
        featured_image_id: Some(form.featured_image_id),
        scheduled_at: Some(scheduled_at),
        seo_title: Some(form.seo_title.trim().to_string()),
        seo_description: Some(form.seo_description.trim().to_string()),
    };

    let post = Post::update(&pool, id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;
    let tag_ids = resolve_tags(&pool, form.tags.as_deref()).await?;
    Post::set_tags(&pool, post.id, &tag_ids).await?;
    PostMeta::replace_all(&pool, post.id, &meta_pairs_from_form(&form)).await?;
    Ok(Redirect::to("/admin/posts").into_response())
}

async fn delete_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Post::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/posts").into_response())
}

async fn toggle_published(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let post = Post::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    let payload = UpdatePost {
        title: None,
        slug: None,
        content: None,
        excerpt: None,
        published: Some(!post.published),
        sticky: None,
        category_id: None,
        featured_image_id: None,
        scheduled_at: None,
        seo_title: None,
        seo_description: None,
    };
    Post::update(&pool, id, payload).await?;
    Ok(Redirect::to("/admin/posts").into_response())
}

// ─── Category handlers ────────────────────────────────────────────────────────

async fn list_categories(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let categories = Category::find_all_with_count(&pool).await?;
    render(CategoryListTemplate { categories })
}

async fn new_category_form() -> Result<Response, AppError> {
    render(CategoryFormTemplate {
        heading: "New Category".into(),
        action: "/admin/categories".into(),
        name: String::new(),
        slug: String::new(),
        error: None,
    })
}

async fn create_category(
    State(pool): State<DbPool>,
    Form(form): Form<CategoryForm>,
) -> Result<Response, AppError> {
    let slug = if form.slug.as_deref().unwrap_or("").trim().is_empty() {
        slugify(&form.name)
    } else {
        form.slug.unwrap().trim().to_string()
    };
    match Category::create(&pool, &form.name, &slug).await {
        Ok(_) => Ok(Redirect::to("/admin/categories").into_response()),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Name or slug already exists.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            render(CategoryFormTemplate {
                heading: "New Category".into(),
                action: "/admin/categories".into(),
                name: form.name,
                slug,
                error: Some(msg),
            })
        }
    }
}

async fn edit_category_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let cat = Category::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    render(CategoryFormTemplate {
        heading: "Edit Category".into(),
        action: format!("/admin/categories/{}", id),
        name: cat.name,
        slug: cat.slug,
        error: None,
    })
}

async fn update_category(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<CategoryForm>,
) -> Result<Response, AppError> {
    let slug = if form.slug.as_deref().unwrap_or("").trim().is_empty() {
        slugify(&form.name)
    } else {
        form.slug.unwrap().trim().to_string()
    };
    match Category::update(&pool, id, &form.name, &slug).await {
        Ok(_) => Ok(Redirect::to("/admin/categories").into_response()),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Name or slug already exists.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            render(CategoryFormTemplate {
                heading: "Edit Category".into(),
                action: format!("/admin/categories/{}", id),
                name: form.name,
                slug,
                error: Some(msg),
            })
        }
    }
}

async fn delete_category(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Category::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/categories").into_response())
}

// ─── Tag handlers ─────────────────────────────────────────────────────────────

async fn list_tags(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let tags = Tag::find_all_with_count(&pool).await?;
    render(TagListTemplate { tags })
}

async fn delete_tag(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Tag::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/tags").into_response())
}

// ─── Settings ─────────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct SettingsForm {
    post_url_type: Option<String>,
    site_name: Option<String>,
    site_description: Option<String>,
    logo_url: Option<String>,
    active_theme: Option<String>,
}

async fn settings_page(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (post_url_type, site_name, site_description, logo_url, active_theme, robots_txt) = tokio::join!(
        Setting::post_url_type(&pool),
        Setting::site_name(&pool),
        Setting::site_description(&pool),
        Setting::logo_url(&pool),
        Setting::active_theme(&pool),
        Setting::get(&pool, "robots_txt"),
    );
    render(SettingsTemplate {
        post_url_type, site_name, site_description, logo_url, active_theme,
        robots_txt: robots_txt.unwrap_or_default(),
        saved: None,
    })
}

async fn settings_save(
    State(pool): State<DbPool>,
    Form(form): Form<SettingsForm>,
) -> Result<Response, AppError> {
    let url_type = match form.post_url_type.as_deref() {
        Some("id") => "id",
        _ => "slug",
    };
    let site_name = form.site_name.as_deref().unwrap_or("").trim().to_string();
    let site_name = if site_name.is_empty() { "RedLeaf CMS".to_string() } else { site_name };
    let site_description = form.site_description.as_deref().unwrap_or("").trim().to_string();
    let logo_url = form.logo_url.as_deref().unwrap_or("").trim().to_string();
    let active_theme = form.active_theme.as_deref().unwrap_or("default").trim().to_string();
    let active_theme = if active_theme.is_empty() { "default".to_string() } else { active_theme };

    tokio::try_join!(
        Setting::set(&pool, "post_url_type", url_type),
        Setting::set(&pool, "site_name", &site_name),
        Setting::set(&pool, "site_description", &site_description),
        Setting::set(&pool, "logo_url", &logo_url),
        Setting::set(&pool, "active_theme", &active_theme),
    )?;

    let robots_txt = Setting::get(&pool, "robots_txt").await.unwrap_or_default();
    render(SettingsTemplate {
        post_url_type: url_type.to_string(),
        site_name,
        site_description,
        logo_url,
        active_theme,
        robots_txt,
        saved: Some("Settings saved.".to_string()),
    })
}

#[derive(Deserialize)]
struct RobotsTxtForm {
    robots_txt: Option<String>,
}

async fn save_robots_txt(
    State(pool): State<DbPool>,
    Form(form): Form<RobotsTxtForm>,
) -> Result<Response, AppError> {
    let value = form.robots_txt.as_deref().unwrap_or("").trim();
    Setting::set(&pool, "robots_txt", value).await?;
    Ok(Redirect::to("/admin/settings").into_response())
}

// ─── Media handlers ───────────────────────────────────────────────────────────

const UPLOAD_DIR: &str = "static/uploads";

fn generate_filename(original: &str) -> String {
    let ext = std::path::Path::new(original)
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("bin")
        .to_lowercase();
    let ts = chrono::Utc::now().timestamp_millis();
    let rand: u64 = rand::random();
    format!("{ts}_{rand:016x}.{ext}")
}

async fn list_media(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let media = Media::find_all(&pool).await?;
    render(MediaListTemplate { media, error: None })
}

async fn upload_media(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    mut multipart: Multipart,
) -> Result<Response, AppError> {
    tokio::fs::create_dir_all(UPLOAD_DIR)
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?;

    let mut uploaded = false;

    while let Some(field) = multipart
        .next_field()
        .await
        .map_err(|e| AppError::Internal(e.to_string()))?
    {
        if field.name() != Some("file") {
            continue;
        }

        let original_name = field
            .file_name()
            .unwrap_or("upload")
            .to_string();
        let mime_type = field
            .content_type()
            .unwrap_or("application/octet-stream")
            .to_string();

        let bytes = field
            .bytes()
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        if bytes.is_empty() {
            let media = Media::find_all(&pool).await?;
            return render(MediaListTemplate {
                media,
                error: Some("No file was provided.".into()),
            });
        }

        let filename = generate_filename(&original_name);
        let path = format!("{UPLOAD_DIR}/{filename}");

        tokio::fs::write(&path, &bytes)
            .await
            .map_err(|e| AppError::Internal(e.to_string()))?;

        let url = format!("/static/uploads/{filename}");
        Media::create(
            &pool,
            &filename,
            &original_name,
            &mime_type,
            bytes.len() as i64,
            &url,
            Some(claims.sub),
        )
        .await?;

        uploaded = true;
        break;
    }

    if !uploaded {
        let media = Media::find_all(&pool).await?;
        return render(MediaListTemplate {
            media,
            error: Some("No file was provided.".into()),
        });
    }

    Ok(Redirect::to("/admin/media").into_response())
}

async fn delete_media_handler(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    if let Some(media) = Media::delete(&pool, id).await? {
        let path = format!("{UPLOAD_DIR}/{}", media.filename);
        tokio::fs::remove_file(&path).await.ok();
    }
    Ok(Redirect::to("/admin/media").into_response())
}

// ─── Page handlers ────────────────────────────────────────────────────────────

#[derive(Debug, Deserialize)]
struct PageFormInput {
    title: String,
    slug: Option<String>,
    content: String,
    status: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    parent_id: Option<i64>,
}

async fn list_pages(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let pages = Page::find_all(&pool).await?;
    render(PageListTemplate { pages })
}

async fn new_page_form(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (all_pages, media) = tokio::join!(Page::find_all(&pool), Media::find_all(&pool));
    let media_images = media?.into_iter().filter(|m| m.is_image()).collect();
    render(PageFormTemplate {
        heading: "New Page".into(),
        action: "/admin/pages".into(),
        title: String::new(),
        slug: String::new(),
        content: String::new(),
        status: "draft".into(),
        parent_id: None,
        all_pages: all_pages?,
        error: None,
        media_images,
    })
}

async fn create_page(
    State(pool): State<DbPool>,
    Form(form): Form<PageFormInput>,
) -> Result<Response, AppError> {
    let slug = match form.slug.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(s) => s.trim().to_string(),
        None => slugify(&form.title),
    };
    let payload = CreatePage {
        title: form.title,
        slug,
        content: form.content,
        status: form.status.unwrap_or_else(|| "draft".into()),
        parent_id: form.parent_id,
    };
    match Page::create(&pool, payload).await {
        Ok(_) => Ok(Redirect::to("/admin/pages").into_response()),
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Slug already exists.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            let (all_pages, media) = tokio::join!(Page::find_all(&pool), Media::find_all(&pool));
            let media_images = media?.into_iter().filter(|m| m.is_image()).collect();
            render(PageFormTemplate {
                heading: "New Page".into(),
                action: "/admin/pages".into(),
                title: String::new(),
                slug: String::new(),
                content: String::new(),
                status: "draft".into(),
                parent_id: None,
                all_pages: all_pages?,
                error: Some(msg),
                media_images,
            })
        }
    }
}

async fn edit_page_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let (page, all_pages, media) = tokio::join!(
        Page::find_by_id(&pool, id),
        Page::find_all(&pool),
        Media::find_all(&pool),
    );
    let page = page?.ok_or(AppError::NotFound)?;
    let media_images = media?.into_iter().filter(|m| m.is_image()).collect();
    render(PageFormTemplate {
        heading: "Edit Page".into(),
        action: format!("/admin/pages/{}", id),
        title: page.title,
        slug: page.slug,
        content: page.content,
        status: page.status,
        parent_id: page.parent_id,
        all_pages: all_pages?,
        error: None,
        media_images,
    })
}

async fn update_page_handler(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<PageFormInput>,
) -> Result<Response, AppError> {
    let slug = match form.slug.as_deref().filter(|s| !s.trim().is_empty()) {
        Some(s) => Some(s.trim().to_string()),
        None => Some(slugify(&form.title)),
    };
    let payload = UpdatePage {
        title: Some(form.title),
        slug,
        content: Some(form.content),
        status: form.status,
        parent_id: Some(form.parent_id),
    };
    Page::update(&pool, id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;
    Ok(Redirect::to("/admin/pages").into_response())
}

async fn delete_page_handler(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Page::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/pages").into_response())
}

// ─── Revision handlers ────────────────────────────────────────────────────────

async fn list_revisions(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let post = Post::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    let revisions = PostRevision::find_by_post(&pool, id).await?;
    render(RevisionListTemplate {
        post_id: id,
        post_title: post.title,
        revisions,
    })
}

async fn restore_revision(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path((post_id, rev_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    let rev = PostRevision::find_by_id(&pool, rev_id).await?.ok_or(AppError::NotFound)?;
    // Save current state as revision before restoring
    if let Ok(Some(cur)) = Post::find_by_id(&pool, post_id).await {
        PostRevision::save(&pool, post_id, &cur.title, &cur.content, cur.excerpt.as_deref(), Some(claims.sub)).await?;
    }
    let payload = UpdatePost {
        title: Some(rev.title),
        slug: None,
        content: Some(rev.content),
        excerpt: Some(rev.excerpt),
        published: None,
        sticky: None,
        category_id: None,
        featured_image_id: None,
        scheduled_at: None,
        seo_title: None,
        seo_description: None,
    };
    Post::update(&pool, post_id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;
    Ok(Redirect::to(&format!("/admin/posts/{}/edit", post_id)).into_response())
}

// ─── Widget handlers ──────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/widgets/index.html")]
struct WidgetListTemplate {
    areas_with_widgets: Vec<(WidgetArea, Vec<Widget>)>,
}

async fn list_widgets(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let areas = WidgetArea::find_all(&pool).await?;
    let mut areas_with_widgets = Vec::new();
    for area in areas {
        let widgets = Widget::find_by_area(&pool, area.id).await?;
        areas_with_widgets.push((area, widgets));
    }
    render(WidgetListTemplate { areas_with_widgets })
}

#[derive(Deserialize)]
struct WidgetForm {
    area_id: Option<i64>,
    widget_type: Option<String>,
    title: Option<String>,
    count: Option<i64>,
    text_content: Option<String>,
}

async fn create_widget(
    State(pool): State<DbPool>,
    Form(form): Form<WidgetForm>,
) -> Result<Response, AppError> {
    let area_id = form.area_id.ok_or_else(|| AppError::Internal("area_id required".into()))?;
    let widget_type = form.widget_type.as_deref().unwrap_or("text");
    let title = form.title.as_deref().unwrap_or("").trim().to_owned();
    let settings = build_settings(widget_type, &form);
    Widget::create(&pool, area_id, widget_type, &title, &settings).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

async fn update_widget(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<WidgetForm>,
) -> Result<Response, AppError> {
    let widget = Widget::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    let title = form.title.as_deref().unwrap_or("").trim().to_owned();
    let settings = build_settings(&widget.widget_type, &form);
    Widget::update(&pool, id, &title, &settings).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

fn build_settings(widget_type: &str, form: &WidgetForm) -> String {
    match widget_type {
        "recent_posts" => {
            let count = form.count.unwrap_or(5);
            format!(r#"{{"count":{count}}}"#)
        }
        "text" => {
            let content = form.text_content.as_deref().unwrap_or("");
            serde_json::json!({"content": content}).to_string()
        }
        _ => "{}".to_string(),
    }
}

async fn delete_widget(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Widget::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

#[derive(Deserialize)]
struct ReorderForm {
    #[serde(rename = "ids[]")]
    ids: Vec<i64>,
}

async fn reorder_widgets(
    State(pool): State<DbPool>,
    Form(form): Form<ReorderForm>,
) -> Result<Response, AppError> {
    Widget::reorder(&pool, &form.ids).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

#[derive(Deserialize)]
struct WidgetAreaForm {
    name: String,
    slug: String,
}

async fn create_widget_area(
    State(pool): State<DbPool>,
    Form(form): Form<WidgetAreaForm>,
) -> Result<Response, AppError> {
    WidgetArea::create(&pool, &form.name, &form.slug).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

async fn delete_widget_area(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    WidgetArea::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/widgets").into_response())
}

// ─── Nav menu handlers ────────────────────────────────────────────────────────

#[derive(Template)]
#[template(path = "admin/menus/list.html")]
struct MenuListTemplate {
    menus: Vec<(NavMenu, i64)>,
    locations: &'static [(&'static str, &'static str)],
    assigned_locations: Vec<String>,
}

impl MenuListTemplate {
    fn is_assigned(&self, slug: &str) -> bool {
        self.assigned_locations.iter().any(|l| l == slug)
    }
}

#[derive(Template)]
#[template(path = "admin/menus/edit.html")]
struct MenuEditTemplate {
    heading: String,
    action: String,
    name: String,
    location: String,
    menu_id: Option<i64>,
    items: Vec<NavMenuItem>,
    pages: Vec<Page>,
    categories: Vec<Category>,
    locations: &'static [(&'static str, &'static str)],
    error: Option<String>,
}

async fn list_menus(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let all = NavMenu::find_all(&pool).await?;
    let mut menus = Vec::new();
    for menu in all {
        let count: i64 = sqlx::query_scalar(
            "SELECT COUNT(*) FROM nav_menu_items WHERE menu_id = ?",
        )
        .bind(menu.id)
        .fetch_one(&pool)
        .await?;
        menus.push((menu, count));
    }
    let assigned_locations: Vec<String> = menus
        .iter()
        .map(|(m, _)| m.location.clone())
        .filter(|l| !l.is_empty())
        .collect();
    render(MenuListTemplate { menus, locations: LOCATIONS, assigned_locations })
}

async fn new_menu_form() -> Result<Response, AppError> {
    render(MenuEditTemplate {
        heading: "New Menu".into(),
        action: "/admin/menus/new".into(),
        name: String::new(),
        location: String::new(),
        menu_id: None,
        items: vec![],
        pages: vec![],
        categories: vec![],
        locations: LOCATIONS,
        error: None,
    })
}

#[derive(Deserialize)]
struct MenuForm {
    name: String,
    location: Option<String>,
}

async fn create_menu(
    State(pool): State<DbPool>,
    Form(form): Form<MenuForm>,
) -> Result<Response, AppError> {
    let loc = form.location.as_deref().unwrap_or("").trim().to_owned();
    let menu = NavMenu::create(&pool, form.name.trim(), &loc).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", menu.id)).into_response())
}

async fn edit_menu_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let menu = NavMenu::find_by_id(&pool, id).await?.ok_or(AppError::NotFound)?;
    let (items, pages, categories) = tokio::join!(
        NavMenuItem::find_by_menu(&pool, id),
        Page::find_all(&pool),
        Category::find_all(&pool),
    );
    render(MenuEditTemplate {
        heading: format!("Edit — {}", menu.name),
        action: format!("/admin/menus/{}/edit", id),
        name: menu.name,
        location: menu.location,
        menu_id: Some(id),
        items: items?,
        pages: pages?,
        categories: categories?,
        locations: LOCATIONS,
        error: None,
    })
}

async fn update_menu(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<MenuForm>,
) -> Result<Response, AppError> {
    let loc = form.location.as_deref().unwrap_or("").trim().to_owned();
    NavMenu::update(&pool, id, form.name.trim(), &loc).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", id)).into_response())
}

async fn delete_menu(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    NavMenu::delete(&pool, id).await?;
    Ok(Redirect::to("/admin/menus").into_response())
}

#[derive(Deserialize)]
struct MenuItemForm {
    #[serde(default)]
    _item_id: String,
    item_type: Option<String>,
    // custom
    label: Option<String>,
    url: Option<String>,
    // page
    ref_id_page: Option<i64>,
    label_page: Option<String>,
    // category
    ref_id_cat: Option<i64>,
    label_cat: Option<String>,
    #[serde(default, deserialize_with = "empty_string_as_none")]
    parent_id: Option<i64>,
}

async fn create_menu_item(
    State(pool): State<DbPool>,
    Path(menu_id): Path<i64>,
    Form(form): Form<MenuItemForm>,
) -> Result<Response, AppError> {
    let (item_type, label, url, ref_id) = resolve_item_fields(&pool, &form).await;
    NavMenuItem::create(&pool, menu_id, form.parent_id, &item_type, &label, &url, ref_id).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", menu_id)).into_response())
}

async fn update_menu_item(
    State(pool): State<DbPool>,
    Path((menu_id, item_id)): Path<(i64, i64)>,
    Form(form): Form<MenuItemForm>,
) -> Result<Response, AppError> {
    let label = form.label.as_deref().unwrap_or("").trim().to_owned();
    let url = form.url.as_deref().unwrap_or("").trim().to_owned();
    NavMenuItem::update(&pool, item_id, form.parent_id, &label, &url).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", menu_id)).into_response())
}

async fn delete_menu_item(
    State(pool): State<DbPool>,
    Path((menu_id, item_id)): Path<(i64, i64)>,
) -> Result<Response, AppError> {
    NavMenuItem::delete(&pool, item_id).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", menu_id)).into_response())
}

#[derive(Deserialize)]
struct ReorderItemsForm {
    #[serde(rename = "ids[]")]
    ids: Vec<i64>,
}

async fn reorder_menu_items(
    State(pool): State<DbPool>,
    Path(menu_id): Path<i64>,
    Form(form): Form<ReorderItemsForm>,
) -> Result<Response, AppError> {
    NavMenuItem::reorder(&pool, &form.ids).await?;
    Ok(Redirect::to(&format!("/admin/menus/{}/edit", menu_id)).into_response())
}

async fn resolve_item_fields(pool: &DbPool, form: &MenuItemForm) -> (String, String, String, Option<i64>) {
    match form.item_type.as_deref().unwrap_or("custom") {
        "page" => {
            if let Some(ref_id) = form.ref_id_page {
                if let Ok(Some(page)) = Page::find_by_id(pool, ref_id).await {
                    let label = form.label_page.as_deref().unwrap_or("").trim().to_owned();
                    let label = if label.is_empty() { page.title.clone() } else { label };
                    let url = format!("/pages/{}", page.slug);
                    return ("page".into(), label, url, Some(ref_id));
                }
            }
            ("custom".into(), String::new(), String::new(), None)
        }
        "category" => {
            if let Some(ref_id) = form.ref_id_cat {
                if let Ok(Some(cat)) = Category::find_by_id(pool, ref_id).await {
                    let label = form.label_cat.as_deref().unwrap_or("").trim().to_owned();
                    let label = if label.is_empty() { cat.name.clone() } else { label };
                    let url = format!("/categories/{}", cat.slug);
                    return ("category".into(), label, url, Some(ref_id));
                }
            }
            ("custom".into(), String::new(), String::new(), None)
        }
        _ => {
            let label = form.label.as_deref().unwrap_or("").trim().to_owned();
            let url = form.url.as_deref().unwrap_or("").trim().to_owned();
            ("custom".into(), label, url, None)
        }
    }
}

// ─── Comment moderation handlers ─────────────────────────────────────────────

async fn list_comments(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let (comments, pending_count) = tokio::join!(
        Comment::find_all_admin(&pool),
        Comment::count_pending(&pool),
    );
    render(CommentListTemplate { comments: comments?, pending_count: pending_count? })
}

async fn approve_comment(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Comment::approve(&pool, id).await?;
    Ok(Redirect::to("/admin/comments").into_response())
}

async fn reject_comment(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Comment::reject(&pool, id).await?;
    Ok(Redirect::to("/admin/comments").into_response())
}

async fn spam_comment(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    Comment::mark_spam(&pool, id).await?;
    Ok(Redirect::to("/admin/comments").into_response())
}

// ─── User / role handlers ─────────────────────────────────────────────────────

const ALL_ROLES: &[&str] = &["administrator", "editor", "author", "contributor", "subscriber"];

async fn list_users(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let users = User::find_all(&pool).await?;
    let roles: Vec<String> = ALL_ROLES.iter().map(|s| s.to_string()).collect();
    render(UserListTemplate { users, roles })
}

#[derive(Deserialize)]
struct RoleForm {
    role: String,
}

async fn update_user_role(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Path(id): Path<i64>,
    Form(form): Form<RoleForm>,
) -> Result<Response, AppError> {
    if !has_capability(&claims.role, "manage_users") {
        return Err(AppError::Internal("Forbidden".into()));
    }
    let role = if ALL_ROLES.contains(&form.role.as_str()) { form.role.as_str() } else { "subscriber" };
    User::update_role(&pool, id, role).await?;
    Ok(Redirect::to("/admin/users").into_response())
}

// ─── Profile handlers ─────────────────────────────────────────────────────────

async fn edit_profile_form(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
) -> Result<Response, AppError> {
    let user = User::find_by_id(&pool, claims.sub).await?.ok_or(AppError::NotFound)?;
    render(ProfileTemplate {
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        bio: user.bio,
        website: user.website,
        avatar_url: user.avatar_url,
        saved: None,
        error: None,
        pw_saved: None,
        pw_error: None,
    })
}

#[derive(Deserialize)]
struct ProfileForm {
    display_name: Option<String>,
    bio: Option<String>,
    website: Option<String>,
    avatar_url: Option<String>,
}

async fn update_profile(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<ProfileForm>,
) -> Result<Response, AppError> {
    let profile = UpdateProfile {
        display_name: form.display_name.unwrap_or_default().trim().to_string(),
        bio: form.bio.unwrap_or_default().trim().to_string(),
        website: form.website.unwrap_or_default().trim().to_string(),
        avatar_url: form.avatar_url.unwrap_or_default().trim().to_string(),
    };
    let user = User::update_profile(&pool, claims.sub, profile).await?;
    render(ProfileTemplate {
        username: user.username,
        email: user.email,
        display_name: user.display_name,
        bio: user.bio,
        website: user.website,
        avatar_url: user.avatar_url,
        saved: Some("Profile saved.".into()),
        error: None,
        pw_saved: None,
        pw_error: None,
    })
}

#[derive(Deserialize)]
struct PasswordForm {
    current_password: String,
    new_password: String,
    confirm_password: String,
}

async fn change_password(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<PasswordForm>,
) -> Result<Response, AppError> {
    let user = User::find_by_id(&pool, claims.sub).await?.ok_or(AppError::NotFound)?;

    let profile_data = ProfileTemplate {
        username: user.username.clone(),
        email: user.email.clone(),
        display_name: user.display_name.clone(),
        bio: user.bio.clone(),
        website: user.website.clone(),
        avatar_url: user.avatar_url.clone(),
        saved: None,
        error: None,
        pw_saved: None,
        pw_error: None,
    };

    if form.new_password != form.confirm_password {
        return render(ProfileTemplate { pw_error: Some("Passwords do not match.".into()), ..profile_data });
    }
    if form.new_password.len() < 8 {
        return render(ProfileTemplate { pw_error: Some("Password must be at least 8 characters.".into()), ..profile_data });
    }
    if !User::verify_password(&form.current_password, &user.password_hash)
        .unwrap_or(false)
    {
        return render(ProfileTemplate { pw_error: Some("Current password is incorrect.".into()), ..profile_data });
    }

    User::change_password(&pool, claims.sub, &form.new_password).await
        .map_err(|e| AppError::Internal(e.to_string()))?;
    render(ProfileTemplate { pw_saved: Some("Password changed.".into()), ..profile_data })
}