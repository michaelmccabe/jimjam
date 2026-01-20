use crate::config::AppConfig;
use crate::matcher::{find_matching_response, match_path, RequestInfo};
use crate::mock::{MockEndpoint, MockFile};
use axum::{
    body::Body,
    extract::State,
    http::{HeaderMap, HeaderName, HeaderValue, Method, Request, StatusCode},
    response::Response,
    routing::{any, post},
    Router,
};
use glob::glob;
use std::collections::HashMap;
use std::fs;
use std::path::Path;
use std::sync::Arc;
use tokio::time::{sleep, Duration};
use tracing::{error, info, warn};
use tokio::sync::RwLock;

/// Shared application state
pub struct AppState {
    pub endpoints: RwLock<Vec<MockEndpoint>>, // mutable for hot reload
    pub config: AppConfig,                    // keep config for reload
}

/// Load all mock definitions from the configured directory
pub fn load_mocks(config: &AppConfig) -> Result<Vec<MockEndpoint>, Box<dyn std::error::Error>> {
    let mut all_endpoints = Vec::new();
    let base_dir = Path::new(&config.mock_files.directory);

    for pattern in &config.mock_files.patterns {
        let full_pattern = base_dir.join(pattern);
        let pattern_str = full_pattern.to_string_lossy();

        info!("Loading mocks from pattern: {}", pattern_str);

        for entry in glob(&pattern_str)? {
            match entry {
                Ok(path) => {
                    info!("Loading mock file: {:?}", path);
                    match MockFile::load(&path) {
                        Ok(mock_file) => {
                            all_endpoints.extend(mock_file.mocks);
                        }
                        Err(e) => {
                            error!("Failed to load mock file {:?}: {}", path, e);
                        }
                    }
                }
                Err(e) => {
                    warn!("Error reading glob entry: {}", e);
                }
            }
        }
    }

    info!("Loaded {} mock endpoints", all_endpoints.len());
    Ok(all_endpoints)
}

impl AppState {
    pub async fn reload_mocks(&self) -> Result<usize, Box<dyn std::error::Error>> {
        let new_endpoints = load_mocks(&self.config)?;
        let count = new_endpoints.len();
        *self.endpoints.write().await = new_endpoints;
        info!("Reloaded {} mock endpoints", count);
        Ok(count)
    }
}

/// Create the Axum router with all routes
pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/__admin/reload", post(reload_mocks_handler))
        .route("/*path", any(handle_request))
        .route("/", any(handle_request))
        .with_state(state)
}

/// Main request handler - matches against all loaded mock endpoints
async fn handle_request(
    State(state): State<Arc<AppState>>,
    method: Method,
    headers: HeaderMap,
    request: Request<Body>,
) -> Response<Body> {
    let uri = request.uri().clone();
    let path = uri.path();
    let query_string = uri.query().map(|s| s.to_string());
    let method_str = method.to_string();

    // Extract headers into a HashMap (lowercase keys for case-insensitive matching)
    let mut header_map = HashMap::new();
    for (name, value) in headers.iter() {
        if let Ok(v) = value.to_str() {
            header_map.insert(name.to_string().to_lowercase(), v.to_string());
        }
    }

    // Read request body
    let body_bytes = match axum::body::to_bytes(request.into_body(), usize::MAX).await {
        Ok(bytes) => bytes,
        Err(e) => {
            error!("Failed to read request body: {}", e);
            return build_error_response(500, "Failed to read request body");
        }
    };
    let body_str = String::from_utf8_lossy(&body_bytes).to_string();

    let request_info = RequestInfo {
        query_string,
        headers: header_map,
        body: body_str,
    };

    info!("{} {}", method_str, path);

    // Find matching endpoint and response
    let endpoints = state.endpoints.read().await;
    for endpoint in endpoints.iter() {
        // Check method matches
        if endpoint.method.to_uppercase() != method_str {
            continue;
        }

        // Check path matches
        if let Some(path_match) = match_path(&endpoint.path, path) {
            if let Some(response) = find_matching_response(endpoint, &request_info, &path_match.params) {
                return build_mock_response(response, &endpoint.path).await;
            }
        }
    }

    // No match found
    warn!("No mock found for {} {}", method_str, path);
    build_error_response(404, &format!("No mock defined for {} {}", method_str, path))
}

/// Admin handler to trigger manual reload
async fn reload_mocks_handler(State(state): State<Arc<AppState>>) -> Response<Body> {
    match state.reload_mocks().await {
        Ok(count) => {
            let body = serde_json::json!({
                "status": "ok",
                "endpoints_loaded": count
            })
            .to_string();
            Response::builder()
                .status(StatusCode::OK)
                .header("Content-Type", "application/json")
                .body(Body::from(body))
                .unwrap()
        }
        Err(e) => build_error_response(500, &format!("Reload failed: {}", e)),
    }
}

/// Build the HTTP response from a MockResponse
async fn build_mock_response(
    mock_response: &crate::mock::MockResponse,
    endpoint_path: &str,
) -> Response<Body> {
    // Apply delay if configured
    if let Some(delay) = mock_response.delay_ms {
        sleep(Duration::from_millis(delay)).await;
    }

    // Get response body
    // Supports three formats:
    // 1. body: "@./path/to/file.json" - load from file (@ prefix)
    // 2. body_file: "./path/to/file.json" - load from file (explicit field)
    // 3. body: "inline content" - use directly
    let body_content = if let Some(ref body) = mock_response.body {
        if body.starts_with('@') {
            // Body references a file with @ prefix
            let file_path = &body[1..]; // Remove @ prefix
            match fs::read_to_string(file_path) {
                Ok(content) => content,
                Err(e) => {
                    error!("Failed to read body file {}: {}", file_path, e);
                    return build_error_response(500, &format!("Failed to read body file: {}", e));
                }
            }
        } else {
            body.clone()
        }
    } else if let Some(ref body_file) = mock_response.body_file {
        match fs::read_to_string(body_file) {
            Ok(content) => content,
            Err(e) => {
                error!("Failed to read body_file {}: {}", body_file, e);
                return build_error_response(500, &format!("Failed to read body file: {}", e));
            }
        }
    } else {
        String::new()
    };

    // Build response
    let mut builder = Response::builder().status(mock_response.status);

    // Add default Content-Type if not specified
    let mut has_content_type = false;
    for (name, value) in &mock_response.headers {
        if name.to_lowercase() == "content-type" {
            has_content_type = true;
        }
        if let (Ok(header_name), Ok(header_value)) = (
            HeaderName::try_from(name.as_str()),
            HeaderValue::try_from(value.as_str()),
        ) {
            builder = builder.header(header_name, header_value);
        }
    }

    if !has_content_type && !body_content.is_empty() {
        builder = builder.header("Content-Type", "application/json");
    }

    info!(
        "Responding with status {} for {}",
        mock_response.status, endpoint_path
    );

    builder.body(Body::from(body_content)).unwrap()
}

/// Build an error response
fn build_error_response(status: u16, message: &str) -> Response<Body> {
    let body = serde_json::json!({
        "error": message
    })
    .to_string();

    Response::builder()
        .status(StatusCode::from_u16(status).unwrap_or(StatusCode::INTERNAL_SERVER_ERROR))
        .header("Content-Type", "application/json")
        .body(Body::from(body))
        .unwrap()
}
