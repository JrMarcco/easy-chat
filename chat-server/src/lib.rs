mod api;
mod config;
mod error;
mod middleware;
mod model;
mod util;

use anyhow::Context;
use api::*;
use axum::{
    http::Method,
    routing::{get, patch, post},
    Router,
};
use sqlx::PgPool;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use tower_http::cors::{Any, CorsLayer};
use util::{JwtDecodingKey, JwtEncodingKey};

pub use config::AppConfig;
pub use error::AppErr;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pg: PgPool,
    pub(crate) jwt_ek: JwtEncodingKey,
    pub(crate) jwt_dk: JwtDecodingKey,
}

// state.config => state.inner.config
impl Deref for AppState {
    type Target = AppStateInner;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

pub async fn init_app(state: AppState) -> Result<Router, AppErr> {
    let cors = CorsLayer::new()
        .allow_methods([Method::GET, Method::POST, Method::PUT, Method::DELETE])
        .allow_origin(Any)
        .allow_headers(Any);

    let api = Router::new()
        .route("/signup", post(sign_up_handler))
        .route("/signin", post(sign_in_handler))
        .route("/chat", get(list_chat_handler).post(create_chat_handler))
        .route(
            "/chat/:id",
            patch(update_chat_handler)
                .delete(delete_chat_handler)
                .post(send_message_handler),
        )
        .route("/chat/:id/message", get(list_message_handler))
        .layer(cors);

    let app = Router::new()
        .route("/", get(index_handler))
        .nest("/api", api)
        .with_state(state);

    Ok(app)
}

impl AppState {
    pub async fn try_new(config: AppConfig) -> Result<Self, AppErr> {
        let pg = PgPool::connect(&config.db.dsn)
            .await
            .context("connectg to db fail")?;

        let jwt_ek = JwtEncodingKey::load(&config.auth.private_key)?;
        let jwt_dk = JwtDecodingKey::load(&config.auth.public_key)?;

        Ok(Self {
            inner: Arc::new(AppStateInner {
                config,
                pg,
                jwt_ek,
                jwt_dk,
            }),
        })
    }
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}
