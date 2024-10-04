use anyhow::Result;
use chat_server::{init_app, AppConfig};
use tokio::net::TcpListener;
use tracing::level_filters::LevelFilter;
use tracing_subscriber::{fmt::Layer, layer::SubscriberExt, util::SubscriberInitExt, Layer as _};

#[tokio::main]
async fn main() -> Result<()> {
    let layer = Layer::new().pretty().with_filter(LevelFilter::INFO);
    tracing_subscriber::registry().with(layer).init();

    let config = AppConfig::try_load()?;
    let addr = format!("0.0.0.0:{}", config.server.port);

    let app = init_app(config);
    let listener = TcpListener::bind(addr).await?;

    axum::serve(listener, app.into_make_service()).await?;
    Ok(())
}
