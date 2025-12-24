mod config;
mod matcher;
mod mock;
mod server;

use config::AppConfig;
use server::{create_router, load_mocks, AppState};
use std::sync::Arc;
use tracing::info;
use tracing_subscriber;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    tracing_subscriber::fmt::init();

    info!("Starting jimjam mock server...");

    // Load configuration
    let config = AppConfig::load_default().map_err(|e| {
        eprintln!("Failed to load config from ./config/config.yaml: {}", e);
        e
    })?;

    info!(
        "Configuration loaded. Server will listen on {}:{}",
        config.server.host, config.server.port
    );

    // Load all mock definitions
    let endpoints = load_mocks(&config)?;

    if endpoints.is_empty() {
        eprintln!("Warning: No mock endpoints were loaded!");
    }

    // Create application state
    let state = Arc::new(AppState { endpoints });

    // Create router
    let app = create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("jimjam is listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
