use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrOutput {
    pub err_msg: String,
}

#[derive(Debug, Error)]
pub enum AppErr {
    #[error("io error: {0}")]
    IoErr(#[from] std::io::Error),

    #[error("sqlx error: {0}")]
    SqlXErr(#[from] sqlx::Error),

    #[error("anyhow error: {0}")]
    AnyhowErr(#[from] anyhow::Error),

    #[error("password hash error: {0}")]
    PasswdHashErr(#[from] argon2::password_hash::Error),

    #[error("jwt error: {0}")]
    JwtErr(#[from] jsonwebtoken::errors::Error),

    #[error("auth error: {0}")]
    AuthErr(String),
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::IoErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqlXErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AnyhowErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::PasswdHashErr(_) => StatusCode::UNPROCESSABLE_ENTITY,
            Self::JwtErr(_) => StatusCode::UNAUTHORIZED,
            Self::AuthErr(_) => StatusCode::UNAUTHORIZED,
        };

        (status, Json(ErrOutput::new(self.to_string()))).into_response()
    }
}

impl ErrOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            err_msg: error.into(),
        }
    }
}
