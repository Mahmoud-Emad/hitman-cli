# Hitman 🎯

[![VS Code Extension](https://img.shields.io/visual-studio-marketplace/v/hitman-dev.hitman-http-scripting?label=VS%20Code%20Extension&logo=visual-studio-code&color=blue)](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)
[![Crates.io](https://img.shields.io/crates/v/hitman?logo=rust)](https://crates.io/crates/hitman-cli)
[![GitHub Release](https://img.shields.io/github/v/release/Mahmoud-Emad/hitman-cli?logo=github)](https://github.com/Mahmoud-Emad/hitman-cli/releases)

A lightweight, fast, and intuitive HTTP testing tool that reads `.hit` files and executes HTTP requests with support for variables, comments, and parallel execution.

## ✨ Features

- 🚀 **Simple Syntax**: Easy-to-read `.hit` file format
- 🔧 **Variable Support**: Define and reuse variables with `{{variable}}` syntax
- 📝 **Multiple Comment Styles**: `#`, `//`, and `/* */` comments
- ⚡ **Parallel Execution**: Run requests concurrently for faster testing
- 🎨 **Rich CLI**: Comprehensive command-line interface with colored output
- 📊 **JSON Reports**: Generate detailed execution reports
- 🔍 **Dry Run Mode**: Validate syntax without sending requests
- 🌐 **Cross-Platform**: Available for Linux, macOS, and Windows
- 🎯 **VS Code Extension**: Official extension with syntax highlighting and IntelliSense

## 📦 Installation

### Pre-built Binaries

**[Download from GitHub Releases →](https://github.com/Mahmoud-Emad/hitman-cli/releases)**

- **Linux (x86_64)**: `hitman-linux-x86_64.tar.gz`
- **Linux (ARM64)**: `hitman-linux-aarch64.tar.gz`
- **macOS (Intel)**: `hitman-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `hitman-macos-aarch64.tar.gz`
- **Windows (x86_64)**: `hitman-windows-x86_64.zip`

### Package Managers

```bash
# Rust/Cargo
cargo install hitman

# Homebrew (coming soon)
brew install hitman
```

## 🚀 Quick Start

### 1. Create a `.hit` file

```hit
# Define variables
DEFINE baseUrl="https://jsonplaceholder.typicode.com"
DEFINE userId=1

# Simple GET request
GET {{baseUrl}}/users/{{userId}}

# POST request with data
POST {{baseUrl}}/posts
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "title": "My Post",
        "body": "This is the post content",
        "userId": {{userId}}
    }
```

### 2. Run it

```bash
# Execute requests
hitman example.hit

# Dry run (validate without sending)
hitman example.hit --dry-run

# Verbose output
hitman example.hit --verbose

# Parallel execution
hitman example.hit --parallel
```

## 🎨 VS Code Extension

Enhance your `.hit` file editing experience:

**[Install from VS Code Marketplace →](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)**

Features:
- 🎨 Syntax highlighting
- 📝 IntelliSense and autocomplete
- ❌ Error detection and validation
- 🔧 Code formatting
- 📋 Snippets and templates

## 🔧 Key Features

### Variables and Substitution
```hit
DEFINE token="abc123"
DEFINE user={"name": "John", "age": 30}

GET https://api.example.com/profile
    WITH HEADER {"Authorization": "Bearer {{token}}"}
    WITH DATA {{user}}
```

### Multiple HTTP Methods
```hit
GET https://api.example.com/users
POST https://api.example.com/users
PUT https://api.example.com/users/1
DELETE https://api.example.com/users/1
```

### Flexible Comments
```hit
# Hash comments
// Double slash comments
/* Block comments */

GET https://api.example.com/users  # Inline comments
```

## 🛠️ CLI Options

```bash
# Basic usage
hitman requests.hit

# Common options
hitman requests.hit --dry-run          # Validate without sending
hitman requests.hit --verbose          # Detailed output
hitman requests.hit --parallel         # Concurrent execution
hitman requests.hit --quiet            # Minimal output

# Variable overrides
hitman requests.hit --define baseUrl="https://staging.api.com"
hitman requests.hit --env staging.env

# Reporting
hitman requests.hit --report results.json
```

## 📖 Documentation

- **[Installation Guide](docs/installation.md)** - Detailed installation instructions
- **[Usage Guide](docs/usage.md)** - Examples and common patterns  
- **[File Format](docs/file-format.md)** - Complete `.hit` syntax reference
- **[CLI Reference](docs/cli-reference.md)** - All command-line options
- **[Cross-Platform Setup](CROSS_PLATFORM_SETUP.md)** - Build and deployment info

## 🤝 Contributing

We welcome contributions! Please see our [Contributing Guide](CONTRIBUTING.md) for details.

### Development Setup

```bash
# Clone the repository
git clone https://github.com/Mahmoud-Emad/hitman-cli.git
cd hitman-cli

# Build and test
cargo build
cargo test

# Run locally
cargo run -- example.hit --dry-run
```

## 📄 License

This project is licensed under the MIT OR Apache-2.0 license.

## 🔗 Links

- **[GitHub Repository](https://github.com/Mahmoud-Emad/hitman-cli)**
- **[VS Code Extension](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)**
- **[Issue Tracker](https://github.com/Mahmoud-Emad/hitman-cli/issues)**
- **[Releases](https://github.com/Mahmoud-Emad/hitman-cli/releases)**

---

**Made with ❤️ for developers who love simple, powerful HTTP testing tools.**
