use chrono::{DateTime, Utc};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, FromRow)]
pub struct ActivityLogWithUser {
    pub id: i64,
    pub user_id: Option<i64>,
    pub action: String,
    pub target_type: Option<String>,
    pub target_id: Option<i64>,
    pub created_at: DateTime<Utc>,
    pub username: Option<String>,
}

impl ActivityLogWithUser {
    pub fn display_action(&self) -> &str {
        match self.action.as_str() {
            "post.created"       => "Created post",
            "post.updated"       => "Updated post",
            "post.deleted"       => "Deleted post",
            "post.published"     => "Published post",
            "post.unpublished"   => "Unpublished post",
            "post.bulk_delete"   => "Bulk deleted posts",
            "post.bulk_publish"  => "Bulk published posts",
            "post.bulk_unpublish" => "Bulk unpublished posts",
            "user.login"         => "Logged in",
            _ => &self.action,
        }
    }
}

pub struct ActivityLog;

impl ActivityLog {
    pub async fn create(
        pool: &DbPool,
        user_id: Option<i64>,
        action: &str,
        target_type: Option<&str>,
        target_id: Option<i64>,
    ) -> Result<(), sqlx::Error> {
        sqlx::query(
            "INSERT INTO activity_logs (user_id, action, target_type, target_id) VALUES (?, ?, ?, ?)",
        )
        .bind(user_id)
        .bind(action)
        .bind(target_type)
        .bind(target_id)
        .execute(pool)
        .await?;
        Ok(())
    }

    pub async fn find_recent(
        pool: &DbPool,
        limit: i64,
    ) -> Result<Vec<ActivityLogWithUser>, sqlx::Error> {
        sqlx::query_as::<_, ActivityLogWithUser>(
            r#"
            SELECT l.*, u.username
            FROM activity_logs l
            LEFT JOIN users u ON u.id = l.user_id
            ORDER BY l.created_at DESC
            LIMIT ?
            "#,
        )
        .bind(limit)
        .fetch_all(pool)
        .await
    }
}