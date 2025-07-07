use anyhow::Result;
use tracing::info;
use tracing_subscriber::{
    EnvFilter, fmt::layer, prelude::__tracing_subscriber_SubscriberExt, registry,
    util::SubscriberInitExt,
};
mod libs;
use axum::{Router, extract::Json, routing::get};
use libs::config::{Config, LogFormat};
use libs::data::data_router;
use libs::error::HttpResult;
use libs::postgres::conn;
use libs::schema::Store;
use libs::shared::Shared;
use serde_json::Value;

async fn is_ready() -> HttpResult<Json<Value>> {
    Ok(axum::Json("ok".into())).into()
}

#[tokio::main]
async fn main() -> Result<()> {
    let cfg = Config::new()?;

    let filter = EnvFilter::try_from_default_env().unwrap_or_else(|_| EnvFilter::new("info"));
    match &cfg.trace.format {
        LogFormat::compact => {
            registry().with(layer().compact()).with(filter).init();
        }
        LogFormat::json => {
            registry().with(layer().json()).with(filter).init();
        }
    };

    let client = conn(&cfg.database).await?;
    let shared = Shared::new(Store::new(client, cfg.database.allow_list));

    let app = Router::new()
        .nest("/v1", data_router())
        .route("/is_ready", get(is_ready))
        .with_state(shared);

    let addr = "0.0.0.0:5050";
    let listener = tokio::net::TcpListener::bind(&addr).await?;
    info!("Listening on {}", addr);

    axum::serve(listener, app).await?;
    Ok(())
}
