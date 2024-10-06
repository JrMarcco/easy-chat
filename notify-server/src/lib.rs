mod sse;

use axum::{response::IntoResponse, routing::get, Router};
use axum_extra::response::Html;
use sse::sse_handler;

pub fn init_router() -> Router {
    Router::new()
        .route("/", get(index_handler))
        .route("/events", get(sse_handler))
}

async fn index_handler() -> impl IntoResponse {
    Html(include_str!("../index.html"))
}
