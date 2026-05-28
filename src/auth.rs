use chrono::Utc;
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};
use std::env;

use crate::models::User;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claims {
    pub sub: i64,
    pub username: String,
    pub email: String,
    #[serde(default = "default_role")]
    pub role: String,
    pub iat: u64,
    pub exp: u64,
}

fn default_role() -> String {
    "administrator".to_string()
}

impl Claims {
    pub fn new(user: &User) -> Self {
        let now = Utc::now().timestamp() as u64;
        Self {
            sub: user.id,
            username: user.username.clone(),
            email: user.email.clone(),
            role: user.role.clone(),
            iat: now,
            exp: now + 60 * 60 * 24 * 7, // 7 days
        }
    }
}

fn secret() -> Vec<u8> {
    env::var("JWT_SECRET")
        .unwrap_or_else(|_| "change-this-secret-in-production".to_string())
        .into_bytes()
}

pub fn generate_token(user: &User) -> anyhow::Result<String> {
    let claims = Claims::new(user);
    encode(
        &Header::default(),
        &claims,
        &EncodingKey::from_secret(&secret()),
    )
    .map_err(|e| anyhow::anyhow!("Failed to generate token: {}", e))
}

pub fn validate_token(token: &str) -> anyhow::Result<Claims> {
    let data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(&secret()),
        &Validation::default(),
    )
    .map_err(|e| anyhow::anyhow!("Invalid token: {}", e))?;
    Ok(data.claims)
}