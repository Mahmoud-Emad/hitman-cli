# Hitman Documentation

Welcome to the Hitman documentation! This directory contains comprehensive guides and references for using Hitman effectively.

## 📚 Documentation Index

### Getting Started
- **[Installation Guide](installation.md)** - How to install Hitman on your system
- **[Usage Guide](usage.md)** - Practical examples and common use cases
- **[Quick Start](../README.md#quick-start)** - Get up and running in minutes

### Reference
- **[File Format Specification](file-format.md)** - Complete `.hit` file syntax reference
- **[CLI Reference](cli-reference.md)** - All command-line options and flags
- **[Cross-Platform Setup](../CROSS_PLATFORM_SETUP.md)** - Build and deployment information

### Development
- **[Contributing Guide](contributing.md)** - How to contribute to the project
- **[Architecture Overview](contributing.md#architecture)** - Understanding the codebase

## 🚀 Quick Navigation

### For Users
If you're new to Hitman, start here:
1. [Install Hitman](installation.md)
2. [Learn the basics](usage.md#quick-start)
3. [Explore examples](usage.md#common-api-testing-patterns)
4. [Master the CLI](cli-reference.md)

### For Developers
If you want to contribute or understand the internals:
1. [Development setup](contributing.md#development-setup)
2. [Code architecture](contributing.md#architecture)
3. [Testing guidelines](contributing.md#testing)
4. [Contribution workflow](contributing.md#pull-request-process)

## 📖 Key Concepts

### .hit File Format
Hitman uses a simple, readable file format for defining HTTP requests:

```hit
# Variables
DEFINE baseUrl="https://api.example.com"
DEFINE token="your-token"

# HTTP Request
GET {{baseUrl}}/users
    WITH HEADER {
        "Authorization": "Bearer {{token}}"
    }
```

### Variable System
- Define variables with `DEFINE name=value`
- Use variables with `{{variableName}}` syntax
- Support for strings, numbers, booleans, and JSON
- Command-line overrides with `--define`
- Environment file support with `--env`

### Execution Modes
- **Sequential**: Default mode, requests run one after another
- **Parallel**: Use `--parallel` for concurrent execution
- **Dry Run**: Use `--dry-run` to validate without sending requests

## 🎯 Common Use Cases

### API Testing
```bash
# Test API endpoints
hitman api-tests.hit

# Test with different environments
hitman api-tests.hit --env staging.env
hitman api-tests.hit --env production.env
```

### CI/CD Integration
```bash
# Quiet mode for scripts
hitman api-tests.hit --quiet --strict --report results.json

# Check exit code
if [ $? -eq 0 ]; then
    echo "All tests passed"
else
    echo "Tests failed"
    exit 1
fi
```

### Load Testing
```bash
# Parallel execution for performance testing
hitman load-test.hit --parallel

# Add delays to control request rate
hitman api-tests.hit --delay 1000
```

## 🔧 Advanced Features

### Variable Interpolation
Variables can be used in URLs, headers, data, and query parameters:

```hit
DEFINE userId=123
DEFINE authToken="abc123"

GET https://api.example.com/users/{{userId}}
    WITH HEADER {"Authorization": "Bearer {{authToken}}"}
    WITH QUERY {"include": "profile", "format": "json"}
```

### JSON Data Support
Full JSON support with variable substitution:

```hit
DEFINE user={"name": "John", "age": 30}

POST https://api.example.com/users
    WITH DATA {{user}}
```

### Environment Management
Organize configurations with environment files:

```env
# staging.env
BASE_URL=https://staging-api.example.com
API_TOKEN=staging-token-123
DEBUG=true
```

```bash
hitman api-tests.hit --env staging.env
```

## 🛠️ Tools and Extensions

### VS Code Extension
The official VS Code extension provides:
- Syntax highlighting for `.hit` files
- IntelliSense and autocomplete
- Real-time error detection
- Code formatting and snippets

**[Install from Marketplace →](https://marketplace.visualstudio.com/items?itemName=hitman-dev.hitman-http-scripting)**

### Command Line Tools
```bash
# Validate syntax
hitman --dry-run requests.hit

# Generate reports
hitman --report results.json requests.hit

# Debug with verbose output
hitman --verbose --log debug requests.hit
```

## 📊 Best Practices

### File Organization
```hit
# 1. Variables at the top
DEFINE baseUrl="https://api.example.com"
DEFINE token="your-token"

# 2. Group related requests
# === Authentication ===
POST {{baseUrl}}/auth/login

# === User Management ===
GET {{baseUrl}}/users
POST {{baseUrl}}/users
```

### Error Handling
```bash
# Use strict mode to stop on first error
hitman api-tests.hit --strict

# Set appropriate timeouts
hitman api-tests.hit --timeout 30

# Generate reports for analysis
hitman api-tests.hit --report results.json
```

### Testing Strategy
```bash
# Validate before execution
hitman api-tests.hit --dry-run

# Test different scenarios
hitman success-cases.hit
hitman error-cases.hit
hitman edge-cases.hit
```

## 🆘 Troubleshooting

### Common Issues

**Variable not found**
```bash
# Check variable definitions with dry run
hitman requests.hit --dry-run --verbose
```

**Request timeout**
```bash
# Increase timeout
hitman requests.hit --timeout 60
```

**JSON parsing errors**
```bash
# Validate JSON syntax
hitman requests.hit --dry-run
```

### Getting Help

- **[GitHub Issues](https://github.com/Mahmoud-Emad/hitman-cli/issues)** - Bug reports and feature requests
- **[GitHub Discussions](https://github.com/Mahmoud-Emad/hitman-cli/discussions)** - Questions and community support
- **[Documentation](.)** - Comprehensive guides and references

## 🔗 External Resources

- **[HTTP Status Codes](https://httpstatuses.com/)** - Reference for HTTP status codes
- **[JSON Validator](https://jsonlint.com/)** - Validate JSON syntax
- **[REST API Tutorial](https://restfulapi.net/)** - Learn REST API concepts
- **[HTTP Methods](https://developer.mozilla.org/en-US/docs/Web/HTTP/Methods)** - HTTP method reference

---

**Need help?** Check our [GitHub Issues](https://github.com/Mahmoud-Emad/hitman-cli/issues) or start a [Discussion](https://github.com/Mahmoud-Emad/hitman-cli/discussions)!
