mod config;
mod matcher;
mod mock;
mod server;
mod watcher;

use config::AppConfig;
use server::{create_router, load_mocks, AppState};
use std::sync::Arc;
use tokio::sync::RwLock;
use tracing::{debug, info};
use tracing_subscriber;
use watcher::start_file_watcher;

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

    debug!("Configuration loaded successfully");
    info!(
        host = %config.server.host,
        port = config.server.port,
        "Server configuration loaded"
    );

    // Load all mock definitions
    let endpoints = load_mocks(&config)?;

    if endpoints.is_empty() {
        eprintln!("Warning: No mock endpoints were loaded!");
    } else {
        eprintln!("Loaded {} mock endpoints", endpoints.len());
    }

    // Create application state
    let state = Arc::new(AppState {
        endpoints: RwLock::new(endpoints),
        config: config.clone(),
    });

    // Start file watcher for hot reload if enabled
    if config.mock_files.hot_reload {
        if let Err(e) = start_file_watcher(Arc::clone(&state), &config).await {
            tracing::warn!("Failed to start file watcher: {}. Hot reload disabled.", e);
        }
    }

    // Create router
    let app = create_router(state);

    // Start server
    let addr = format!("{}:{}", config.server.host, config.server.port);
    let listener = tokio::net::TcpListener::bind(&addr).await?;

    info!("jimjam is listening on http://{}", addr);

    axum::serve(listener, app).await?;

    Ok(())
}
