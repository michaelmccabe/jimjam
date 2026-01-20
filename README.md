# jimjam 🎭

A configurable HTTP mock server that serves responses based on YAML-defined rules. Perfect for API mocking, testing, and development.

## Features

* **YAML-based configuration** - Define mock responses in simple YAML files
* **Multiple response scenarios** - Define different responses for the same endpoint based on conditions
* **Path parameters** - Support for dynamic URL segments like `/users/{id}`
* **Response delays** - Simulate slow APIs with configurable delays
* **File-based bodies** - Load large response bodies from external files

## Quick Start

### Build

```bash
cargo build --release
```

### Run

```bash
cargo run
```

The server starts on `http://127.0.0.1:8080` by default.

### Test

```bash
curl http://127.0.0.1:8080/api/health
# → {"status": "ok", "service": "jimjam"}
```

## Configuration

### Main Config (`config/config.yaml`)

```yaml
# jimjam main configuration

server:
  host: "127.0.0.1"
  port: 8080

# Location of mock response definition files
mock_files:
  directory: "./mocks"
  patterns:
    - "**/*.yaml"
    - "**/*.yml"
  hot_reload: true
```

mocks:

* path: "/api/users/{id}"
  id: "1"
  status: 200
  {"id": 1, "name": "Alice"}

  # Fallback response (no conditions)

| Condition | Description | Example |
|----|----|----|
| `path_params` | Match URL path parameters | `id: "123"` |
| `query_params` | Exact query parameter match | `sort: "asc"` |
| `query_contains` | Query string substring | `"category=books"` |
| `headers` | Exact header match | `Authorization: "Bearer token"` |
| `header_contains` | Header substring | `Authorization: "Bearer"` |
| `body_contains` | Body substring | `'"role": "admin"'` |
| `body_json` | JSON field match | `name: "test"` |
| `body_regex` | Regex pattern | `'"email":\\s*".*@test\\.com"'` |

## Response Options

```yaml
- status: 201                    # HTTP status code
  headers:                       # Response headers
    Content-Type: "application/json"
    X-Custom-Header: "value"
  body: |                        # Inline response body
    {"created": true}
  body: "@./data/large.json"     # Or use @ prefix to load from file
  body_file: "./data/large.json" # Alternative: explicit file reference
  delay_ms: 2000                 # Simulate slow response
```

### Body Content Options




1. **Inline body** - Write content directly in YAML

   ```yaml
   body: '{"name": "example"}'
   ```
2. **File reference with @** - Prefix path with `@` (recommended)

   ```yaml
   body: "@./mocks/data/users.json"
   ```
3. **Explicit body_file** - Use separate field

   ```yaml
   body_file: "./mocks/data/users.json"
   ```

## Directory Structure

```
jimjam/
├── config/
│   └── config.yaml       # Server configuration
├── mocks/
│   ├── users.yaml        # User API mocks
│   ├── products.yaml     # Product API mocks
│   └── data/
│       └── products.json # Large response bodies
└── src/
    └── ...
```

## Running Tests

```bash
cargo test
```

## How jimjam works

See [how-jimjam-works.md](./how-jimjam-works.md) for schema, matching, and advanced usage.