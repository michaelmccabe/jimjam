# How Jimjam Works

Jimjam is a configurable HTTP mock server. You define endpoints and their responses in YAML files; Jimjam loads them and serves matches based on the incoming request.

## Overview
- Endpoints are defined with a `path` (supports `{param}` placeholders) and `method` (e.g., `GET`, `POST`).
- Each endpoint has an ordered list of `responses`. The first matching response wins; a response with no `when` acts as a fallback.
- Conditions can match on path params, query string, headers, body substrings, JSON key/value pairs, or regex.
- Response bodies can be inline strings or loaded from files (via `body_file:` or `body: "@path"`).
- Optional `delay_ms` simulates latency.

## Configuration
- Main config file: `./config/config.yaml`
- Controls where mock YAML files live and what glob patterns to load.

Example `config.yaml`:
```yaml
server:
  host: 127.0.0.1
  port: 8080
mock_files:
  directory: "./mocks"
  patterns:
    - "**/*.yaml"
    - "**/*.yml"
```

Jimjam loads all matching files via glob patterns and merges all endpoints.

## Mock File Schema
A mock file contains a `mocks` array. Each item is an endpoint:
```yaml
mocks:
  - path: "/api/users/{id}"
    method: GET
    responses:
      - when:
          path_params:
            id: "1"
        status: 200
        headers:
          Content-Type: application/json
        body: |
          {"id": 1, "name": "Alice"}

      - status: 404  # fallback when no conditions match
        body: |
          {"error": "User not found"}
```

### Endpoint Fields
- `path`: URL pattern. You can use `{param}` placeholders (e.g., `/api/users/{id}`).
- `method`: HTTP method (GET, POST, PUT, DELETE, etc.).
- `responses`: ordered list; first that matches is used.

### Response Fields
- `when` (optional): conditions to match the request.
  - `path_params`: exact matches for extracted placeholders (e.g., `{id}` must equal `"123"`).
  - `query_params`: exact key/value match on query string (no URL decoding).
  - `query_contains`: substring present anywhere in query (e.g., `category=books`).
  - `headers`: exact header value match; header names are matched case-insensitively.
  - `header_contains`: header value must contain substring.
  - `body_contains`: substring match on raw request body.
  - `body_json`: JSON top-level key/value equality (strict type/value match).
  - `body_regex`: regex applied to raw body string.
- `status`: HTTP status code (default `200`).
- `headers`: response headers.
- `body`: inline response body. If it starts with `@`, the remainder is treated as a file path to read.
- `body_file`: path to file containing body content.
- `delay_ms`: add an artificial delay (milliseconds).

## Matching Details
1. Filter endpoints by `method` and `path` pattern. Path must match segment count and literal segments; `{param}` segments capture values.
2. Evaluate `responses` in order:
   - If `when` is absent or empty, it's a fallback and will match.
   - Otherwise all specified conditions must pass.
3. The first response that matches is returned.

Notes:
- Header names are normalized to lowercase internally; you can write them in YAML as usual (`Authorization`, `Content-Type`).
- Query parsing is simple `key=value` pairs split by `&` (no decoding). Provide values exactly as they appear.
- `body_json` checks only top-level keys and requires exact value equality (including types).
- If a body is present and no `Content-Type` is set, Jimjam defaults to `application/json`.

## File-Based Bodies
You can reference external files for response bodies:
```yaml
responses:
  - status: 200
    headers:
      Content-Type: application/json
    body: "@./mocks/data/products.json"  # using '@' prefix
```
Or:
```yaml
responses:
  - status: 200
    body_file: "./mocks/data/products.json"
```

## End-to-End Flow
- Config is loaded from `./config/config.yaml`.
- Jimjam discovers mock files using `directory` + `patterns` and parses them with YAML.
- For each request, it logs the method and path, matches endpoints, evaluates conditions, and builds the response.

## Running
From the project root:
```bash
cargo run
```
Default server address is `http://127.0.0.1:8080` (configurable in `config.yaml`).

## Quick Examples
- Health check:
```bash
curl -i http://127.0.0.1:8080/api/health
```
- Path params:
```bash
curl -i http://127.0.0.1:8080/api/users/1
```
- Query matching:
```bash
curl -i "http://127.0.0.1:8080/api/search?q=error"
curl -i "http://127.0.0.1:8080/api/search?q=test&category=books"
```
- Headers:
```bash
curl -i -H "Authorization: Bearer valid-token-123" http://127.0.0.1:8080/api/protected
```
- Body JSON:
```bash
curl -i -X POST -H "Content-Type: application/json" \
  -d '{"name":"test"}' http://127.0.0.1:8080/api/users
```

## Implementation Pointers
- Config types and loaders: `src/config.rs`
- Mock schema types and YAML parsing: `src/mock.rs`
- Matching logic: `src/matcher.rs`
- HTTP server and request handling: `src/server.rs`

If you'd like additional examples or a stricter query parser (URL-decoding, multi-value params), we can extend the matcher accordingly.
