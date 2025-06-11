# Hitman 🎯

A lightweight HTTP testing tool that reads `.hit` files and executes HTTP requests with support for variables, comments, parallel execution, and comprehensive reporting.

## Features

### Core Features
- 📝 **Simple Syntax**: Clean `.hit` file format with multiple comment styles (`#`, `//`, `/* */`)
- 🔧 **All HTTP Methods**: GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS
- 📋 **Flexible Headers**: Multiple headers with any order using `WITH HEADER`
- 📊 **Request Body**: JSON data support with `WITH DATA`
- 🔍 **Query Parameters**: URL query parameters with `WITH QUERY`
- 🧪 **JSON Validation**: Built-in JSON syntax validation

### Variables & Configuration
- 🔤 **Variables**: Define and use variables with `DEFINE` statements and `{{variable}}` interpolation
- ⚙️ **Command Line Overrides**: Override variables with `--define KEY=VALUE`
- 📁 **Environment Files**: Load variables from `.env` files with `--env`
- 🔧 **Multiple Value Types**: Strings, numbers, booleans, JSON objects/arrays

### Execution & Output
- 🚀 **Parallel Execution**: Run requests concurrently with `--parallel`
- ⏱️ **Request Delays**: Add delays between requests with `--delay`
- 📊 **Multiple Log Levels**: Debug, info, warn, error with `--log`
- 📄 **JSON Reports**: Structured execution reports with `--report`
- 🔍 **Dry Run Mode**: Validate syntax without sending requests
- 🤫 **Quiet Mode**: Suppress output with `--quiet`
- 📥 **Stdin Support**: Read from stdin with `--stdin`

### Quality & Reliability
- ⚡ **Fast & Reliable**: Built with Rust for performance
- 🎨 **Color Output**: Configurable ANSI colors
- 🚫 **Strict Mode**: Fail-fast error handling
- ✅ **Comprehensive Testing**: 58 tests covering all functionality
- 🔢 **Proper Exit Codes**: 0 (success), 1 (parse errors), 2 (runtime errors), 3 (invalid args)

## Installation

### Pre-built Binaries (Recommended)

Download the latest release for your platform from the [GitHub Releases](https://github.com/Mahmoud-Emad/hitman-cli/releases) page:

- **Linux (x86_64)**: `hitman-linux-x86_64.tar.gz`
- **Linux (x86_64, musl)**: `hitman-linux-x86_64-musl.tar.gz`
- **Linux (ARM64)**: `hitman-linux-aarch64.tar.gz`
- **macOS (Intel)**: `hitman-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `hitman-macos-aarch64.tar.gz`
- **Windows (x86_64)**: `hitman-windows-x86_64.zip`
- **Windows (ARM64)**: `hitman-windows-aarch64.zip`

```bash
# Example for Linux x86_64
curl -L https://github.com/Mahmoud-Emad/hitman-cli/releases/latest/download/hitman-linux-x86_64.tar.gz | tar xz
sudo mv hitman /usr/local/bin/
```

### From Cargo
```bash
cargo install hitman
```

### From Source
```bash
# Clone the repository
git clone https://github.com/Mahmoud-Emad/hitman-cli.git
cd hitman

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .
```

### Package Managers

#### Homebrew (macOS/Linux)
```bash
# Coming soon
brew install hitman
```

#### Chocolatey (Windows)
```bash
# Coming soon
choco install hitman
```

## Quick Start

### Basic Usage
```bash
# Execute requests from a file
hitman requests.hit

# Read from stdin
echo "GET https://httpbin.org/get" | hitman --stdin

# Dry run (validate only)
hitman --dry-run requests.hit

# Verbose output
hitman --verbose requests.hit

# Quiet mode (minimal output)
hitman --quiet requests.hit
```

### Advanced Usage
```bash
# Override variables from command line
hitman --define API_URL=https://api.example.com --define TOKEN=abc123 requests.hit

# Load variables from environment file
hitman --env .env requests.hit

# Parallel execution with delays
hitman --parallel --delay 100 requests.hit

# Generate JSON report
hitman --report report.json requests.hit

# Strict mode (fail on first error)
hitman --strict requests.hit

# Custom timeout and log level
hitman --timeout 10 --log debug --color never requests.hit
```

### CLI Options

| Flag | Description |
|------|-------------|
| `--stdin` | Read .hit content from stdin instead of a file |
| `--verbose, -v` | Enable verbose output with detailed information |
| `--quiet, -q` | Suppress non-essential output |
| `--log <LEVEL>` | Set log level: debug, info, warn, error (default: info) |
| `--strict` | Stop execution on first error |
| `--report <PATH>` | Save JSON execution report |
| `--dry-run` | Validate syntax without sending requests |
| `--timeout <SECONDS>` | Request timeout in seconds (default: 30) |
| `--color <WHEN>` | Color output: auto, always, never |
| `--parallel` | Execute requests in parallel when possible |
| `--delay <MS>` | Delay between requests in milliseconds |
| `--define KEY=VALUE` | Override or add variables (can be used multiple times) |
| `--env <FILE>` | Load variables from environment file |

## File Format

The `.hit` file format supports variables, multiple comment styles, and HTTP request blocks:

### Variables
```hit
# Define variables at the top of the file
DEFINE baseUrl="https://api.example.com"
DEFINE token="abc123"
DEFINE timeout=30
DEFINE debug=true
DEFINE user={"name": "John", "role": "admin"}

# Use variables with {{}} syntax
GET {{baseUrl}}/users
    WITH HEADER {
        "Authorization": "Bearer {{token}}",
        "User-Agent": "Hitman/1.0"
    }
```

### Comments
```hit
# Line comment with hash
// Line comment with double slash

/* Block comment
   spanning multiple lines */

DEFINE api="https://api.example.com" # Inline comment
DEFINE key="secret" // Another inline comment
DEFINE flag=true; # Semicolon with comment
```

### HTTP Requests
```hit
# Basic GET request
GET https://httpbin.org/get

# POST with headers and JSON data
POST {{baseUrl}}/users
    WITH HEADER {Content-Type: application/json}
    WITH DATA {"name": "John", "job": "developer"}

# Request with query parameters
GET {{baseUrl}}/search
    WITH QUERY {q: "rust", limit: 10}
    WITH HEADER {Authorization: "Bearer {{token}}"}

# All WITH clauses can be in any order
PUT {{baseUrl}}/users/1
    WITH DATA {"name": "Jane"}
    WITH HEADER {Authorization: "Bearer {{token}}"}
    WITH QUERY {include: "profile"}
```

### Syntax Rules

- **Variables**: `DEFINE name=value` (must be at the top, before HTTP requests)
- **Variable Types**: Strings (`"text"`), numbers (`123`), booleans (`true`/`false`), JSON objects/arrays
- **Variable Names**: Must start with letter or underscore, contain only letters, numbers, underscores
- **Variable Usage**: `{{variableName}}` anywhere in URLs, headers, data, or query parameters
- **Comments**: `#`, `//` (line comments), `/* */` (block comments)
- **HTTP Requests**: `METHOD URL` followed by optional `WITH` clauses
- **WITH Clauses**: `WITH HEADER {...}`, `WITH DATA {...}`, `WITH QUERY {...}`
- **Blocks**: HTTP requests separated by empty lines
- **Order**: `WITH` clauses can be in any order within a block

## Example Output

### Verbose Mode
```bash
$ hitman --verbose example.hit

Found 2 HTTP command block(s) in example.hit

=== Block 1 ===
→ GET https://httpbin.org/get?name=John
  Headers:
    Authorization: Bearer abc123
    User-Agent: Hitman/1.0
← 200 OK (245ms, 1520 bytes)
  Response Headers:
    content-type: application/json
    server: gunicorn/19.9.0

=== Block 2 ===
→ POST https://httpbin.org/post
  Headers:
    Content-Type: application/json
  Body:
    {"name": "John", "job": "developer"}
← 200 OK (308ms, 1872 bytes)

📊 Summary:
  ✅ Successful requests: 2
  📁 Total blocks processed: 2
  ⏱️  Total execution time: 0.61s
```

### Quiet Mode
```bash
$ hitman --quiet example.hit
# No output unless there are errors
```

### Dry Run
```bash
$ hitman --dry-run example.hit

Found 2 HTTP command block(s) in example.hit

=== Block 1 ===
  🔍 Dry run - request not executed

=== Block 2 ===
  🔍 Dry run - request not executed

🔍 Dry run completed:
  📁 Total blocks validated: 2
  ❌ Parse errors: 0
  ⏱️  Validation time: 0.01s
```

## Environment Files

Create a `.env` file to define variables:

```env
# API Configuration
API_URL=https://api.example.com
API_KEY=your-secret-key
TIMEOUT=30
DEBUG=true

# User Data
USER_NAME=john_doe
USER_EMAIL=john@example.com
```

Use with the `--env` flag:
```bash
hitman --env .env requests.hit
```

## Variable Precedence

Variables are resolved in the following order (highest to lowest priority):

1. **Command line** (`--define KEY=VALUE`)
2. **Environment file** (`--env file.env`)
3. **File variables** (`DEFINE` statements in .hit file)

## Exit Codes

| Code | Meaning |
|------|---------|
| 0 | Success - all requests completed successfully |
| 1 | Parse/syntax errors in .hit file |
| 2 | Runtime/network errors during execution |
| 3 | Invalid command line arguments |

## Development

```bash
# Run all tests
cargo test

# Run specific test
cargo test test_variables

# Build release
cargo build --release

# Install locally for development
cargo install --path .

# Check code formatting
cargo fmt

# Run linter
cargo clippy
```

### Test Coverage
- **58 total tests** covering all functionality
- **Unit tests** for parsing, variables, execution
- **Integration tests** for end-to-end workflows
- **CLI tests** for command line interface

## Contributing

1. Fork the repository
2. Create a feature branch (`git checkout -b feature/amazing-feature`)
3. Make your changes
4. Add tests for new functionality
5. Ensure all tests pass (`cargo test`)
6. Commit your changes (`git commit -m 'Add amazing feature'`)
7. Push to the branch (`git push origin feature/amazing-feature`)
8. Open a Pull Request

## License

This project is licensed under either of

- Apache License, Version 2.0, ([LICENSE-APACHE](LICENSE-APACHE) or http://www.apache.org/licenses/LICENSE-2.0)
- MIT license ([LICENSE-MIT](LICENSE-MIT) or http://opensource.org/licenses/MIT)

at your option.
