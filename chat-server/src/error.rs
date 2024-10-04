use axum::{
    http::StatusCode,
    response::{IntoResponse, Response},
    Json,
};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Serialize, Deserialize)]
pub struct ErrOutput {
    pub error: String,
}

impl ErrOutput {
    pub fn new(error: impl Into<String>) -> Self {
        Self {
            error: error.into(),
        }
    }
}

#[derive(Debug, Error)]
pub enum AppErr {
    #[error("io error: {0}")]
    IoErr(#[from] std::io::Error),

    #[error("sqlx error: {0}")]
    SqlXErr(#[from] sqlx::Error),

    #[error("anyhow error: {0}")]
    AnyhowErr(#[from] anyhow::Error),
}

impl IntoResponse for AppErr {
    fn into_response(self) -> Response {
        let status = match &self {
            Self::IoErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::SqlXErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::AnyhowErr(_) => StatusCode::INTERNAL_SERVER_ERROR,
        };

        (status, Json(ErrOutput::new(self.to_string()))).into_response()
    }
}
