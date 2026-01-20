# How to Test Jimjam with Ramjam

This guide explains how to use [ramjam](https://github.com/michaelmccabe/ramjam) to test your jimjam mock server.

## What is Ramjam?

Ramjam is a CLI tool for executing HTTP API workflows defined in YAML files. It allows you to:

* Define HTTP requests and expected responses
* Capture values from responses for use in later steps
* Chain requests together in workflows
* Validate responses with JSONPath assertions

## Prerequisites

You need to have support for go on your machine.

### **Install ramjam**

[ramjam](https://github.com/michaelmccabe/ramjam/blob/main/README.md)  is a testing tool written in go.

Install directly to your `$GOPATH/bin` or `$GOBIN`:

```javascript
make install
```

This will compile and install the `ramjam` binary to your Go bin directory.

Alternatively, download from [ramjam releases](https://github.com/michaelmccabe/ramjam/releases) and install locally.

### **Start jimjam**

```bash
cd /path/to/jimjam
cargo run
```


jimjam will start on `http://127.0.0.1:8080` by default.

## Running Tests

### Run a Single Test

```bash
ramjam run ramjam-test/health-check.yaml
```

### Run All Tests in the Folder

```bash
ramjam run ramjam-test/
```

### Run with Verbose Output

```bash
ramjam run ramjam-test/ --verbose
```

### Run Specific Tests

```bash
ramjam run ramjam-test/path-params.yaml ramjam-test/headers.yaml
```

## Test Files Overview

| File | Description |
|----|----|
| `health-check.yaml` | Basic server health check |
| `path-params.yaml` | Tests path parameter matching (`/api/users/{id}`) |
| `query-params.yaml` | Tests query string matching |
| `headers.yaml` | Tests header-based conditional responses |
| `post-body.yaml` | Tests POST body matching (contains, JSON, regex) |
| `file-reference.yaml` | Tests body loaded from external JSON file |
| `admin-reload.yaml` | Tests the `/__admin/reload` hot-reload endpoint |
| `full-workflow.yaml` | End-to-end workflow with variable capture |

## Test File Structure

Each ramjam test file follows this structure:

```yaml
metadata:
  name: "Test Name"
  author: "Author"
  description: "What this test does"

config:
  base_url: "http://127.0.0.1:8080"

workflow:
  - step: "step-id"
    description: "Step description"
    request:
      method: "GET"
      url: "${base_url}/api/endpoint"
      headers:
        Accept: "application/json"
    expect:
      status: 200
      json_path_match:
        - path: "field"
          value: "expected"
    capture:
      - json_path: "id"
        as: "captured_id"
    output:
      print: "Step completed with ID: ${captured_id}"
```

## Key Features

### Variable Capture and Substitution

Capture values from responses and use them in later steps:

```yaml
capture:
  - json_path: "id"
    as: "user_id"
# Later use: ${user_id}
```

### JSONPath Assertions

Validate response JSON:

```yaml
expect:
  json_path_match:
    - path: "data.name"
      value: "Alice"
    - path: "items[0].id"
      value: 1
```

### Header Validation

Check response headers:

```yaml
expect:
  headers:
    - name: "Content-Type"
      contains: "application/json"
    - name: "X-Request-Id"
      value: "mock-12345"
```

### Request Body

Send JSON body in POST/PUT requests:

```yaml
request:
  method: "POST"
  url: "${base_url}/api/users"
  headers:
    Content-Type: "application/json"
  body:
    name: "New User"
    email: "user@example.com"
```

## Writing New Tests




1. Create a new `.yaml` file in `ramjam-test/`
2. Define metadata and config
3. Add workflow steps
4. Run with `ramjam run ramjam-test/your-test.yaml`

### Example: Testing a New Mock Endpoint

If you add a new endpoint to `mocks/example.yaml`:

```yaml
# In mocks/example.yaml
- path: "/api/orders/{orderId}"
  method: GET
  responses:
    - when:
        path_params:
          orderId: "12345"
      status: 200
      body: |
        {"orderId": "12345", "status": "shipped"}
    - status: 404
      body: |
        {"error": "Order not found"}
```


Create a test in `ramjam-test/orders.yaml`

```yaml
metadata:
  name: "Orders Test"
  author: "jimjam"
  description: "Test order endpoint"

config:
  base_url: "http://127.0.0.1:8080"

workflow:
  - step: "get-order"
    description: "Fetch order 12345"
    request:
      method: "GET"
      url: "${base_url}/api/orders/12345"
    expect:
      status: 200
      json_path_match:
        - path: "orderId"
          value: "12345"
        - path: "status"
          value: "shipped"
    output:
      print: "Order found with correct status"

  - step: "order-not-found"
    description: "Fetch unknown order"
    request:
      method: "GET"
      url: "${base_url}/api/orders/99999"
    expect:
      status: 404
    output:
      print: "Unknown order correctly returns 404"
```

## CI/CD Integration

Add ramjam tests to your CI pipeline:

```yaml
# .github/workflows/test.yml
jobs:
  integration-tests:
    runs-on: ubuntu-latest
    steps:
      - uses: actions/checkout@v4
      
      - name: Install Rust
        uses: dtolnay/rust-action@stable
      
      - name: Install ramjam
        run: cargo install ramjam
      
      - name: Start jimjam
        run: cargo run &
        
      - name: Wait for server
        run: sleep 3
        
      - name: Run ramjam tests
        run: ramjam run ramjam-test/
```

## Troubleshooting

### Server not responding

Make sure jimjam is running on the correct port:

```bash
curl http://127.0.0.1:8080/api/health
```

### Tests failing after mock changes

If you're using hot reload, changes are picked up automatically. Otherwise, restart jimjam or trigger a reload:

```bash
curl -X POST http://127.0.0.1:8080/__admin/reload
```

### JSONPath not matching

Use verbose mode to see the actual response:

```bash
ramjam run ramjam-test/your-test.yaml --verbose
```

## Further Reading

* [Ramjam Documentation](https://github.com/michaelmccabe/ramjam/blob/main/RAMJAM.md)
* [Jimjam Mock Schema](./../.instructions/how-jijam-works.md)


