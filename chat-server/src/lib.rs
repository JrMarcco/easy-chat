mod api;
mod config;
mod error;
mod model;

use anyhow::Context;
use api::*;
use axum::{http::Method, routing::get, Router};
use sqlx::PgPool;
use std::{fmt::Debug, ops::Deref, sync::Arc};
use tower_http::cors::{Any, CorsLayer};

pub use config::AppConfig;
pub use error::AppErr;

#[derive(Debug, Clone)]
pub struct AppState {
    inner: Arc<AppStateInner>,
}

pub struct AppStateInner {
    pub(crate) config: AppConfig,
    pub(crate) pg: PgPool,
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

    let api = Router::new().layer(cors);

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

        Ok(Self {
            inner: Arc::new(AppStateInner { config, pg }),
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
