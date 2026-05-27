use crate::db::DbPool;

pub struct Setting;

impl Setting {
    pub async fn get(pool: &DbPool, key: &str) -> Option<String> {
        sqlx::query_scalar("SELECT value FROM settings WHERE key = ?")
            .bind(key)
            .fetch_optional(pool)
            .await
            .ok()
            .flatten()
    }

    pub async fn set(pool: &DbPool, key: &str, value: &str) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO settings (key, value, updated_at)
             VALUES (?, ?, CURRENT_TIMESTAMP)
             ON CONFLICT(key) DO UPDATE SET value = excluded.value, updated_at = excluded.updated_at",
        )
        .bind(key)
        .bind(value)
        .execute(pool)
        .await?;
        Ok(())
    }

    /// Returns "slug" or "id". Defaults to "slug" if not set.
    pub async fn post_url_type(pool: &DbPool) -> String {
        match Self::get(pool, "post_url_type").await.as_deref() {
            Some("id") => "id".to_string(),
            _ => "slug".to_string(),
        }
    }

    pub async fn site_name(pool: &DbPool) -> String {
        Self::get(pool, "site_name")
            .await
            .unwrap_or_else(|| "RedLeaf CMS".into())
    }

    pub async fn site_description(pool: &DbPool) -> String {
        Self::get(pool, "site_description")
            .await
            .unwrap_or_else(|| "A lightweight CMS built with Rust".into())
    }

    pub async fn logo_url(pool: &DbPool) -> String {
        Self::get(pool, "logo_url").await.unwrap_or_default()
    }
}