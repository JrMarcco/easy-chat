use axum::{
    extract::{Request, State},
    middleware::Next,
    response::Response,
};

use crate::AppState;

pub async fn verify_token(State(state): State<AppState>, req: Request, next: Next) -> Response {
    next.run(req).await
}
