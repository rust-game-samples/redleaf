use argon2::{
    password_hash::{PasswordHash, PasswordHasher, PasswordVerifier, SaltString, rand_core::OsRng},
    Argon2,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::FromRow;

use crate::db::DbPool;

#[derive(Debug, Clone, Serialize, Deserialize, FromRow)]
pub struct User {
    pub id: i64,
    pub username: String,
    pub email: String,
    #[serde(skip_serializing)]
    pub password_hash: String,
    pub role: String,
    pub display_name: String,
    pub bio: String,
    pub website: String,
    pub avatar_url: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct CreateUser {
    pub username: String,
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct LoginUser {
    pub email: String,
    pub password: String,
}

#[derive(Debug, Deserialize)]
pub struct UpdateProfile {
    pub display_name: String,
    pub bio: String,
    pub website: String,
    pub avatar_url: String,
}

impl User {
    pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();
        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();
        Ok(password_hash)
    }

    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, anyhow::Error> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;
        let argon2 = Argon2::default();
        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_by_username(
        pool: &DbPool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    pub async fn find_all(pool: &DbPool) -> Result<Vec<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users ORDER BY created_at ASC")
            .fetch_all(pool)
            .await
    }

    pub async fn create(pool: &DbPool, create_user: CreateUser) -> Result<User, anyhow::Error> {
        let password_hash = Self::hash_password(&create_user.password)?;
        let now = Utc::now();
        let user = sqlx::query_as::<_, User>(
            r#"
            INSERT INTO users (username, email, password_hash, created_at, updated_at)
            VALUES (?, ?, ?, ?, ?)
            RETURNING *
            "#,
        )
        .bind(&create_user.username)
        .bind(&create_user.email)
        .bind(&password_hash)
        .bind(now)
        .bind(now)
        .fetch_one(pool)
        .await?;
        Ok(user)
    }

    pub async fn has_any(pool: &DbPool) -> bool {
        sqlx::query_scalar::<_, i64>("SELECT COUNT(*) FROM users")
            .fetch_one(pool)
            .await
            .map(|n| n > 0)
            .unwrap_or(false)
    }

    pub async fn authenticate(
        pool: &DbPool,
        login: LoginUser,
    ) -> Result<Option<User>, anyhow::Error> {
        let user = Self::find_by_email(pool, &login.email).await?;
        match user {
            Some(user) => {
                if Self::verify_password(&login.password, &user.password_hash)? {
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
    }

    pub async fn update_role(pool: &DbPool, id: i64, role: &str) -> Result<(), sqlx::Error> {
        sqlx::query("UPDATE users SET role = ?, updated_at = ? WHERE id = ?")
            .bind(role)
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub async fn update_profile(
        pool: &DbPool,
        id: i64,
        profile: UpdateProfile,
    ) -> Result<User, sqlx::Error> {
        sqlx::query_as::<_, User>(
            r#"
            UPDATE users
            SET display_name = ?, bio = ?, website = ?, avatar_url = ?, updated_at = ?
            WHERE id = ?
            RETURNING *
            "#,
        )
        .bind(&profile.display_name)
        .bind(&profile.bio)
        .bind(&profile.website)
        .bind(&profile.avatar_url)
        .bind(Utc::now())
        .bind(id)
        .fetch_one(pool)
        .await
    }

    pub async fn change_password(
        pool: &DbPool,
        id: i64,
        new_password: &str,
    ) -> Result<(), anyhow::Error> {
        let hash = Self::hash_password(new_password)?;
        sqlx::query("UPDATE users SET password_hash = ?, updated_at = ? WHERE id = ?")
            .bind(&hash)
            .bind(Utc::now())
            .bind(id)
            .execute(pool)
            .await?;
        Ok(())
    }

    pub fn effective_display_name(&self) -> &str {
        if self.display_name.is_empty() {
            &self.username
        } else {
            &self.display_name
        }
    }
}