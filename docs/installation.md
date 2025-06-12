# Installation Guide

This guide covers all the ways to install Hitman on your system.

## Pre-built Binaries (Recommended)

Download the latest release for your platform from the [GitHub Releases](https://github.com/Mahmoud-Emad/hitman-cli/releases) page:

- **Linux (x86_64)**: `hitman-linux-x86_64.tar.gz`
- **Linux (x86_64, musl)**: `hitman-linux-x86_64-musl.tar.gz`
- **Linux (ARM64)**: `hitman-linux-aarch64.tar.gz`
- **macOS (Intel)**: `hitman-macos-x86_64.tar.gz`
- **macOS (Apple Silicon)**: `hitman-macos-aarch64.tar.gz`
- **Windows (x86_64)**: `hitman-windows-x86_64.zip`

### Linux/macOS Installation

```bash
# Example for Linux x86_64
curl -L https://github.com/Mahmoud-Emad/hitman-cli/releases/latest/download/hitman-linux-x86_64.tar.gz | tar xz
sudo mv hitman /usr/local/bin/

# Verify installation
hitman --version
```

### Windows Installation

1. Download `hitman-windows-x86_64.zip`
2. Extract to a folder (e.g., `C:\tools\hitman\`)
3. Add the folder to your PATH environment variable
4. Open a new command prompt and run `hitman --version`

## Package Managers

### Cargo (Rust Package Manager)

```bash
cargo install hitman
```

### Homebrew (macOS/Linux)

```bash
# Coming soon
brew install hitman
```

### Chocolatey (Windows)

```bash
# Coming soon
choco install hitman
```

## Build from Source

### Prerequisites

- Rust 1.70+ (install from [rustup.rs](https://rustup.rs/))
- Git

### Steps

```bash
# Clone the repository
git clone https://github.com/Mahmoud-Emad/hitman-cli.git
cd hitman-cli

# Build the project
cargo build --release

# Install globally (optional)
cargo install --path .

# Verify installation
./target/release/hitman --version
```

## VS Code Extension

Enhance your `.hit` file editing experience with the official VS Code extension:

**[Install from VS Code Marketplace](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)**

### Features
- 🎨 **Syntax Highlighting**: Beautiful color coding for HTTP methods, URLs, headers, and variables
- 📝 **IntelliSense**: Smart autocomplete for HTTP methods, headers, and variable names
- ❌ **Error Detection**: Real-time validation with error squiggles and helpful messages
- 🔧 **Code Formatting**: Automatic formatting and indentation
- 📋 **Snippets**: Quick templates for common request patterns

### Installation Methods
1. **From VS Code**: Search for "Hitman HTTP Scripting" in Extensions (`Ctrl+Shift+X`)
2. **From Marketplace**: Visit the [extension page](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)
3. **Command Line**: `code --install-extension hitman-dev.hitman-http-scripting`

## Verification

After installation, verify that Hitman is working correctly:

```bash
# Check version
hitman --version

# Test with a simple request
echo 'GET https://httpbin.org/get' | hitman --stdin --dry-run

# Should output: request validation without actually sending it
```

## Troubleshooting

### Common Issues

**Command not found**
- Ensure the binary is in your PATH
- On Linux/macOS, check that `/usr/local/bin` is in your PATH
- On Windows, verify the installation directory is added to PATH

**Permission denied (Linux/macOS)**
```bash
chmod +x hitman
```

**SSL/TLS errors**
- Use the musl build on older Linux systems
- Ensure your system has up-to-date CA certificates

### Getting Help

- Check the [CLI Reference](cli-reference.md) for command options
- See [Usage Guide](usage.md) for examples
- Report issues on [GitHub](https://github.com/Mahmoud-Emad/hitman-cli/issues)
