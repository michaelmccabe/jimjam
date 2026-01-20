use crate::mock::{MockEndpoint, MockResponse, ResponseCondition};
use regex::Regex;
use std::collections::HashMap;

/// Represents an incoming HTTP request for matching
pub struct RequestInfo {
    pub query_string: Option<String>,
    pub headers: HashMap<String, String>,
    pub body: String,
}

/// Result of path matching, includes extracted path parameters
pub struct PathMatch {
    pub params: HashMap<String, String>,
}

/// Check if a path pattern matches a request path
/// Supports path parameters like /users/{id}/posts/{post_id}
pub fn match_path(pattern: &str, request_path: &str) -> Option<PathMatch> {
    let pattern_segments: Vec<&str> = pattern.trim_matches('/').split('/').collect();
    let path_segments: Vec<&str> = request_path.trim_matches('/').split('/').collect();

    if pattern_segments.len() != path_segments.len() {
        return None;
    }

    let mut params = HashMap::new();

    for (pattern_seg, path_seg) in pattern_segments.iter().zip(path_segments.iter()) {
        if pattern_seg.starts_with('{') && pattern_seg.ends_with('}') {
            // This is a path parameter
            let param_name = &pattern_seg[1..pattern_seg.len() - 1];
            params.insert(param_name.to_string(), path_seg.to_string());
        } else if pattern_seg != path_seg {
            // Literal segment doesn't match
            return None;
        }
    }

    Some(PathMatch { params })
}

/// Find the first matching response for a request from an endpoint
pub fn find_matching_response<'a>(
    endpoint: &'a MockEndpoint,
    request: &RequestInfo,
    path_params: &HashMap<String, String>,
) -> Option<&'a MockResponse> {
    for response in &endpoint.responses {
        if matches_conditions(response, request, path_params) {
            return Some(response);
        }
    }
    None
}

/// Check if all conditions in a response match the request
fn matches_conditions(
    response: &MockResponse,
    request: &RequestInfo,
    path_params: &HashMap<String, String>,
) -> bool {
    let condition = match &response.when {
        Some(cond) => cond,
        None => return true, // No conditions means this is a default/fallback response
    };

    // If condition exists but is empty, treat as default
    if condition.is_empty() {
        return true;
    }

    // Check path parameters
    if !check_path_params(condition, path_params) {
        return false;
    }

    // Check query parameters
    if !check_query_params(condition, request) {
        return false;
    }

    // Check query contains
    if !check_query_contains(condition, request) {
        return false;
    }

    // Check headers
    if !check_headers(condition, request) {
        return false;
    }

    // Check header contains
    if !check_header_contains(condition, request) {
        return false;
    }

    // Check body contains
    if !check_body_contains(condition, request) {
        return false;
    }

    // Check body JSON
    if !check_body_json(condition, request) {
        return false;
    }

    // Check body regex
    if !check_body_regex(condition, request) {
        return false;
    }

    true
}

fn check_path_params(condition: &ResponseCondition, path_params: &HashMap<String, String>) -> bool {
    for (key, expected_value) in &condition.path_params {
        match path_params.get(key) {
            Some(actual_value) if actual_value == expected_value => continue,
            _ => return false,
        }
    }
    true
}

fn check_query_params(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    if condition.query_params.is_empty() {
        return true;
    }

    let query_map = parse_query_string(request.query_string.as_deref().unwrap_or(""));

    for (key, expected_value) in &condition.query_params {
        match query_map.get(key) {
            Some(actual_value) if actual_value == expected_value => continue,
            _ => return false,
        }
    }
    true
}

fn check_query_contains(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    match (&condition.query_contains, &request.query_string) {
        (Some(substring), Some(query)) => query.contains(substring),
        (Some(_), None) => false,
        (None, _) => true,
    }
}

fn check_headers(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    for (key, expected_value) in &condition.headers {
        let key_lower = key.to_lowercase();
        match request.headers.get(&key_lower) {
            Some(actual_value) if actual_value == expected_value => continue,
            _ => return false,
        }
    }
    true
}

fn check_header_contains(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    for (key, substring) in &condition.header_contains {
        let key_lower = key.to_lowercase();
        match request.headers.get(&key_lower) {
            Some(actual_value) if actual_value.contains(substring) => continue,
            _ => return false,
        }
    }
    true
}

fn check_body_contains(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    match &condition.body_contains {
        Some(substring) => request.body.contains(substring),
        None => true,
    }
}

fn check_body_json(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    if condition.body_json.is_empty() {
        return true;
    }

    let body_value: serde_json::Value = match serde_json::from_str(&request.body) {
        Ok(v) => v,
        Err(_) => return false,
    };

    let body_obj = match body_value.as_object() {
        Some(obj) => obj,
        None => return false,
    };

    for (key, expected_value) in &condition.body_json {
        match body_obj.get(key) {
            Some(actual_value) if actual_value == expected_value => continue,
            _ => return false,
        }
    }
    true
}

fn check_body_regex(condition: &ResponseCondition, request: &RequestInfo) -> bool {
    match &condition.body_regex {
        Some(pattern) => {
            match Regex::new(pattern) {
                Ok(re) => re.is_match(&request.body),
                Err(_) => false, // Invalid regex pattern
            }
        }
        None => true,
    }
}

fn parse_query_string(query: &str) -> HashMap<String, String> {
    let mut map = HashMap::new();
    for pair in query.split('&') {
        if let Some((key, value)) = pair.split_once('=') {
            map.insert(key.to_string(), value.to_string());
        }
    }
    map
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mock::{MockEndpoint, MockResponse, ResponseCondition};

    fn create_request(
        query: Option<&str>,
        headers: Vec<(&str, &str)>,
        body: &str,
    ) -> RequestInfo {
        RequestInfo {
            query_string: query.map(|s| s.to_string()),
            headers: headers
                .into_iter()
                .map(|(k, v)| (k.to_lowercase(), v.to_string()))
                .collect(),
            body: body.to_string(),
        }
    }

    fn create_endpoint(path: &str, method: &str, responses: Vec<MockResponse>) -> MockEndpoint {
        MockEndpoint {
            path: path.to_string(),
            method: method.to_string(),
            responses,
        }
    }

    fn create_response(status: u16, condition: Option<ResponseCondition>) -> MockResponse {
        MockResponse {
            when: condition,
            status,
            headers: HashMap::new(),
            body: None,
            body_file: None,
            delay_ms: None,
        }
    }

    // ========== Path Matching Tests ==========

    #[test]
    fn test_match_path_simple() {
        let result = match_path("/api/users", "/api/users");
        assert!(result.is_some());
        assert!(result.unwrap().params.is_empty());
    }

    #[test]
    fn test_match_path_with_param() {
        let result = match_path("/api/users/{id}", "/api/users/123");
        assert!(result.is_some());
        let params = result.unwrap().params;
        assert_eq!(params.get("id"), Some(&"123".to_string()));
    }

    #[test]
    fn test_match_path_multiple_params() {
        let result = match_path("/api/users/{user_id}/posts/{post_id}", "/api/users/42/posts/99");
        assert!(result.is_some());
        let params = result.unwrap().params;
        assert_eq!(params.get("user_id"), Some(&"42".to_string()));
        assert_eq!(params.get("post_id"), Some(&"99".to_string()));
    }

    #[test]
    fn test_match_path_no_match() {
        let result = match_path("/api/users", "/api/products");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_path_different_length() {
        let result = match_path("/api/users/{id}", "/api/users");
        assert!(result.is_none());
    }

    #[test]
    fn test_match_path_trailing_slash() {
        let result = match_path("/api/users/", "/api/users");
        assert!(result.is_some());
    }

    #[test]
    fn test_match_path_root() {
        let result = match_path("/", "/");
        assert!(result.is_some());
    }

    // ========== Path Params Condition Tests ==========

    #[test]
    fn test_condition_path_params_match() {
        let mut condition = ResponseCondition::default();
        condition.path_params.insert("id".to_string(), "123".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/users/{id}", "GET", vec![response]);
        let request = create_request(None, vec![], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "123".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 200);
    }

    #[test]
    fn test_condition_path_params_no_match() {
        let mut condition = ResponseCondition::default();
        condition.path_params.insert("id".to_string(), "123".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/users/{id}", "GET", vec![response]);
        let request = create_request(None, vec![], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "456".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_none());
    }

    // ========== Query Params Condition Tests ==========

    #[test]
    fn test_condition_query_params_match() {
        let mut condition = ResponseCondition::default();
        condition.query_params.insert("sort".to_string(), "asc".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/items", "GET", vec![response]);
        let request = create_request(Some("sort=asc&limit=10"), vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    #[test]
    fn test_condition_query_params_no_match() {
        let mut condition = ResponseCondition::default();
        condition.query_params.insert("sort".to_string(), "asc".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/items", "GET", vec![response]);
        let request = create_request(Some("sort=desc"), vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_none());
    }

    #[test]
    fn test_condition_query_contains() {
        let mut condition = ResponseCondition::default();
        condition.query_contains = Some("category=".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/search", "GET", vec![response]);
        let request = create_request(Some("q=test&category=books"), vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    // ========== Headers Condition Tests ==========

    #[test]
    fn test_condition_headers_exact_match() {
        let mut condition = ResponseCondition::default();
        condition.headers.insert("Authorization".to_string(), "Bearer token123".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/protected", "GET", vec![response]);
        let request = create_request(None, vec![("authorization", "Bearer token123")], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    #[test]
    fn test_condition_header_contains() {
        let mut condition = ResponseCondition::default();
        condition.header_contains.insert("Authorization".to_string(), "Bearer".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/protected", "GET", vec![response]);
        let request = create_request(None, vec![("authorization", "Bearer any-token-here")], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    #[test]
    fn test_condition_header_missing() {
        let mut condition = ResponseCondition::default();
        condition.headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let response = create_response(401, Some(condition));
        let endpoint = create_endpoint("/api/protected", "GET", vec![response]);
        let request = create_request(None, vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_none());
    }

    // ========== Body Contains Condition Tests ==========

    #[test]
    fn test_condition_body_contains() {
        let mut condition = ResponseCondition::default();
        condition.body_contains = Some("\"role\": \"admin\"".to_string());

        let response = create_response(403, Some(condition));
        let endpoint = create_endpoint("/api/users", "POST", vec![response]);
        let request = create_request(None, vec![], r#"{"name": "test", "role": "admin"}"#);

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 403);
    }

    // ========== Body JSON Condition Tests ==========

    #[test]
    fn test_condition_body_json_string_value() {
        let mut condition = ResponseCondition::default();
        condition.body_json.insert("name".to_string(), serde_json::json!("test"));

        let response = create_response(400, Some(condition));
        let endpoint = create_endpoint("/api/users", "POST", vec![response]);
        let request = create_request(None, vec![], r#"{"name": "test", "email": "test@example.com"}"#);

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 400);
    }

    #[test]
    fn test_condition_body_json_number_value() {
        let mut condition = ResponseCondition::default();
        condition.body_json.insert("count".to_string(), serde_json::json!(42));

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/data", "POST", vec![response]);
        let request = create_request(None, vec![], r#"{"count": 42}"#);

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    #[test]
    fn test_condition_body_json_invalid_json() {
        let mut condition = ResponseCondition::default();
        condition.body_json.insert("name".to_string(), serde_json::json!("test"));

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/users", "POST", vec![response]);
        let request = create_request(None, vec![], "not valid json");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_none());
    }

    // ========== Body Regex Condition Tests ==========

    #[test]
    fn test_condition_body_regex_match() {
        let mut condition = ResponseCondition::default();
        condition.body_regex = Some(r#""email":\s*"[^"]+@test\.com""#.to_string());

        let response = create_response(400, Some(condition));
        let endpoint = create_endpoint("/api/users", "POST", vec![response]);
        let request = create_request(None, vec![], r#"{"email": "user@test.com"}"#);

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
    }

    #[test]
    fn test_condition_body_regex_no_match() {
        let mut condition = ResponseCondition::default();
        condition.body_regex = Some(r#"@test\.com"#.to_string());

        let response = create_response(400, Some(condition));
        let endpoint = create_endpoint("/api/users", "POST", vec![response]);
        let request = create_request(None, vec![], r#"{"email": "user@example.com"}"#);

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_none());
    }

    #[test]
    fn test_condition_body_regex_invalid_pattern() {
        let mut condition = ResponseCondition::default();
        condition.body_regex = Some("[invalid".to_string()); // Invalid regex

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/api/test", "POST", vec![response]);
        let request = create_request(None, vec![], "anything");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_none()); // Invalid regex fails to match
    }

    // ========== Multiple Conditions Tests ==========

    #[test]
    fn test_multiple_conditions_all_match() {
        let mut condition = ResponseCondition::default();
        condition.path_params.insert("id".to_string(), "123".to_string());
        condition.query_params.insert("include".to_string(), "details".to_string());
        condition.headers.insert("Accept".to_string(), "application/json".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/users/{id}", "GET", vec![response]);
        let request = create_request(Some("include=details"), vec![("accept", "application/json")], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "123".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_some());
    }

    #[test]
    fn test_multiple_conditions_one_fails() {
        let mut condition = ResponseCondition::default();
        condition.path_params.insert("id".to_string(), "123".to_string());
        condition.query_params.insert("include".to_string(), "details".to_string());

        let response = create_response(200, Some(condition));
        let endpoint = create_endpoint("/users/{id}", "GET", vec![response]);
        let request = create_request(Some("include=summary"), vec![], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "123".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_none()); // query_params condition fails
    }

    // ========== Fallback Response Tests ==========

    #[test]
    fn test_fallback_response_no_condition() {
        let fallback = create_response(404, None);
        let endpoint = create_endpoint("/api/test", "GET", vec![fallback]);
        let request = create_request(None, vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 404);
    }

    #[test]
    fn test_fallback_response_empty_condition() {
        let fallback = create_response(404, Some(ResponseCondition::default()));
        let endpoint = create_endpoint("/api/test", "GET", vec![fallback]);
        let request = create_request(None, vec![], "");

        let matched = find_matching_response(&endpoint, &request, &HashMap::new());
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 404);
    }

    #[test]
    fn test_first_matching_response_wins() {
        let mut cond1 = ResponseCondition::default();
        cond1.path_params.insert("id".to_string(), "1".to_string());

        let responses = vec![
            create_response(200, Some(cond1)),
            create_response(404, None), // fallback
        ];

        let endpoint = create_endpoint("/users/{id}", "GET", responses);
        let request = create_request(None, vec![], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "1".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 200); // First match, not fallback
    }

    #[test]
    fn test_fallback_when_no_specific_match() {
        let mut cond1 = ResponseCondition::default();
        cond1.path_params.insert("id".to_string(), "1".to_string());

        let responses = vec![
            create_response(200, Some(cond1)),
            create_response(404, None), // fallback
        ];

        let endpoint = create_endpoint("/users/{id}", "GET", responses);
        let request = create_request(None, vec![], "");

        let mut path_params = HashMap::new();
        path_params.insert("id".to_string(), "999".to_string());

        let matched = find_matching_response(&endpoint, &request, &path_params);
        assert!(matched.is_some());
        assert_eq!(matched.unwrap().status, 404); // Falls through to fallback
    }
}

