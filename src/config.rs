use serde::Deserialize;
use std::fs;
use std::path::Path;

/// Main application configuration loaded from config/config.yaml
#[derive(Debug, Deserialize, Clone)]
pub struct AppConfig {
    pub server: ServerConfig,
    pub mock_files: MockFilesConfig,
}

/// HTTP server configuration
#[derive(Debug, Deserialize, Clone)]
pub struct ServerConfig {
    #[serde(default = "default_host")]
    pub host: String,
    #[serde(default = "default_port")]
    pub port: u16,
}

fn default_host() -> String {
    "127.0.0.1".to_string()
}

fn default_port() -> u16 {
    8080
}

/// Configuration for locating mock definition files
#[derive(Debug, Deserialize, Clone)]
pub struct MockFilesConfig {
    /// Directory containing mock YAML files
    pub directory: String,
    /// Glob patterns for matching mock files (e.g., "**/*.yaml")
    #[serde(default = "default_patterns")]
    pub patterns: Vec<String>,
    /// Enable or disable hot reload of mock files
    #[serde(default = "default_hot_reload")]
    pub hot_reload: bool,
}

fn default_patterns() -> Vec<String> {
    vec!["**/*.yaml".to_string(), "**/*.yml".to_string()]
}

fn default_hot_reload() -> bool {
    true
}

impl AppConfig {
    /// Load configuration from the specified path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(path)?;
        Self::from_yaml(&content)
    }

    /// Load configuration from default location (./config/config.yaml)
    pub fn load_default() -> Result<Self, Box<dyn std::error::Error>> {
        Self::load("./config/config.yaml")
    }

    /// Parse configuration from a YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let config: AppConfig = serde_yaml::from_str(yaml)?;
        Ok(config)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_full_config() {
        let yaml = r#"
server:
  host: "0.0.0.0"
  port: 9000
mock_files:
  directory: "./test-mocks"
  patterns:
    - "*.yaml"
"#;
        let config = AppConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.server.host, "0.0.0.0");
        assert_eq!(config.server.port, 9000);
        assert_eq!(config.mock_files.directory, "./test-mocks");
        assert_eq!(config.mock_files.patterns, vec!["*.yaml"]);
    }

    #[test]
    fn test_default_host_and_port() {
        let yaml = r#"
server: {}
mock_files:
  directory: "./mocks"
"#;
        let config = AppConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1");
        assert_eq!(config.server.port, 8080);
    }

    #[test]
    fn test_default_patterns() {
        let yaml = r#"
server:
  host: "127.0.0.1"
  port: 8080
mock_files:
  directory: "./mocks"
"#;
        let config = AppConfig::from_yaml(yaml).unwrap();
        assert_eq!(
            config.mock_files.patterns,
            vec!["**/*.yaml".to_string(), "**/*.yml".to_string()]
        );
    }

    #[test]
    fn test_load_missing_file() {
        let result = AppConfig::load("/nonexistent/path/config.yaml");
        assert!(result.is_err());
    }

    #[test]
    fn test_invalid_yaml() {
        let yaml = "this is not valid yaml: [";
        let result = AppConfig::from_yaml(yaml);
        assert!(result.is_err());
    }

    #[test]
    fn test_partial_server_config() {
        let yaml = r#"
server:
  port: 3000
mock_files:
  directory: "./mocks"
"#;
        let config = AppConfig::from_yaml(yaml).unwrap();
        assert_eq!(config.server.host, "127.0.0.1"); // default
        assert_eq!(config.server.port, 3000); // specified
    }
}

