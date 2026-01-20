use notify_debouncer_mini::{new_debouncer, DebouncedEventKind};
use notify::{RecursiveMode, Result as NotifyResult};
use std::path::Path;
use std::sync::Arc;
use std::time::Duration;
use tokio::sync::mpsc;
use tracing::{error, info, warn};

use crate::server::AppState;
use crate::config::AppConfig;

pub async fn start_file_watcher(
    state: Arc<AppState>,
    config: &AppConfig,
) -> NotifyResult<()> {
    let mock_dir = Path::new(&config.mock_files.directory).canonicalize()?;

    let (tx, mut rx) = mpsc::channel(100);

    // Create debounced watcher (waits 500ms after last change)
    let mut debouncer = new_debouncer(Duration::from_millis(500), move |res| {
        let _ = tx.blocking_send(res);
    })?;

    debouncer
        .watcher()
        .watch(&mock_dir, RecursiveMode::Recursive)?;

    info!("Watching for mock file changes in: {:?}", mock_dir);

    // Keep watcher alive and handle events
    tokio::spawn(async move {
        let _debouncer = debouncer; // Keep ownership to prevent drop

        while let Some(result) = rx.recv().await {
            match result {
                Ok(events) => {
                    // Filter for relevant file changes
                    let has_yaml_changes = events.iter().any(|e| {
                        e.path.extension()
                            .map(|ext| ext == "yaml" || ext == "yml")
                            .unwrap_or(false)
                            && matches!(e.kind, DebouncedEventKind::Any)
                    });

                    if has_yaml_changes {
                        info!("Mock files changed, reloading...");
                        if let Err(e) = state.reload_mocks().await {
                            error!("Failed to reload mocks: {}", e);
                        }
                    }
                }
                Err(e) => {
                    warn!("File watch error: {:?}", e);
                }
            }
        }
    });

    Ok(())
}
