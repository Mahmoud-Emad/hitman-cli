# Cross-Platform Build Setup

This document describes the cross-platform build setup for Hitman, enabling automatic builds for macOS, Linux, and Windows.

> **💡 Tip**: For the best development experience, install the [VS Code extension](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting) which provides syntax highlighting, error detection, and IntelliSense for `.hit` files.

## GitHub Workflows

### 1. CI Workflow (`.github/workflows/ci.yml`)
- **Triggers**: Push to main/develop, Pull Requests
- **Platforms**: Ubuntu, macOS, Windows
- **Rust Versions**: Stable, Beta (Ubuntu only)
- **Features**:
  - Runs all tests on multiple platforms
  - Code formatting checks (`cargo fmt`)
  - Linting with Clippy (`cargo clippy`)
  - Security audit (`cargo audit`)
  - Code coverage with Codecov
  - Cross-compilation build checks
  - Integration tests with CLI functionality

### 2. Release Workflow (`.github/workflows/release.yml`)
- **Triggers**: Git tags starting with `v*`, Manual dispatch
- **Platforms**: Ubuntu, macOS, Windows
- **Target Architectures**:
  - Linux: x86_64 (glibc), x86_64 (musl), aarch64
  - macOS: x86_64 (Intel), aarch64 (Apple Silicon)
  - Windows: x86_64, aarch64
- **Features**:
  - Automated binary builds for all platforms
  - Proper packaging (tar.gz for Unix, zip for Windows)
  - SHA256 checksums generation
  - GitHub Release creation with artifacts
  - Changelog integration

### 3. Publish Workflow (`.github/workflows/publish.yml`)
- **Triggers**: GitHub Release published, Manual dispatch
- **Features**:
  - Automatic publishing to crates.io
  - Dry-run capability for testing
  - Package verification before publishing

## Build Targets

| Platform | Architecture | Target Triple | Package Format | Cross-Compile |
|----------|-------------|---------------|----------------|---------------|
| Linux | x86_64 | `x86_64-unknown-linux-gnu` | tar.gz | No |
| Linux (musl) | x86_64 | `x86_64-unknown-linux-musl` | tar.gz | Yes |
| Linux | ARM64 | `aarch64-unknown-linux-gnu` | tar.gz | Yes |
| macOS | Intel | `x86_64-apple-darwin` | tar.gz | No |
| macOS | Apple Silicon | `aarch64-apple-darwin` | tar.gz | No |
| Windows | x86_64 | `x86_64-pc-windows-msvc` | zip | No |

## Local Build Scripts

### Unix/Linux/macOS (`scripts/build-release.sh`)
```bash
# Build all platforms (where possible)
./scripts/build-release.sh v0.1.0

# Build specific version
./scripts/build-release.sh v1.2.3
```

### Windows (`scripts/build-release.bat`)
```cmd
REM Build all platforms (where possible)
scripts\build-release.bat v0.1.0

REM Build specific version
scripts\build-release.bat v1.2.3
```

## Cross-Compilation Setup

### Dependencies
- **cross**: For Linux cross-compilation from other platforms
- **Docker**: Required by cross for containerized builds
- **Platform-specific toolchains**: Automatically handled by GitHub Actions

### Configuration (`Cross.toml`)
- Specifies Docker images for cross-compilation
- Uses latest cross-rs images for better compatibility
- Configured for all supported target platforms

## Cargo Configuration

### Release Profile Optimizations
```toml
[profile.release]
lto = true              # Link-time optimization
codegen-units = 1       # Better optimization
panic = "abort"         # Smaller binaries
strip = true           # Remove debug symbols
```

### Dependencies
- **reqwest**: Configured with `rustls-tls` for better cross-platform support
- **tokio**: Full feature set for async functionality
- All dependencies chosen for cross-platform compatibility

## Release Process

### Automatic Release (Recommended)
1. **Create and push a tag**:
   ```bash
   git tag v0.1.0
   git push origin v0.1.0
   ```

2. **GitHub Actions will**:
   - Run all tests across platforms
   - Build binaries for all targets
   - Create GitHub Release with artifacts
   - Generate checksums
   - Publish to crates.io (if configured)

### Manual Release
1. **Run local build script**:
   ```bash
   ./scripts/build-release.sh v0.1.0
   ```

2. **Upload artifacts manually** to GitHub Releases

## Installation Methods

### Pre-built Binaries
Users can download platform-specific binaries from GitHub Releases:
- Automatic architecture detection
- No compilation required
- Immediate usage

### Package Managers (Future)
- **Homebrew** (macOS/Linux)
- **Chocolatey** (Windows)
- **Scoop** (Windows)
- **APT/YUM** packages (Linux)

### Cargo
```bash
cargo install hitman
```

## Security

### Checksums
- SHA256 checksums generated for all artifacts
- Verification instructions in release notes
- Automated checksum validation in CI

### Supply Chain
- Dependabot for dependency updates
- Security audits in CI pipeline
- Minimal dependency footprint
- Reproducible builds

## Troubleshooting

### Common Issues
1. **Cross-compilation failures**: Usually Docker/cross setup issues
2. **Missing targets**: Run `rustup target add <target>`
3. **Permission errors**: Ensure Docker has proper permissions
4. **Network timeouts**: Retry or check network connectivity

### Platform-Specific Notes
- **macOS**: Apple Silicon builds require macOS runner
- **Windows**: ARM64 support is experimental
- **Linux**: musl builds provide better portability

## Monitoring

### CI Status
- All workflows report status to GitHub
- Failed builds block releases
- Coverage reports track test quality

### Release Metrics
- Download statistics from GitHub Releases
- Platform usage analytics
- Performance benchmarks across platforms
