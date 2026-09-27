# jimjam 🎭

A configurable HTTP mock server that serves responses based on YAML-defined rules. Perfect for API mocking, testing, and development.

---

## 🤖 Jimjam for AI Coding Agents & LLM Workflows

Jimjam is designed to be an **Agentic-Native HTTP Mock Server**. While humans love using Jimjam for its simplicity, **AI coding agents (like Gemini, Cursor, Copilot, and Claude) find it significantly more reliable than writing custom mock scripts in code** because it avoids:

- ❌ Writing boilerplate mock servers in Node/Python/Go with fragile route handlers.
- ❌ Port collisions, background daemon hangs, and unhandled shutdown states.
- ❌ Heavy test fixture bloat and brittle stubbing frameworks.

By providing a declarative YAML interface, agents can rapidly stand up mock APIs, simulate complex edge cases and latencies, and execute deterministic integration tests. Read the full [Agentic Mocking Guide](./AGENT_README.md) for instructions and prompt templates.

---

## Features

- **YAML-based configuration** - Define mock responses in simple YAML files
- **Multiple response scenarios** - Define different responses for the same endpoint based on conditions
- **Path parameters** - Support for dynamic URL segments like `/users/{id}`
- **Response delays** - Simulate slow APIs with configurable delays
- **File-based bodies** - Load large response bodies from external files

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

### Mock Definitions (`mocks/users.yaml`)

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
          Content-Type: "application/json"
        body: |
          {"id": 1, "name": "Alice"}

      # Fallback response (no conditions)
      - status: 404
        body: |
          {"error": "User not found"}
```

### Matching Conditions

| Condition         | Description                 | Example                         |
| ----------------- | --------------------------- | ------------------------------- |
| `path_params`     | Match URL path parameters   | `id: "123"`                     |
| `query_params`    | Exact query parameter match | `sort: "asc"`                   |
| `query_contains`  | Query string substring      | `"category=books"`              |
| `headers`         | Exact header match          | `Authorization: "Bearer token"` |
| `header_contains` | Header substring            | `Authorization: "Bearer"`       |
| `body_contains`   | Body substring              | `'"role": "admin"'`             |
| `body_json`       | JSON field match            | `name: "test"`                  |
| `body_regex`      | Regex pattern               | `'"email":\\s*".*@test\\.com"'` |

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
1. **File reference with @** - Prefix path with `@` (recommended)
   ```yaml
   body: "@./mocks/data/users.json"
   ```
1. **Explicit body_file** - Use separate field
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

## 📖 Documentation Directory

| Document | Description |
| :--- | :--- |
| 🤖 [**Agentic Mocking Guide**](./AGENT_README.md) | Comprehensive instructions, schema reference, and prompt templates for AI coding agents |
| 📖 [**How Jimjam Works**](./how-jimjam-works.md) | Detailed schema, matching conditions, and runtime architecture |
| 🧪 [**Testing with Ramjam**](./ramjam-test/how-to-test-with-ramjam.md) | Guide for validating Jimjam endpoints using Ramjam declarative workflows |
