use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct Comment {
    pub id: i64,
    pub post_id: i64,
    pub parent_id: Option<i64>,
    pub author_name: String,
    pub author_email: String,
    pub content: String,
    pub approved: bool,
    pub spam: bool,
    pub created_at: DateTime<Utc>,
}

/// Comment joined with its post title/slug for admin display.
#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct CommentWithPost {
    pub id: i64,
    pub post_id: i64,
    pub parent_id: Option<i64>,
    pub author_name: String,
    pub author_email: String,
    pub content: String,
    pub approved: bool,
    pub spam: bool,
    pub created_at: DateTime<Utc>,
    pub post_title: String,
    pub post_slug: String,
}

impl Comment {
    /// All approved comments for a post (ordered parent-first, then by created_at).
    pub async fn find_by_post(pool: &DbPool, post_id: i64) -> Result<Vec<Comment>, sqlx::Error> {
        sqlx::query_as::<_, Comment>(
            "SELECT * FROM comments WHERE post_id = ? AND approved = 1 AND spam = 0 ORDER BY created_at ASC",
        )
        .bind(post_id)
        .fetch_all(pool)
        .await
    }

    /// Count approved comments for a post.
    pub async fn count_by_post(pool: &DbPool, post_id: i64) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE post_id = ? AND approved = 1 AND spam = 0")
            .bind(post_id)
            .fetch_one(pool)
            .await
    }

    /// All comments for admin moderation (including pending/spam), newest first.
    pub async fn find_all_admin(pool: &DbPool) -> Result<Vec<CommentWithPost>, sqlx::Error> {
        sqlx::query_as::<_, CommentWithPost>(
            r#"
            SELECT c.*, p.title AS post_title, p.slug AS post_slug
            FROM comments c
            JOIN posts p ON p.id = c.post_id
            ORDER BY c.created_at DESC
            "#,
        )
        .fetch_all(pool)
        .await
    }

    /// Count all comments pending approval.
    pub async fn count_pending(pool: &DbPool) -> Result<i64, sqlx::Error> {
        sqlx::query_scalar("SELECT COUNT(*) FROM comments WHERE approved = 0 AND spam = 0")
            .fetch_one(pool)
            .await
    }

    /// Submit a new comment (pending by default).
    pub async fn create(
        pool: &DbPool,
        post_id: i64,
        parent_id: Option<i64>,
        author_name: &str,
        author_email: &str,
        content: &str,
    ) -> Result<Comment, sqlx::Error> {
        sqlx::query_as::<_, Comment>(
            r#"
            INSERT INTO comments (post_id, parent_id, author_name, author_email, content, approved, spam)
            VALUES (?, ?, ?, ?, ?, 0, 0)
            RETURNING *
            "#,
        )
        .bind(post_id)
        .bind(parent_id)
        .bind(author_name)
        .bind(author_email)
        .bind(content)
        .fetch_one(pool)
        .await
    }

    pub async fn approve(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE comments SET approved = 1, spam = 0 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn reject(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("DELETE FROM comments WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn mark_spam(pool: &DbPool, id: i64) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE comments SET spam = 1, approved = 0 WHERE id = ?")
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    /// Render threaded comments as an `<ol>` HTML string.
    pub fn render_thread(all: &[Comment], parent_id: Option<i64>) -> String {
        let children: Vec<&Comment> = all
            .iter()
            .filter(|c| c.parent_id == parent_id)
            .collect();

        if children.is_empty() {
            return String::new();
        }

        let mut html = String::from(r#"<ol class="comment-list">"#);
        for c in children {
            let author = escape_html(&c.author_name);
            let content = escape_html(&c.content);
            let date = c.created_at.format("%Y-%m-%d %H:%M").to_string();
            let initial = c.author_name.chars().next()
                .map(|ch| ch.to_uppercase().to_string())
                .unwrap_or_else(|| "?".into());

            html.push_str(&format!(
                "<li id=\"comment-{id}\" class=\"comment\">\
<div class=\"comment-header\">\
<span class=\"comment-avatar\">{initial}</span>\
<div><strong class=\"comment-author\">{author}</strong> \
<time class=\"comment-date\">{date}</time></div>\
<a class=\"comment-reply-link\" href=\"#comment-form\" data-parent=\"{id}\">Reply</a>\
</div>\
<div class=\"comment-body\">{content}</div>\n",
                id = c.id,
                initial = initial,
                author = author,
                date = date,
                content = content,
            ));

            let sub = Self::render_thread(all, Some(c.id));
            if !sub.is_empty() {
                html.push_str(&sub);
            }
            html.push_str("</li>\n");
        }
        html.push_str("</ol>\n");
        html
    }
}

fn escape_html(s: &str) -> String {
    s.replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}