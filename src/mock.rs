use serde::Deserialize;
use std::collections::HashMap;
use std::fs;
use std::path::Path;

/// Root container for a mock definition file
#[derive(Debug, Deserialize, Clone)]
pub struct MockFile {
    pub mocks: Vec<MockEndpoint>,
}

/// A single endpoint definition with its possible responses
#[derive(Debug, Deserialize, Clone)]
pub struct MockEndpoint {
    /// URL path pattern, can include path parameters like {id}
    pub path: String,
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: String,
    /// List of possible responses, evaluated in order
    pub responses: Vec<MockResponse>,
}

/// A mock response configuration
#[derive(Debug, Deserialize, Clone)]
pub struct MockResponse {
    /// Optional conditions that must match for this response
    #[serde(default)]
    pub when: Option<ResponseCondition>,
    /// HTTP status code
    #[serde(default = "default_status")]
    pub status: u16,
    /// Response headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Response body as inline string
    #[serde(default)]
    pub body: Option<String>,
    /// Path to file containing response body
    #[serde(default)]
    pub body_file: Option<String>,
    /// Artificial delay in milliseconds
    #[serde(default)]
    pub delay_ms: Option<u64>,
}

fn default_status() -> u16 {
    200
}

/// Conditions for matching a request to a response
#[derive(Debug, Deserialize, Clone, Default)]
pub struct ResponseCondition {
    /// Exact match on URL path parameters (e.g., {id})
    #[serde(default)]
    pub path_params: HashMap<String, String>,
    /// Exact match on query string parameters
    #[serde(default)]
    pub query_params: HashMap<String, String>,
    /// Query string contains this substring
    #[serde(default)]
    pub query_contains: Option<String>,
    /// Exact match on request headers
    #[serde(default)]
    pub headers: HashMap<String, String>,
    /// Header value contains substring (header_name -> substring)
    #[serde(default)]
    pub header_contains: HashMap<String, String>,
    /// Request body contains this substring
    #[serde(default)]
    pub body_contains: Option<String>,
    /// Request body JSON has these key/value pairs
    #[serde(default)]
    pub body_json: HashMap<String, serde_json::Value>,
    /// Request body matches this regex pattern
    #[serde(default)]
    pub body_regex: Option<String>,
}

impl MockFile {
    /// Load a mock definition file from the specified path
    pub fn load<P: AsRef<Path>>(path: P) -> Result<Self, Box<dyn std::error::Error>> {
        let content = fs::read_to_string(&path)?;
    Self::from_yaml(&content)
    }
}

impl ResponseCondition {
    /// Check if this condition has any actual conditions defined
    pub fn is_empty(&self) -> bool {
        self.path_params.is_empty()
            && self.query_params.is_empty()
            && self.query_contains.is_none()
            && self.headers.is_empty()
            && self.header_contains.is_empty()
            && self.body_contains.is_none()
            && self.body_json.is_empty()
            && self.body_regex.is_none()
    }
}

impl MockFile {
    /// Parse mock definitions from a YAML string
    pub fn from_yaml(yaml: &str) -> Result<Self, Box<dyn std::error::Error>> {
        let mock_file: MockFile = serde_yaml::from_str(yaml)?;
        Ok(mock_file)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_mock() {
        let yaml = r#"
mocks:
  - path: "/api/health"
    method: GET
    responses:
      - status: 200
        body: '{"status": "ok"}'
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(mock_file.mocks.len(), 1);
        assert_eq!(mock_file.mocks[0].path, "/api/health");
        assert_eq!(mock_file.mocks[0].method, "GET");
        assert_eq!(mock_file.mocks[0].responses.len(), 1);
        assert_eq!(mock_file.mocks[0].responses[0].status, 200);
    }

    #[test]
    fn test_parse_mock_with_path_params_condition() {
        let yaml = r#"
mocks:
  - path: "/api/users/{id}"
    method: GET
    responses:
      - when:
          path_params:
            id: "123"
        status: 200
        body: '{"id": 123}'
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        let condition = mock_file.mocks[0].responses[0].when.as_ref().unwrap();
        assert_eq!(condition.path_params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_parse_mock_with_headers() {
        let yaml = r#"
mocks:
  - path: "/api/test"
    method: GET
    responses:
      - status: 200
        headers:
          Content-Type: "application/json"
          X-Custom: "value"
        body: '{}'
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        let headers = &mock_file.mocks[0].responses[0].headers;
        assert_eq!(headers.get("Content-Type"), Some(&"application/json".to_string()));
        assert_eq!(headers.get("X-Custom"), Some(&"value".to_string()));
    }

    #[test]
    fn test_parse_mock_with_delay() {
        let yaml = r#"
mocks:
  - path: "/api/slow"
    method: GET
    responses:
      - status: 200
        delay_ms: 5000
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(mock_file.mocks[0].responses[0].delay_ms, Some(5000));
    }

    #[test]
    fn test_parse_mock_with_body_file() {
        let yaml = r#"
mocks:
  - path: "/api/data"
    method: GET
    responses:
      - status: 200
        body_file: "./data/large.json"
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(
            mock_file.mocks[0].responses[0].body_file,
            Some("./data/large.json".to_string())
        );
    }

    #[test]
    fn test_parse_mock_with_all_conditions() {
        let yaml = r#"
mocks:
  - path: "/api/complex"
    method: POST
    responses:
      - when:
          path_params:
            id: "1"
          query_params:
            sort: "asc"
          query_contains: "filter="
          headers:
            Authorization: "Bearer token"
          header_contains:
            User-Agent: "Mozilla"
          body_contains: '"name":'
          body_json:
            type: "test"
          body_regex: "\\d+"
        status: 200
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        let condition = mock_file.mocks[0].responses[0].when.as_ref().unwrap();
        
        assert!(!condition.is_empty());
        assert_eq!(condition.path_params.get("id"), Some(&"1".to_string()));
        assert_eq!(condition.query_params.get("sort"), Some(&"asc".to_string()));
        assert_eq!(condition.query_contains, Some("filter=".to_string()));
        assert_eq!(condition.headers.get("Authorization"), Some(&"Bearer token".to_string()));
        assert_eq!(condition.header_contains.get("User-Agent"), Some(&"Mozilla".to_string()));
        assert_eq!(condition.body_contains, Some("\"name\":".to_string()));
        assert_eq!(condition.body_regex, Some("\\d+".to_string()));
    }

    #[test]
    fn test_response_condition_is_empty() {
        let empty = ResponseCondition::default();
        assert!(empty.is_empty());

        let mut with_path_param = ResponseCondition::default();
        with_path_param.path_params.insert("id".to_string(), "1".to_string());
        assert!(!with_path_param.is_empty());
    }

    #[test]
    fn test_default_status() {
        let yaml = r#"
mocks:
  - path: "/api/test"
    method: GET
    responses:
      - body: "ok"
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(mock_file.mocks[0].responses[0].status, 200); // default
    }

    #[test]
    fn test_multiple_endpoints() {
        let yaml = r#"
mocks:
  - path: "/api/users"
    method: GET
    responses:
      - status: 200
  - path: "/api/users"
    method: POST
    responses:
      - status: 201
  - path: "/api/products"
    method: GET
    responses:
      - status: 200
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(mock_file.mocks.len(), 3);
    }

    #[test]
    fn test_multiple_responses_same_endpoint() {
        let yaml = r#"
mocks:
  - path: "/api/users/{id}"
    method: GET
    responses:
      - when:
          path_params:
            id: "1"
        status: 200
      - when:
          path_params:
            id: "2"
        status: 200
      - status: 404
"#;
        let mock_file = MockFile::from_yaml(yaml).unwrap();
        assert_eq!(mock_file.mocks[0].responses.len(), 3);
    }
}

