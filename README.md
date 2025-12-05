# Machine Agent API (Rust)

A high-performance Rust-based API that allows you to execute commands on the machine remotely via HTTP requests.

## Features

- **Execute commands synchronously** - Wait for command completion and get results
- **Execute commands asynchronously** - Fire and forget for long-running tasks
- **Error logging** - All errors are logged to `app_error.log`
- **Cross-platform** - Works on Windows, Linux, and macOS

## Installation

### Prerequisites

- Rust (latest stable version)
- Cargo (comes with Rust)

### Build

```bash
cd machine_agent_rs
cargo build --release
```

The compiled binary will be in `target/release/machine_agent.exe` (Windows) or `target/release/machine_agent` (Linux/Mac).

## Running

```bash
# Development mode
cargo run

# Release mode (optimized)
cargo run --release
```

The server will start on `http://0.0.0.0:6565`

## API Endpoints

### Home
```
GET /
```

Returns API information and available endpoints.

### Health Check
```
GET /health
```

Returns the health status of the API.

### Execute Command (Synchronous)
```
POST /execute
Content-Type: application/json

{
    "command": "your command here",
    "timeout": 30  // optional, default 30 seconds
}
```

**Example:**
```bash
curl -X POST http://localhost:6565/execute \
  -H "Content-Type: application/json" \
  -d '{"command": "echo Hello World"}'
```

**Response:**
```json
{
    "success": true,
    "command": "echo Hello World",
    "stdout": "Hello World\n",
    "stderr": "",
    "return_code": 0,
    "executed": true
}
```

### Execute Command (Asynchronous)
```
POST /execute-async
Content-Type: application/json

{
    "command": "your command here"
}
```

**Example:**
```bash
curl -X POST http://localhost:6565/execute-async \
  -H "Content-Type: application/json" \
  -d '{"command": "long-running-task.bat"}'
```

**Response:**
```json
{
    "success": true,
    "message": "Command started successfully",
    "command": "long-running-task.bat",
    "pid": 12345,
    "started_at": "2024-12-04T23:40:33.866070+00:00",
    "status": "running"
}
```

## Error Logging

All errors are automatically logged to `app_error.log` in the same directory as the executable. The log includes:
- Timestamp
- Endpoint
- Error message
- Command (when applicable)
- Full traceback for debugging

## Security Warning

⚠️ **WARNING**: This API allows arbitrary command execution on your machine. Only use this in:
- Trusted networks
- Development environments
- With proper authentication/authorization added

For production use, consider adding:
- Authentication (API keys, JWT tokens)
- Authorization (user permissions)
- Command whitelisting
- Rate limiting
- Input validation and sanitization

## Performance

Rust provides significant performance advantages over Python:
- Lower memory footprint
- Faster execution
- Better concurrency handling
- No GIL (Global Interpreter Lock)

## Comparison with Python Version

| Feature | Python (Flask) | Rust (Actix-web) |
|---------|---------------|------------------|
| Performance | Good | Excellent |
| Memory Usage | Higher | Lower |
| Startup Time | Slower | Faster |
| Concurrency | Limited | Excellent |
| Type Safety | Runtime | Compile-time |

## Building for Production

```bash
# Build optimized release
cargo build --release

# The binary is in target/release/
# On Windows: target/release/machine_agent.exe
# On Linux/Mac: target/release/machine_agent
```

## Dependencies

- `actix-web` - High-performance web framework
- `serde` - Serialization/deserialization
- `tokio` - Async runtime
- `chrono` - Date and time handling

