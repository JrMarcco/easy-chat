mod auth;

pub(crate) use auth::*;

pub(crate) async fn index_handler() -> &'static str {
    "index"
}
