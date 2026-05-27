use askama::Template;
use axum::{
    extract::{Extension, Form, Multipart, Path, Query, State},
    http::{StatusCode, header},
    response::{IntoResponse, Redirect, Response},
    routing::{get, post},
    Router,
};
use serde::Deserialize;

use crate::{
    auth::{generate_token, Claims},
    db::DbPool,
    errors::AppError,
    models::{
        Category, CategoryWithCount,
        Media,
        Post, PostWithAuthor,
        Setting,
        Tag, TagWithCount,
        post::{CreatePost, UpdatePost},
        user::{CreateUser, LoginUser},
        User,
    },
    util::{render, slugify, Pagination, PER_PAGE},
};

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
    title: String,
    slug: String,
    content: String,
    excerpt: String,
    published: bool,
    error: Option<String>,
    categories: Vec<Category>,
    selected_category_id: Option<i64>,
    tag_names: String,
}

impl PostFormTemplate {
    fn new(action: impl Into<String>, heading: impl Into<String>) -> Self {
        Self {
            heading: heading.into(),
            action: action.into(),
            title: String::new(),
            slug: String::new(),
            content: String::new(),
            excerpt: String::new(),
            published: false,
            error: None,
            categories: vec![],
            selected_category_id: None,
            tag_names: String::new(),
        }
    }

    fn from_post(post: &Post) -> Self {
        Self {
            heading: "Edit Post".into(),
            action: format!("/admin/posts/{}", post.id),
            title: post.title.clone(),
            slug: post.slug.clone(),
            content: post.content.clone(),
            excerpt: post.excerpt.clone().unwrap_or_default(),
            published: post.published,
            error: None,
            categories: vec![],
            selected_category_id: post.category_id,
            tag_names: String::new(),
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
    category_id: Option<i64>,
    tags: Option<String>,
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
        .route("/media", get(list_media))
        .route("/media/upload", post(upload_media))
        .route("/media/{id}/delete", post(delete_media_handler))
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

fn form_to_create(form: PostForm) -> CreatePost {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };
    CreatePost {
        title: form.title,
        slug,
        content: form.content,
        excerpt: form.excerpt.filter(|s| !s.trim().is_empty()),
        published: form.published.is_some(),
        author_id: None,
        category_id: form.category_id,
    }
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
    let categories = Category::find_all(&pool).await?;
    let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
    tmpl.categories = categories;
    render(tmpl)
}

async fn create_post(
    State(pool): State<DbPool>,
    Extension(claims): Extension<Claims>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let tag_input = form.tags.clone();
    let mut payload = form_to_create(form);
    payload.author_id = Some(claims.sub);

    match Post::create(&pool, payload).await {
        Ok(post) => {
            let tag_ids = resolve_tags(&pool, tag_input.as_deref()).await?;
            Post::set_tags(&pool, post.id, &tag_ids).await?;
            Ok(Redirect::to("/admin/posts").into_response())
        }
        Err(e) => {
            let msg = if e.to_string().contains("UNIQUE constraint failed") {
                "Slug already exists. Please choose a different slug.".to_string()
            } else {
                return Err(AppError::Database(e));
            };
            let categories = Category::find_all(&pool).await?;
            let mut tmpl = PostFormTemplate::new("/admin/posts", "New Post");
            tmpl.error = Some(msg);
            tmpl.categories = categories;
            render(tmpl)
        }
    }
}

async fn edit_post_form(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
) -> Result<Response, AppError> {
    let (post_res, categories) = tokio::join!(
        Post::find_by_id(&pool, id),
        Category::find_all(&pool),
    );
    let post = post_res?.ok_or(AppError::NotFound)?;
    let tags = Tag::find_by_post(&pool, id).await?;
    let tag_names = tags.iter().map(|t| t.name.as_str()).collect::<Vec<_>>().join(", ");
    let mut tmpl = PostFormTemplate::from_post(&post);
    tmpl.categories = categories?;
    tmpl.tag_names = tag_names;
    render(tmpl)
}

async fn update_post(
    State(pool): State<DbPool>,
    Path(id): Path<i64>,
    Form(form): Form<PostForm>,
) -> Result<Response, AppError> {
    let slug = if form.slug.trim().is_empty() {
        slugify(&form.title)
    } else {
        form.slug.trim().to_string()
    };

    let payload = UpdatePost {
        title: Some(form.title),
        slug: Some(slug),
        content: Some(form.content),
        excerpt: Some(form.excerpt.filter(|s| !s.trim().is_empty())),
        published: Some(form.published.is_some()),
        category_id: Some(form.category_id),
    };

    let post = Post::update(&pool, id, payload).await.map_err(|e| match e {
        sqlx::Error::RowNotFound => AppError::NotFound,
        other => AppError::Database(other),
    })?;
    let tag_ids = resolve_tags(&pool, form.tags.as_deref()).await?;
    Post::set_tags(&pool, post.id, &tag_ids).await?;
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
        category_id: None,
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
}

async fn settings_page(State(pool): State<DbPool>) -> Result<Response, AppError> {
    let post_url_type = Setting::post_url_type(&pool).await;
    render(SettingsTemplate { post_url_type, saved: None })
}

async fn settings_save(
    State(pool): State<DbPool>,
    Form(form): Form<SettingsForm>,
) -> Result<Response, AppError> {
    let url_type = match form.post_url_type.as_deref() {
        Some("id") => "id",
        _ => "slug",
    };
    Setting::set(&pool, "post_url_type", url_type).await?;
    render(SettingsTemplate {
        post_url_type: url_type.to_string(),
        saved: Some("Settings saved.".to_string()),
    })
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