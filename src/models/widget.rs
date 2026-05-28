use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct WidgetArea {
    pub id: i64,
    pub name: String,
    pub slug: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Widget {
    pub id: i64,
    pub area_id: i64,
    pub widget_type: String,
    pub title: String,
    pub settings: String,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
}

/// Widget with its area slug — used in admin list views.
#[derive(Debug, Clone, FromRow)]
pub struct WidgetWithArea {
    pub id: i64,
    pub area_id: i64,
    pub area_slug: String,
    pub area_name: String,
    pub widget_type: String,
    pub title: String,
    pub settings: String,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
}

impl WidgetArea {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<WidgetArea>, sqlx::Error> {
        sqlx::query_as::<_, WidgetArea>(
            "SELECT * FROM widget_areas ORDER BY name ASC",
        )
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_slug(pool: &DbPool, slug: &str) -> Result<Option<WidgetArea>, sqlx::Error> {
        sqlx::query_as::<_, WidgetArea>("SELECT * FROM widget_areas WHERE slug = ?")
            .bind(slug)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(pool: &DbPool, name: &str, slug: &str) -> Result<WidgetArea, sqlx::Error> {
        sqlx::query_as::<_, WidgetArea>(
            "INSERT INTO widget_areas (name, slug) VALUES (?, ?) RETURNING *",
        )
        .bind(name)
        .bind(slug)
        .fetch_one(pool)
        .await
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM widget_areas WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }
}

impl Widget {
    /// Returns the `count` setting for `recent_posts` widgets (default 5).
    pub fn count_setting(&self) -> i64 {
        serde_json::from_str::<serde_json::Value>(&self.settings)
            .ok()
            .and_then(|v| v.get("count").and_then(|c| c.as_i64()))
            .unwrap_or(5)
    }

    /// Returns the `content` setting for `text` widgets.
    pub fn text_setting(&self) -> String {
        serde_json::from_str::<serde_json::Value>(&self.settings)
            .ok()
            .and_then(|v| v.get("content").and_then(|c| c.as_str()).map(|s| s.to_owned()))
            .unwrap_or_default()
    }

    /// One-line summary of widget settings for admin display.
    pub fn settings_summary(&self) -> String {
        match self.widget_type.as_str() {
            "recent_posts" => format!("Count: {}", self.count_setting()),
            "text" => {
                let c = self.text_setting();
                if c.is_empty() { "(no content)".to_owned() } else { format!("{}…", c.chars().take(40).collect::<String>()) }
            }
            _ => String::new(),
        }
    }

    pub async fn find_by_area(pool: &DbPool, area_id: i64) -> Result<Vec<Widget>, sqlx::Error> {
        sqlx::query_as::<_, Widget>(
            "SELECT * FROM widgets WHERE area_id = ? ORDER BY sort_order ASC",
        )
        .bind(area_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<Widget>, sqlx::Error> {
        sqlx::query_as::<_, Widget>("SELECT * FROM widgets WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(
        pool: &DbPool,
        area_id: i64,
        widget_type: &str,
        title: &str,
        settings: &str,
    ) -> Result<Widget, sqlx::Error> {
        let max_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) FROM widgets WHERE area_id = ?",
        )
        .bind(area_id)
        .fetch_one(pool)
        .await?;

        sqlx::query_as::<_, Widget>(
            r#"INSERT INTO widgets (area_id, widget_type, title, settings, sort_order)
               VALUES (?, ?, ?, ?, ?) RETURNING *"#,
        )
        .bind(area_id)
        .bind(widget_type)
        .bind(title)
        .bind(settings)
        .bind(max_order + 1)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &DbPool,
        id: i64,
        title: &str,
        settings: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE widgets SET title = ?, settings = ? WHERE id = ?")
            .bind(title)
            .bind(settings)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM widgets WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Update sort_order for a list of widget IDs in the given order.
    pub async fn reorder(pool: &DbPool, ordered_ids: &[i64]) -> Result<(), sqlx::Error> {
        for (i, &id) in ordered_ids.iter().enumerate() {
            sqlx::query("UPDATE widgets SET sort_order = ? WHERE id = ?")
                .bind(i as i64)
                .bind(id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }

    /// Pre-render all widget areas to HTML strings. Returns a map of slug → HTML.
    pub async fn prerender_all(pool: &DbPool) -> HashMap<String, String> {
        let areas = match WidgetArea::find_all(pool).await {
            Ok(a) => a,
            Err(_) => return HashMap::new(),
        };
        let mut map = HashMap::new();
        for area in areas {
            let html = Self::render_area_by_id(pool, area.id).await;
            map.insert(area.slug, html);
        }
        map
    }

    /// Render all widgets in an area (by slug) to a single HTML string.
    pub async fn render_area(pool: &DbPool, slug: &str) -> String {
        let area = match WidgetArea::find_by_slug(pool, slug).await {
            Ok(Some(a)) => a,
            _ => return String::new(),
        };
        Self::render_area_by_id(pool, area.id).await
    }

    async fn render_area_by_id(pool: &DbPool, area_id: i64) -> String {
        let widgets = match Self::find_by_area(pool, area_id).await {
            Ok(w) => w,
            Err(_) => return String::new(),
        };
        let mut html = String::new();
        for w in &widgets {
            html.push_str(&Self::render_widget(pool, w).await);
        }
        html
    }

    async fn render_widget(pool: &DbPool, widget: &Widget) -> String {
        let title_html = if widget.title.is_empty() {
            String::new()
        } else {
            let escaped = escape_html(&widget.title);
            format!("<h3 class=\"widget-title\">{escaped}</h3>")
        };

        let body = match widget.widget_type.as_str() {
            "recent_posts" => render_recent_posts(pool, &widget.settings).await,
            "categories" => render_categories(pool).await,
            "tag_cloud" => render_tag_cloud(pool).await,
            "text" => render_text(&widget.settings),
            "search" => render_search(),
            _ => String::new(),
        };

        format!(
            "<section class=\"widget widget-{wtype}\">{title}{body}</section>\n",
            wtype = escape_html(&widget.widget_type),
            title = title_html,
            body = body,
        )
    }
}

// ─── Per-type renderers ───────────────────────────────────────────────────────

async fn render_recent_posts(pool: &DbPool, settings_json: &str) -> String {
    let count: i64 = serde_json::from_str::<serde_json::Value>(settings_json)
        .ok()
        .and_then(|v| v.get("count").and_then(|c| c.as_i64()))
        .unwrap_or(5);

    let posts = match crate::models::Post::find_recent_with_author(pool, count).await {
        Ok(p) => p,
        Err(_) => return String::new(),
    };

    let mut html = String::from("<ul class=\"widget-list\">");
    for post in &posts {
        let escaped_title = escape_html(&post.title);
        html.push_str(&format!(
            "<li><a href=\"/posts/{slug}\">{title}</a></li>",
            slug = escape_html(&post.slug),
            title = escaped_title,
        ));
    }
    html.push_str("</ul>");
    html
}

async fn render_categories(pool: &DbPool) -> String {
    use crate::models::Category;
    let cats = match Category::find_all_with_count(pool).await {
        Ok(c) => c,
        Err(_) => return String::new(),
    };

    let mut html = String::from("<ul class=\"widget-list\">");
    for cat in &cats {
        html.push_str(&format!(
            "<li><a href=\"/categories/{slug}\">{name} ({count})</a></li>",
            slug = escape_html(&cat.slug),
            name = escape_html(&cat.name),
            count = cat.post_count,
        ));
    }
    html.push_str("</ul>");
    html
}

async fn render_tag_cloud(pool: &DbPool) -> String {
    use crate::models::Tag;
    let tags = match Tag::find_all_with_count(pool).await {
        Ok(t) => t,
        Err(_) => return String::new(),
    };

    let mut html = String::from("<div class=\"widget-tags\">");
    for tag in &tags {
        html.push_str(&format!(
            "<a href=\"/tags/{slug}\" class=\"widget-tag\">{name}</a> ",
            slug = escape_html(&tag.slug),
            name = escape_html(&tag.name),
        ));
    }
    html.push_str("</div>");
    html
}

fn render_text(settings_json: &str) -> String {
    let content = serde_json::from_str::<serde_json::Value>(settings_json)
        .ok()
        .and_then(|v| v.get("content").and_then(|c| c.as_str()).map(|s| s.to_owned()))
        .unwrap_or_default();
    format!("<div class=\"widget-text\">{}</div>", content)
}

fn render_search() -> String {
    r#"<form action="/search" method="get" class="widget-search">
  <input type="text" name="q" placeholder="Search…" style="width:100%;padding:.4rem .6rem;border:1px solid #d1d5db;border-radius:4px;">
  <button type="submit" style="margin-top:.4rem;width:100%;padding:.4rem;background:#2d5f3e;color:white;border:none;border-radius:4px;cursor:pointer;">Search</button>
</form>"#.to_string()
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}