use anyhow::Result;
use notify_server::init_router;
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, prelude::*};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let addr = "0.0.0.0:3001";

    let app = init_router();

    let listener = TcpListener::bind(addr).await?;
    println!("listening on {}", addr);

    axum::serve(listener, app.into_make_service()).await?;

    Ok(())
}
