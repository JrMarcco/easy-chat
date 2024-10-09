use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::{prelude::FromRow, PgPool};

use crate::AppErr;

#[derive(Debug, Clone, FromRow, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct User {
    pub id: i64,
    pub username: String,
    #[serde(skip)]
    pub passwd: String,
    pub email: String,
    pub avatar: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Deserialize)]
pub struct SignInForm {
    pub email: String,
    pub passwd: String,
}

#[derive(Debug, Deserialize)]
pub struct SignUpForm {
    pub username: String,
    pub email: String,
    pub passwd: String,
    pub avatar: String,
}

impl User {
    /// Find a user by email
    pub async fn find_by_email(email: &str, pg: &PgPool) -> Result<Option<User>, AppErr> {
        let user = sqlx::query_as(
            "SELECT id, username, passwd, email, avatar, created_at, updated_at FROM t_user WHERE email = $1",
        )
        .bind(email)
        .fetch_optional(pg)
        .await?;

        Ok(user)
    }

    /// Create a new user
    pub async fn create(
        username: &str,
        passwd: &str,
        email: &str,
        avatar: &str,
        pg: &PgPool,
    ) -> Result<Self, AppErr> {
        let passwd_hash = hash_passwd(passwd)?;

        let user = sqlx::query_as(
            "INSERT INTO t_user (username, passwd, email, avatar) VALUES ($1, $2, $3, $4) RETURNING id, username, passwd, email, avatar, created_at, updated_at",
        )
        .bind(username)
        .bind(passwd_hash)
        .bind(email)
        .bind(avatar)
        .fetch_one(pg)
        .await?;

        Ok(user)
    }
}

fn hash_passwd(passwd: &str) -> Result<String, AppErr> {
    let salt = SaltString::generate(&mut OsRng);

    let argon2 = Argon2::default();

    let passwd_hash = argon2.hash_password(passwd.as_bytes(), &salt)?.to_string();

    Ok(passwd_hash)
}

fn verify_passwd(passwd: &str, passwd_hash: &str) -> Result<bool, AppErr> {
    let argon2 = Argon2::default();
    let passwd_hash = PasswordHash::new(passwd_hash)?;

    let is_valid = argon2
        .verify_password(passwd.as_bytes(), &passwd_hash)
        .is_ok();

    Ok(is_valid)
}
