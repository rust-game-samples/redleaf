use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;
use std::collections::HashMap;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NavMenu {
    pub id: i64,
    pub name: String,
    pub location: String,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct NavMenuItem {
    pub id: i64,
    pub menu_id: i64,
    pub parent_id: Option<i64>,
    pub item_type: String,
    pub label: String,
    pub url: String,
    pub ref_id: Option<i64>,
    pub sort_order: i64,
    pub created_at: DateTime<Utc>,
}

/// Known menu locations registered by the theme.
pub const LOCATIONS: &[(&str, &str)] = &[
    ("primary", "Primary Navigation"),
    ("footer",  "Footer Navigation"),
    ("social",  "Social Links"),
];

impl NavMenu {
    pub async fn find_all(pool: &DbPool) -> Result<Vec<NavMenu>, sqlx::Error> {
        sqlx::query_as::<_, NavMenu>("SELECT * FROM nav_menus ORDER BY name ASC")
            .fetch_all(pool)
            .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<NavMenu>, sqlx::Error> {
        sqlx::query_as::<_, NavMenu>("SELECT * FROM nav_menus WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_location(pool: &DbPool, loc: &str) -> Result<Option<NavMenu>, sqlx::Error> {
        sqlx::query_as::<_, NavMenu>("SELECT * FROM nav_menus WHERE location = ? LIMIT 1")
            .bind(loc)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(pool: &DbPool, name: &str, location: &str) -> Result<NavMenu, sqlx::Error> {
        sqlx::query_as::<_, NavMenu>(
            "INSERT INTO nav_menus (name, location) VALUES (?, ?) RETURNING *",
        )
        .bind(name)
        .bind(location)
        .fetch_one(pool)
        .await
    }

    pub async fn update(pool: &DbPool, id: i64, name: &str, location: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE nav_menus SET name = ?, location = ? WHERE id = ?")
            .bind(name)
            .bind(location)
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM nav_menus WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Pre-render all menus keyed by location. Returns location → `<ul>…</ul>` HTML.
    pub async fn prerender_all(pool: &DbPool) -> HashMap<String, String> {
        let menus = match Self::find_all(pool).await {
            Ok(m) => m,
            Err(_) => return HashMap::new(),
        };
        let mut map = HashMap::new();
        for menu in menus {
            if menu.location.is_empty() {
                continue;
            }
            let items = match NavMenuItem::find_by_menu(pool, menu.id).await {
                Ok(i) => i,
                Err(_) => continue,
            };
            let html = render_items(&items, None);
            map.insert(menu.location.clone(), html);
        }
        map
    }
}

impl NavMenuItem {
    pub async fn find_by_menu(pool: &DbPool, menu_id: i64) -> Result<Vec<NavMenuItem>, sqlx::Error> {
        sqlx::query_as::<_, NavMenuItem>(
            "SELECT * FROM nav_menu_items WHERE menu_id = ? ORDER BY sort_order ASC",
        )
        .bind(menu_id)
        .fetch_all(pool)
        .await
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<NavMenuItem>, sqlx::Error> {
        sqlx::query_as::<_, NavMenuItem>("SELECT * FROM nav_menu_items WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn create(
        pool: &DbPool,
        menu_id: i64,
        parent_id: Option<i64>,
        item_type: &str,
        label: &str,
        url: &str,
        ref_id: Option<i64>,
    ) -> Result<NavMenuItem, sqlx::Error> {
        let max_order: i64 = sqlx::query_scalar(
            "SELECT COALESCE(MAX(sort_order), -1) FROM nav_menu_items WHERE menu_id = ?",
        )
        .bind(menu_id)
        .fetch_one(pool)
        .await?;

        sqlx::query_as::<_, NavMenuItem>(
            r#"INSERT INTO nav_menu_items
               (menu_id, parent_id, item_type, label, url, ref_id, sort_order)
               VALUES (?, ?, ?, ?, ?, ?, ?) RETURNING *"#,
        )
        .bind(menu_id)
        .bind(parent_id)
        .bind(item_type)
        .bind(label)
        .bind(url)
        .bind(ref_id)
        .bind(max_order + 1)
        .fetch_one(pool)
        .await
    }

    pub async fn update(
        pool: &DbPool,
        id: i64,
        parent_id: Option<i64>,
        label: &str,
        url: &str,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "UPDATE nav_menu_items SET parent_id = ?, label = ?, url = ? WHERE id = ?",
        )
        .bind(parent_id)
        .bind(label)
        .bind(url)
        .bind(id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn delete(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM nav_menu_items WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn reorder(pool: &DbPool, ordered_ids: &[i64]) -> Result<(), sqlx::Error> {
        for (i, &id) in ordered_ids.iter().enumerate() {
            sqlx::query("UPDATE nav_menu_items SET sort_order = ? WHERE id = ?")
                .bind(i as i64)
                .bind(id)
                .execute(pool)
                .await?;
        }
        Ok(())
    }
}

// ─── HTML rendering ───────────────────────────────────────────────────────────

fn render_items(all: &[NavMenuItem], parent_id: Option<i64>) -> String {
    let children: Vec<&NavMenuItem> = all
        .iter()
        .filter(|i| i.parent_id == parent_id)
        .collect();
    if children.is_empty() {
        return String::new();
    }
    let mut html = String::from("<ul>");
    for item in children {
        html.push_str("<li>");
        html.push_str(&format!(
            r#"<a href="{}">{}</a>"#,
            escape_html(&item.url),
            escape_html(&item.label)
        ));
        html.push_str(&render_items(all, Some(item.id)));
        html.push_str("</li>");
    }
    html.push_str("</ul>");
    html
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

// ─── Breadcrumb helpers ───────────────────────────────────────────────────────

pub struct BreadcrumbItem {
    pub label: String,
    pub url: Option<String>,
}

pub fn breadcrumb_html(items: &[BreadcrumbItem]) -> String {
    let mut li_parts = String::new();
    let last_i = items.len().saturating_sub(1);
    for (i, item) in items.iter().enumerate() {
        if i == last_i {
            li_parts.push_str(&format!(
                "<li class=\"breadcrumb-item active\" aria-current=\"page\">{}</li>",
                escape_html(&item.label)
            ));
        } else {
            let href = item.url.as_deref().unwrap_or("#");
            li_parts.push_str(&format!(
                "<li class=\"breadcrumb-item\"><a href=\"{}\">{}</a></li>",
                escape_html(href),
                escape_html(&item.label)
            ));
        }
    }
    format!(
        r#"<nav aria-label="breadcrumb" class="breadcrumb-nav"><ol class="breadcrumb">{}</ol></nav>"#,
        li_parts
    )
}

pub fn breadcrumb_json_ld(items: &[BreadcrumbItem]) -> String {
    let elements: Vec<String> = items
        .iter()
        .enumerate()
        .map(|(i, item)| {
            if let Some(url) = &item.url {
                format!(
                    r#"{{"@type":"ListItem","position":{},"name":"{}","item":"{}"}}"#,
                    i + 1,
                    item.label.replace('"', "\\\""),
                    url.replace('"', "\\\""),
                )
            } else {
                format!(
                    r#"{{"@type":"ListItem","position":{},"name":"{}"}}"#,
                    i + 1,
                    item.label.replace('"', "\\\""),
                )
            }
        })
        .collect();
    format!(
        r#"<script type="application/ld+json">{{"@context":"https://schema.org","@type":"BreadcrumbList","itemListElement":[{}]}}</script>"#,
        elements.join(",")
    )
}