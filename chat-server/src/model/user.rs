use argon2::{
    password_hash::{rand_core::OsRng, PasswordHash, PasswordHasher, PasswordVerifier, SaltString},
    Argon2,
};
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sqlx::prelude::FromRow;

use crate::{AppErr, AppState};

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
pub struct SignUpForm {
    pub username: String,
    pub email: String,
    pub passwd: String,
    pub avatar: String,
}

#[derive(Debug, Deserialize)]
pub struct SignInForm {
    pub email: String,
    pub passwd: String,
}

impl AppState {
    /// Create a user
    pub async fn create_user(&self, form: SignUpForm) -> Result<User, AppErr> {
        let passwd_hash = hash_passwd(&form.passwd)?;

        let user = sqlx::query_as(
            "INSERT INTO t_user (username, passwd, email, avatar) VALUES ($1, $2, $3, $4) RETURNING id, username, passwd, email, avatar, created_at, updated_at",
        )
        .bind(&form.username)
        .bind(&passwd_hash)
        .bind(&form.email)
        .bind(&form.avatar)
        .fetch_one(&self.pg)
        .await?;

        Ok(user)
    }

    /// Verify user email and password
    pub async fn verify_user(&self, form: SignInForm) -> Result<Option<User>, AppErr> {
        let user: Option<User> = sqlx::query_as(
            "SELECT id, username, passwd, email, avatar, created_at, updated_at FROM t_user where emial = $1"
        )
        .bind(&form.email)
        .fetch_optional(&self.pg)
        .await?;

        match user {
            Some(user) => {
                // verify user password
                let is_valid = verify_passwd(&form.passwd, &user.passwd)?;
                if is_valid {
                    // check ad load workspace info
                    Ok(Some(user))
                } else {
                    Ok(None)
                }
            }
            None => Ok(None),
        }
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

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SessionUser {
    pub id: i64,
    pub username: String,
    pub email: String,
}

impl From<User> for SessionUser {
    fn from(user: User) -> Self {
        Self {
            id: user.id,
            username: user.username,
            email: user.email,
        }
    }
}

impl SessionUser {
    pub fn new(id: i64, username: String, email: String) -> Self {
        Self {
            id,
            username,
            email,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use anyhow::{Ok, Result};

    #[test]
    fn test_hash_and_verify_passwd() -> Result<()> {
        let passwd = "test_password";
        let passwd_hash = hash_passwd(passwd).expect("Failed to hash password");

        assert!(verify_passwd(passwd, &passwd_hash).expect("Failed to verify password"));
        Ok(())
    }

    #[test]
    fn test_verify_invalid_passwd() -> Result<()> {
        let passwd = "test_password";
        let invalid_passwd = "invalid_password";
        let passwd_hash = hash_passwd(passwd).expect("Failed to hash password");

        assert!(!verify_passwd(invalid_passwd, &passwd_hash).expect("Failed to verify password"));
        Ok(())
    }
}
