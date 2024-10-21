use axum::{extract::State, http::StatusCode, response::IntoResponse, Json};
use serde::Serialize;

use crate::{
    error::ErrOutput,
    model::{SignInForm, SignUpForm},
    AppErr, AppState,
};

#[derive(Debug, Serialize)]
pub struct AuthOutput {
    token: String,
}

impl AuthOutput {
    pub fn new(token: String) -> Self {
        Self { token }
    }
}

pub(crate) async fn sign_in_handler(
    State(state): State<AppState>,
    Json(input): Json<SignInForm>,
) -> Result<impl IntoResponse, AppErr> {
    let user = state.verify_user(input).await?;

    match user {
        Some(user) => {
            let token = state.ek.sign(user)?;
            let body = Json(AuthOutput::new(token));
            Ok((StatusCode::OK, body).into_response())
        }
        None => {
            let body = Json(ErrOutput::new("Invalid email or password".to_string()));
            Ok((StatusCode::UNAUTHORIZED, body).into_response())
        }
    }
}

pub(crate) async fn sign_up_handler(
    State(state): State<AppState>,
    Json(input): Json<SignUpForm>,
) -> Result<impl IntoResponse, AppErr> {
    let user = state.create_user(input).await?;
    let token = state.ek.sign(user)?;

    let body = Json(AuthOutput::new(token));

    Ok((StatusCode::CREATED, body))
}
