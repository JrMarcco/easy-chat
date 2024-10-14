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
use middleware::{set_layer, TokenVerifier};
use model::SessionUser;
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

/// Deref to AppStateInner
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
        .with_state(state.clone());

    Ok(set_layer(state.clone(), app))
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

impl TokenVerifier for AppState {
    type Error = AppErr;

    fn verify_token(&self, token: &str) -> Result<SessionUser, Self::Error> {
        self.jwt_dk.verify(token)
    }
}

impl Debug for AppStateInner {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppStateInner")
            .field("config", &self.config)
            .finish()
    }
}

#[cfg(feature = "test-util")]
mod test_util {
    use super::*;
    use sqlx::Executor;
    use sqlx_db_tester::TestPg;

    impl AppState {
        pub async fn new_for_test() -> Result<(TestPg, Self), AppErr> {
            let config = AppConfig::try_load()?;

            let jwt_dk = JwtDecodingKey::load(&config.auth.public_key)?;
            let jwt_ek = JwtEncodingKey::load(&config.auth.private_key)?;

            let dsn_post = config.db.dsn.rfind('/').expect("invalid db dsn");
            let db_server_url = &config.db.dsn[..dsn_post];

            let (test_db, pg) = init_test_pool(Some(db_server_url)).await;

            let state = Self {
                inner: Arc::new(AppStateInner {
                    config,
                    pg,
                    jwt_ek,
                    jwt_dk,
                }),
            };

            Ok((test_db, state))
        }
    }

    pub async fn init_test_pool(url: Option<&str>) -> (TestPg, PgPool) {
        let url = match url {
            Some(s) => s.to_string(),
            None => "postgres://postgres:postgres@localhost:5432".to_string(),
        };

        let test_db = TestPg::new(url, std::path::Path::new("../migrations"));
        let pg_pool = test_db.get_pool().await;

        // run prepared sql for setup test data
        let sqls = include_str!("../fixtures/test_data.sql").split(";");
        let mut tx = pg_pool.begin().await.expect("begin tx fail");
        for sql in sqls {
            if sql.trim().is_empty() {
                continue;
            }

            tx.execute(sql).await.expect("exec sql fail");
        }
        tx.commit().await.expect("commit tx fail");

        (test_db, pg_pool)
    }
}
