mod api;
mod auth;
mod config;
mod error;
mod rooms;
mod webhooks;
mod egress;

use std::sync::Arc;
use tracing_subscriber::EnvFilter;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    dotenvy::dotenv().ok();

    

    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env())
        .init();

    let cfg = config::Config::from_env()?;

    tracing::info!("Connecting to LiveKit at {}", cfg.livekit_url);

    let tokens = Arc::new(auth::TokenService::new(&cfg.api_key, &cfg.api_secret));
    let rooms  = Arc::new(rooms::RoomService::new(&cfg.livekit_url)?);
    let egress = Arc::new(egress::EgressService::new(&cfg.livekit_url, cfg.s3_access_key, cfg.s3_secret, cfg.s3_region, cfg.s3_bucket)?);
    
    let state = api::AppState {
        tokens,
        rooms,
        egress,
        api_key:     cfg.api_key.clone(),
        api_secret:  cfg.api_secret.clone(),
        livekit_url: cfg.livekit_url.clone(),
    };

    let app = api::router(state);
    let listener = tokio::net::TcpListener::bind(&cfg.bind_addr).await?;

    tracing::info!("API server listening on {}", cfg.bind_addr);
    axum::serve(listener, app).await?;

    Ok(())
    
}