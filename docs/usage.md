# Usage Guide

This guide provides practical examples and common use cases for Hitman.

## Quick Start

### Your First .hit File

Create a file called `hello.hit`:

```hit
# Simple GET request
GET https://httpbin.org/get

# POST request with data
POST https://httpbin.org/post
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "message": "Hello, World!"
    }
```

Run it:
```bash
hitman hello.hit
```

## Working with Variables

### Basic Variables

```hit
# Define variables
DEFINE baseUrl="https://jsonplaceholder.typicode.com"
DEFINE userId=1

# Use variables in requests
GET {{baseUrl}}/users/{{userId}}
GET {{baseUrl}}/posts?userId={{userId}}
```

### Complex Variables

```hit
# JSON object variables
DEFINE headers={
    "Content-Type": "application/json",
    "Authorization": "Bearer your-token-here"
}

DEFINE userData={
    "name": "John Doe",
    "email": "john@example.com",
    "age": 30
}

# Use in requests
POST https://api.example.com/users
    WITH HEADER {{headers}}
    WITH DATA {{userData}}
```

### Environment-Specific Variables

Create environment files:

**staging.env:**
```env
BASE_URL=https://staging-api.example.com
API_TOKEN=staging-token-123
DEBUG=true
```

**production.env:**
```env
BASE_URL=https://api.example.com
API_TOKEN=prod-token-456
DEBUG=false
```

**api-tests.hit:**
```hit
# Variables will be loaded from environment file
GET {{BASE_URL}}/health
    WITH HEADER {
        "Authorization": "Bearer {{API_TOKEN}}"
    }
```

Run with different environments:
```bash
# Staging
hitman api-tests.hit --env staging.env

# Production
hitman api-tests.hit --env production.env
```

## Common API Testing Patterns

### Authentication Flow

```hit
# Define auth variables
DEFINE baseUrl="https://api.example.com"
DEFINE username="testuser"
DEFINE password="testpass"

# 1. Login to get token
POST {{baseUrl}}/auth/login
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "username": "{{username}}",
        "password": "{{password}}"
    }

# 2. Use token for authenticated requests
DEFINE token="your-actual-token-here"

GET {{baseUrl}}/profile
    WITH HEADER {
        "Authorization": "Bearer {{token}}"
    }

# 3. CRUD operations
POST {{baseUrl}}/users
    WITH HEADER {
        "Authorization": "Bearer {{token}}",
        "Content-Type": "application/json"
    }
    WITH DATA {
        "name": "New User",
        "email": "newuser@example.com"
    }
```

### REST API Testing

```hit
DEFINE baseUrl="https://jsonplaceholder.typicode.com"
DEFINE userId=1
DEFINE postId=1

# List all users
GET {{baseUrl}}/users

# Get specific user
GET {{baseUrl}}/users/{{userId}}

# Create new user
POST {{baseUrl}}/users
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "name": "John Doe",
        "username": "johndoe",
        "email": "john@example.com"
    }

# Update user
PUT {{baseUrl}}/users/{{userId}}
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "name": "John Smith",
        "username": "johnsmith",
        "email": "johnsmith@example.com"
    }

# Partial update
PATCH {{baseUrl}}/users/{{userId}}
    WITH HEADER {
        "Content-Type": "application/json"
    }
    WITH DATA {
        "name": "John Updated"
    }

# Delete user
DELETE {{baseUrl}}/users/{{userId}}
```

### Query Parameters

```hit
DEFINE baseUrl="https://api.example.com"

# Simple query parameters
GET {{baseUrl}}/users
    WITH QUERY {
        "page": 1,
        "limit": 10,
        "sort": "name"
    }

# Complex filtering
GET {{baseUrl}}/products
    WITH QUERY {
        "category": "electronics",
        "minPrice": 100,
        "maxPrice": 500,
        "inStock": true,
        "tags": ["smartphone", "android"]
    }

# Search with special characters
GET {{baseUrl}}/search
    WITH QUERY {
        "q": "hello world",
        "type": "exact",
        "fields": ["title", "description"]
    }
```

## Testing Strategies

### Validation and Dry Runs

```bash
# Validate syntax without sending requests
hitman api-tests.hit --dry-run

# Check what variables will be substituted
hitman api-tests.hit --dry-run --verbose
```

### Error Testing

```hit
DEFINE baseUrl="https://httpbin.org"

# Test different HTTP status codes
GET {{baseUrl}}/status/200  # Success
GET {{baseUrl}}/status/404  # Not Found
GET {{baseUrl}}/status/500  # Server Error

# Test timeouts
GET {{baseUrl}}/delay/10    # 10 second delay
```

### Performance Testing

```bash
# Sequential execution with timing
hitman load-test.hit --verbose

# Parallel execution for load testing
hitman load-test.hit --parallel

# Add delays to avoid overwhelming server
hitman api-tests.hit --delay 1000  # 1 second between requests
```

## Advanced Usage

### Command Line Overrides

```bash
# Override variables for different environments
hitman api-tests.hit --define baseUrl="https://localhost:3000"

# Multiple overrides
hitman api-tests.hit \
  --define baseUrl="https://staging.api.com" \
  --define token="staging-token" \
  --define debug=true
```

### Reporting and CI/CD

```bash
# Generate detailed report
hitman api-tests.hit --report results.json

# Quiet mode for scripts
hitman api-tests.hit --quiet --strict --report results.json

# Check exit code
if [ $? -eq 0 ]; then
    echo "All tests passed"
else
    echo "Tests failed"
    cat results.json
    exit 1
fi
```

### Pipeline Integration

**GitHub Actions:**
```yaml
- name: Run API Tests
  run: |
    hitman api-tests.hit \
      --env production.env \
      --quiet \
      --strict \
      --report test-results.json
```

**Jenkins:**
```groovy
stage('API Tests') {
    steps {
        sh 'hitman api-tests.hit --env staging.env --report results.json'
        archiveArtifacts artifacts: 'results.json'
    }
}
```

## Best Practices

### File Organization

```hit
# 1. Variables at the top
DEFINE baseUrl="https://api.example.com"
DEFINE token="your-token"

# 2. Group related requests
# === Authentication ===
POST {{baseUrl}}/auth/login
    WITH DATA {"username": "admin", "password": "secret"}

# === User Management ===
GET {{baseUrl}}/users
POST {{baseUrl}}/users
    WITH DATA {"name": "John"}

# === Cleanup ===
DELETE {{baseUrl}}/users/123
```

### Error Handling

```bash
# Use strict mode to stop on first error
hitman api-tests.hit --strict

# Set appropriate timeouts
hitman api-tests.hit --timeout 30

# Use verbose mode for debugging
hitman api-tests.hit --verbose --log debug
```

### Variable Management

```hit
# Use descriptive variable names
DEFINE userCreatePayload={"name": "John", "email": "john@example.com"}
DEFINE authHeaders={"Authorization": "Bearer token"}

# Group related variables
DEFINE apiConfig={
    "baseUrl": "https://api.example.com",
    "version": "v1",
    "timeout": 30
}
```

## Troubleshooting

### Common Issues

**Variable not found:**
```bash
# Check variable definitions
hitman api-tests.hit --dry-run --verbose
```

**Request timeout:**
```bash
# Increase timeout
hitman api-tests.hit --timeout 60
```

**SSL/TLS errors:**
```bash
# Use different build or check certificates
hitman api-tests.hit --log debug
```

### Debugging

```bash
# Maximum verbosity
hitman api-tests.hit --verbose --log debug

# Dry run to check syntax
hitman api-tests.hit --dry-run

# Test single request
echo 'GET https://httpbin.org/get' | hitman --stdin --verbose
```
