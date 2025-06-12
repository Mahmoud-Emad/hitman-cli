# Contributing to Hitman

We welcome contributions to Hitman! This guide will help you get started with contributing to the project.

## 🚀 Getting Started

### Prerequisites

- **Rust 1.70+**: Install from [rustup.rs](https://rustup.rs/)
- **Git**: For version control
- **VS Code** (optional): For the best development experience

### Development Setup

```bash
# 1. Fork the repository on GitHub
# 2. Clone your fork
git clone https://github.com/YOUR_USERNAME/hitman-cli.git
cd hitman-cli

# 3. Add upstream remote
git remote add upstream https://github.com/Mahmoud-Emad/hitman-cli.git

# 4. Install dependencies and build
cargo build

# 5. Run tests
cargo test

# 6. Run clippy for linting
cargo clippy --all-features --all-targets -- -D warnings

# 7. Format code
cargo fmt
```

## 🔧 Development Workflow

### Making Changes

1. **Create a branch** for your feature/fix:
   ```bash
   git checkout -b feature/your-feature-name
   ```

2. **Make your changes** following our coding standards

3. **Test your changes**:
   ```bash
   # Run all tests
   cargo test
   
   # Test specific functionality
   cargo test test_name
   
   # Run integration tests
   cargo test --test integration_tests
   ```

4. **Lint your code**:
   ```bash
   cargo clippy --all-features --all-targets -- -D warnings
   cargo fmt
   ```

5. **Commit your changes**:
   ```bash
   git add .
   git commit -m "feat: add new feature description"
   ```

6. **Push and create PR**:
   ```bash
   git push origin feature/your-feature-name
   ```

### Commit Message Format

We follow conventional commits:

- `feat:` - New features
- `fix:` - Bug fixes
- `docs:` - Documentation changes
- `style:` - Code style changes (formatting, etc.)
- `refactor:` - Code refactoring
- `test:` - Adding or updating tests
- `chore:` - Maintenance tasks

Examples:
```
feat: add support for PATCH HTTP method
fix: resolve variable substitution in nested JSON
docs: update installation guide
test: add tests for error handling
```

## 🧪 Testing

### Running Tests

```bash
# All tests
cargo test

# Unit tests only
cargo test --lib

# Integration tests only
cargo test --test integration_tests

# CLI tests only
cargo test --test cli_tests

# Specific test
cargo test test_variable_substitution

# With output
cargo test -- --nocapture
```

### Writing Tests

#### Unit Tests
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_your_function() {
        // Arrange
        let input = "test input";
        
        // Act
        let result = your_function(input);
        
        // Assert
        assert_eq!(result, expected_output);
    }
}
```

#### Integration Tests
```rust
// tests/integration_tests.rs
use hitman::parser::parse_file;

#[test]
fn test_complete_workflow() {
    let content = r#"
        DEFINE baseUrl="https://api.example.com"
        GET {{baseUrl}}/users
    "#;
    
    let result = parse_file(content);
    assert!(result.is_ok());
}
```

### Test Coverage

We aim for high test coverage. Check coverage with:
```bash
# Install cargo-tarpaulin
cargo install cargo-tarpaulin

# Generate coverage report
cargo tarpaulin --out Html
```

## 📝 Code Style

### Rust Guidelines

- Follow [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- Use `cargo fmt` for formatting
- Use `cargo clippy` for linting
- Write documentation for public APIs
- Use meaningful variable and function names

### Code Organization

```
src/
├── main.rs          # CLI entry point
├── lib.rs           # Library root
├── command.rs       # HTTP command structures
├── parser.rs        # .hit file parsing
├── variables.rs     # Variable handling
├── executor.rs      # Request execution
└── error.rs         # Error types
```

### Documentation

- Document all public functions and structs
- Include examples in documentation
- Update README.md for user-facing changes
- Add entries to CHANGELOG.md

```rust
/// Parse a .hit file and return HTTP commands.
/// 
/// # Arguments
/// 
/// * `content` - The content of the .hit file
/// 
/// # Examples
/// 
/// ```
/// use hitman::parser::parse_file;
/// 
/// let content = "GET https://api.example.com/users";
/// let commands = parse_file(content)?;
/// ```
pub fn parse_file(content: &str) -> Result<Vec<HitCommand>> {
    // Implementation
}
```

## 🐛 Bug Reports

### Before Reporting

1. Check existing issues
2. Test with the latest version
3. Provide minimal reproduction case

### Bug Report Template

```markdown
**Describe the bug**
A clear description of what the bug is.

**To Reproduce**
Steps to reproduce the behavior:
1. Create file with content: '...'
2. Run command: '...'
3. See error

**Expected behavior**
What you expected to happen.

**Environment:**
- OS: [e.g. Ubuntu 20.04]
- Hitman version: [e.g. 0.1.4]
- Rust version: [e.g. 1.70.0]

**Additional context**
Any other context about the problem.
```

## 💡 Feature Requests

### Before Requesting

1. Check existing issues and discussions
2. Consider if it fits the project scope
3. Think about implementation complexity

### Feature Request Template

```markdown
**Is your feature request related to a problem?**
A clear description of what the problem is.

**Describe the solution you'd like**
A clear description of what you want to happen.

**Describe alternatives you've considered**
Other solutions you've considered.

**Additional context**
Any other context or screenshots.
```

## 🔄 Pull Request Process

### Before Submitting

- [ ] Tests pass (`cargo test`)
- [ ] Linting passes (`cargo clippy`)
- [ ] Code is formatted (`cargo fmt`)
- [ ] Documentation is updated
- [ ] CHANGELOG.md is updated (if needed)

### PR Template

```markdown
**Description**
Brief description of changes.

**Type of change**
- [ ] Bug fix
- [ ] New feature
- [ ] Breaking change
- [ ] Documentation update

**Testing**
- [ ] Tests pass
- [ ] New tests added (if applicable)

**Checklist**
- [ ] Code follows style guidelines
- [ ] Self-review completed
- [ ] Documentation updated
- [ ] No breaking changes (or documented)
```

## 🏗️ Architecture

### Core Components

1. **Parser** (`parser.rs`): Parses .hit files into commands
2. **Variables** (`variables.rs`): Handles variable definitions and substitution
3. **Command** (`command.rs`): Represents HTTP commands
4. **Executor** (`executor.rs`): Executes HTTP requests
5. **CLI** (`main.rs`): Command-line interface

### Design Principles

- **Simplicity**: Keep the API simple and intuitive
- **Performance**: Optimize for speed and memory usage
- **Reliability**: Handle errors gracefully
- **Extensibility**: Design for future enhancements

## 📚 Resources

- [Rust Book](https://doc.rust-lang.org/book/)
- [Rust API Guidelines](https://rust-lang.github.io/api-guidelines/)
- [Cargo Book](https://doc.rust-lang.org/cargo/)
- [Project Issues](https://github.com/Mahmoud-Emad/hitman-cli/issues)
- [Project Discussions](https://github.com/Mahmoud-Emad/hitman-cli/discussions)

## 🤝 Community

- Be respectful and inclusive
- Help others learn and grow
- Share knowledge and experiences
- Follow our [Code of Conduct](CODE_OF_CONDUCT.md)

## 📞 Getting Help

- **Issues**: For bugs and feature requests
- **Discussions**: For questions and general discussion
- **Discord**: [Join our community](https://discord.gg/hitman-cli) (coming soon)

Thank you for contributing to Hitman! 🎯
