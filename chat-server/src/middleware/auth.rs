use axum::{
    extract::{FromRequestParts, Query, Request, State},
    http::StatusCode,
    middleware::Next,
    response::{IntoResponse, Response},
};
use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};
use serde::Deserialize;
use tracing::warn;

use crate::AppState;

#[derive(Debug, Deserialize)]
struct Params {
    token: String,
}

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    let (mut parts, body) = req.into_parts();

    let token =
        match TypedHeader::<Authorization<Bearer>>::from_request_parts(&mut parts, &state).await {
            Ok(TypedHeader(Authorization(bearer))) => bearer.token().to_string(),
            Err(e) => {
                if e.is_missing() {
                    match Query::<Params>::from_request_parts(&mut parts, &state).await {
                        Ok(params) => params.token.clone(),
                        Err(e) => {
                            let err_msg = format!("Failed to parse token from query: {}", e);
                            warn!(err_msg);
                            return (StatusCode::UNAUTHORIZED, err_msg).into_response();
                        }
                    }
                } else {
                    let err_msg = format!("Invalid Authorization header: {}", e);
                    warn!(err_msg);
                    return (StatusCode::UNAUTHORIZED, err_msg).into_response();
                }
            }
        };

    // verify token and get user info
    let req = match state.dk.verify(&token) {
        Ok(user) => {
            let mut req = Request::from_parts(parts, body);
            req.extensions_mut().insert(user);
            req
        }
        Err(e) => {
            let err_msg = format!("Failed to verify token: {}", e);
            warn!(err_msg);
            return (StatusCode::UNAUTHORIZED, err_msg).into_response();
        }
    };

    next.run(req).await
}
