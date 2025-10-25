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

impl User {
    // Hash a password
    pub fn hash_password(password: &str) -> Result<String, anyhow::Error> {
        let salt = SaltString::generate(&mut OsRng);
        let argon2 = Argon2::default();

        let password_hash = argon2
            .hash_password(password.as_bytes(), &salt)
            .map_err(|e| anyhow::anyhow!("Failed to hash password: {}", e))?
            .to_string();

        Ok(password_hash)
    }

    // Verify a password
    pub fn verify_password(password: &str, password_hash: &str) -> Result<bool, anyhow::Error> {
        let parsed_hash = PasswordHash::new(password_hash)
            .map_err(|e| anyhow::anyhow!("Failed to parse password hash: {}", e))?;

        let argon2 = Argon2::default();

        Ok(argon2
            .verify_password(password.as_bytes(), &parsed_hash)
            .is_ok())
    }

    // Find user by ID
    pub async fn find_by_id(pool: &DbPool, id: i64) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE id = ?")
            .bind(id)
            .fetch_optional(pool)
            .await
    }

    // Find user by email
    pub async fn find_by_email(pool: &DbPool, email: &str) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE email = ?")
            .bind(email)
            .fetch_optional(pool)
            .await
    }

    // Find user by username
    pub async fn find_by_username(
        pool: &DbPool,
        username: &str,
    ) -> Result<Option<User>, sqlx::Error> {
        sqlx::query_as::<_, User>("SELECT * FROM users WHERE username = ?")
            .bind(username)
            .fetch_optional(pool)
            .await
    }

    // Create new user
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

    // Authenticate user
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
}
