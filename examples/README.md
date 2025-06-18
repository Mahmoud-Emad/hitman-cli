# Hitman CLI Examples 🎯

This directory contains comprehensive examples demonstrating various features and use cases of Hitman CLI.

## 📁 Example Files

### Basic Examples
- **[`simple.hit`](simple.hit)** - Basic HTTP requests and fundamental syntax
- **[`comprehensive.hit`](comprehensive.hit)** - Advanced features and comprehensive syntax showcase

### Real-World Scenarios
- **[`real-world-api-testing.hit`](real-world-api-testing.hit)** - Complete real-world API testing with multiple public APIs
- **[`fast-performance-test.hit`](fast-performance-test.hit)** - Optimized example for fast execution and performance testing

### Specialized Use Cases
- **[`authentication-patterns.hit`](authentication-patterns.hit)** - Various authentication methods (Basic Auth, Bearer tokens, API keys)
- **[`rest-api-crud.hit`](rest-api-crud.hit)** - Complete CRUD operations (Create, Read, Update, Delete)
- **[`error-handling.hit`](error-handling.hit)** - HTTP status codes and error scenario testing
- **[`variables-and-environments.hit`](variables-and-environments.hit)** - Advanced variable usage and environment configurations

### Assertion Examples
- **[`comprehensive_assertions.hit`](comprehensive_assertions.hit)** - Response validation and assertion patterns
- **[`new_with_assert.hit`](new_with_assert.hit)** - Assertion syntax reference

## 🚀 Quick Start

### Run Any Example
```bash
# Basic execution
hitman examples/simple.hit

# Dry run (validate without executing)
hitman examples/real-world-api-testing.hit --dry-run

# Parallel execution for faster testing
hitman examples/fast-performance-test.hit --parallel

# Verbose output for debugging
hitman examples/authentication-patterns.hit --verbose

# Generate JSON report
hitman examples/rest-api-crud.hit --report results.json
```

### Test Different Scenarios
```bash
# Test authentication patterns
hitman examples/authentication-patterns.hit

# Test error handling
hitman examples/error-handling.hit

# Test CRUD operations
hitman examples/rest-api-crud.hit

# Test with custom variables
hitman examples/variables-and-environments.hit --define env="staging"
```

## 📊 Example Categories

### 🌐 **API Testing**
- **Real-world scenarios** with JSONPlaceholder, HTTPBin, and ReqRes APIs
- **Multiple HTTP methods** (GET, POST, PUT, DELETE, PATCH)
- **Query parameters and headers** management
- **JSON payload** handling

### 🔐 **Authentication**
- **Basic Authentication** with encoded credentials
- **Bearer Token** authentication
- **API Key** authentication (header and query parameter)
- **Custom authentication** headers and patterns

### 🔧 **Advanced Features**
- **Variable substitution** with complex JSON objects
- **Environment-specific** configurations
- **Dynamic URL construction** with variables
- **Response validation** with assertions

### ⚡ **Performance**
- **Parallel execution** examples
- **Optimized request patterns** for speed
- **Timeout and delay** handling
- **Bulk operations** simulation

### 🛠️ **Error Handling**
- **HTTP status codes** (2xx, 3xx, 4xx, 5xx)
- **Network error** scenarios
- **Timeout handling** examples
- **Assertion failures** and validation

## 💡 Tips for Using Examples

1. **Start with `simple.hit`** to understand basic syntax
2. **Use `--dry-run`** to validate examples without making actual requests
3. **Try `--parallel`** flag for faster execution of multiple requests
4. **Customize variables** using `--define key=value` for different environments
5. **Generate reports** with `--report filename.json` for CI/CD integration

## 🔗 Related Documentation

- **[Main README](../README.md)** - Project overview and installation
- **[Usage Guide](../docs/usage.md)** - Detailed usage instructions
- **[File Format](../docs/file-format.md)** - Complete syntax reference
- **[CLI Reference](../docs/cli-reference.md)** - All command-line options

## 🤝 Contributing Examples

Have a useful example? We'd love to include it! Please:

1. Follow the existing naming convention
2. Add comprehensive comments explaining the example
3. Include usage instructions in the file header
4. Test the example with `--dry-run` before submitting
5. Update this README with your new example

---

**Happy testing with Hitman CLI! 🎯**
