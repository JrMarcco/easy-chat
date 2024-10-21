mod auth;
mod request_id;
mod server_time;

pub use auth::*;
use axum::{
    middleware::{from_fn, from_fn_with_state},
    Router,
};
use request_id::x_request_id;
use server_time::ServerTimeLayer;
use tower::ServiceBuilder;
use tower_http::{
    compression::CompressionLayer,
    trace::{DefaultMakeSpan, DefaultOnRequest, DefaultOnResponse, TraceLayer},
    LatencyUnit,
};
use tracing::Level;

use crate::AppState;

const X_REQUEST_ID: &str = "x-request-id";
const X_SERVER_TIME: &str = "x-server-time";

pub fn set_layer(state: AppState, app: Router) -> Router {
    app.layer(
        ServiceBuilder::new()
            .layer(
                TraceLayer::new_for_http()
                    .make_span_with(DefaultMakeSpan::new().include_headers(true))
                    .on_request(DefaultOnRequest::new().level(Level::INFO))
                    .on_response(
                        DefaultOnResponse::new()
                            .level(Level::INFO)
                            .latency_unit(LatencyUnit::Micros),
                    ),
            )
            .layer(CompressionLayer::new().gzip(true).br(true).deflate(true))
            .layer(from_fn(x_request_id))
            .layer(from_fn_with_state(state, verify_token))
            .layer(ServerTimeLayer),
    )
}
