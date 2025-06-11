# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [0.1.0] - 2024-01-01

### Added
- Initial release of Hitman HTTP testing tool
- Support for `.hit` file format with HTTP request blocks
- Variable system with `DEFINE` statements and `{{variable}}` interpolation
- Support for all HTTP methods: GET, POST, PUT, PATCH, DELETE, HEAD, OPTIONS
- `WITH HEADER`, `WITH DATA`, and `WITH QUERY` clauses for request configuration
- Multiple comment styles: `#`, `//` (line comments), `/* */` (block comments)
- Comprehensive CLI with the following options:
  - `--stdin`: Read from stdin instead of file
  - `--verbose, -v`: Verbose output with detailed information
  - `--quiet, -q`: Suppress non-essential output
  - `--log <level>`: Set log level (debug|info|warn|error)
  - `--strict`: Stop execution on first error
  - `--report <path>`: Save JSON execution report
  - `--dry-run`: Validate syntax without sending requests
  - `--timeout <seconds>`: Request timeout configuration
  - `--color <when>`: Color output control (auto|always|never)
  - `--parallel`: Execute requests in parallel
  - `--delay <ms>`: Delay between sequential requests
  - `--define KEY=VALUE`: Override variables from command line
  - `--env <file>`: Load variables from environment file
- Variable types: strings, numbers, booleans, JSON objects/arrays
- Variable name validation (identifier rules)
- Command line variable overrides with precedence system
- Environment file support (.env format)
- Parallel execution mode
- Configurable delays between requests
- Proper exit codes (0, 1, 2, 3)
- Comprehensive error handling and validation
- JSON report generation
- Smart comment parsing (handles `//` in URLs correctly)
- 58 comprehensive tests covering all functionality

### Features
- **Language Support**: Complete `.hit` file format implementation
- **Variable System**: Full variable definition, interpolation, and override system
- **HTTP Methods**: Support for all standard HTTP methods
- **Request Configuration**: Flexible header, data, and query parameter support
- **Comments**: Multiple comment styles with smart parsing
- **CLI Interface**: Rich command line interface with extensive options
- **Execution Modes**: Sequential, parallel, and dry-run modes
- **Output Control**: Verbose, quiet, and configurable log levels
- **Error Handling**: Comprehensive error reporting with proper exit codes
- **Testing**: Extensive test suite with unit, integration, and CLI tests

### Technical Details
- Built with Rust for performance and reliability
- Uses `clap` for CLI parsing
- Uses `reqwest` for HTTP client functionality
- Uses `serde_json` for JSON handling
- Uses `tokio` for async execution and parallel processing
- Comprehensive error handling with `thiserror`
- Colored output support with `colored` crate
- Terminal detection with `atty` crate
