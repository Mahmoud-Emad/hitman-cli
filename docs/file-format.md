# .hit File Format Specification

The `.hit` file format is designed to be simple, readable, and powerful for HTTP testing. This document provides the complete specification.

## Basic Structure

A `.hit` file consists of:
1. **Variable definitions** (optional)
2. **HTTP request blocks**
3. **Assertions** (optional)
4. **Comments** (optional)

```hit
# Variable definitions
DEFINE baseUrl="https://api.example.com"
DEFINE token="your-api-token"

# HTTP request block
GET {{baseUrl}}/users
    WITH HEADER {
        "Authorization": "Bearer {{token}}"
    }
```

## Variable Definitions

Variables are defined using the `DEFINE` keyword and can be used throughout the file.

### Syntax
```hit
DEFINE variableName=value
```

### Variable Types

#### Strings
```hit
DEFINE name="John Doe"
DEFINE url="https://api.example.com"
DEFINE empty=""
```

#### Numbers
```hit
DEFINE count=42
DEFINE price=19.99
DEFINE negative=-10
```

#### Booleans
```hit
DEFINE enabled=true
DEFINE disabled=false
```

#### JSON Objects
```hit
DEFINE headers={
    "Content-Type": "application/json",
    "Authorization": "Bearer token123"
}

DEFINE user={
    "name": "John",
    "age": 30,
    "active": true
}
```

#### JSON Arrays
```hit
DEFINE tags=["api", "testing", "http"]
DEFINE numbers=[1, 2, 3, 4, 5]
```

### Variable Names

Variable names must follow identifier rules:
- Start with a letter (a-z, A-Z) or underscore (_)
- Contain only letters, numbers, and underscores
- Case-sensitive

**Valid**: `apiKey`, `base_url`, `_private`, `API_TOKEN`
**Invalid**: `123invalid`, `api-key`, `my variable`

### Variable Substitution

Use `{{variableName}}` syntax to substitute variables:

```hit
DEFINE baseUrl="https://api.example.com"
DEFINE userId=123

GET {{baseUrl}}/users/{{userId}}
```

## HTTP Request Blocks

Each HTTP request block starts with an HTTP method and URL, followed by optional clauses.

### HTTP Methods

Supported methods:
- `GET`
- `POST`
- `PUT`
- `PATCH`
- `DELETE`
- `HEAD`
- `OPTIONS`

### Basic Request
```hit
GET https://api.example.com/users
```

### Request with Headers
```hit
POST https://api.example.com/users
    WITH HEADER {
        "Content-Type": "application/json",
        "Authorization": "Bearer token123"
    }
```

### Request with Data (Body)
```hit
POST https://api.example.com/users
    WITH DATA {
        "name": "John Doe",
        "email": "john@example.com"
    }
```

### Request with Query Parameters
```hit
GET https://api.example.com/users
    WITH QUERY {
        "page": 1,
        "limit": 10,
        "sort": "name"
    }
```

### Complete Request
```hit
POST https://api.example.com/users
    WITH HEADER {
        "Content-Type": "application/json",
        "Authorization": "Bearer {{token}}"
    }
    WITH DATA {
        "name": "{{userName}}",
        "email": "{{userEmail}}"
    }
    WITH QUERY {
        "source": "api",
        "version": "v1"
    }
```

## Comments

Three comment styles are supported:

### Line Comments
```hit
# This is a hash comment
// This is a double-slash comment
; This is a semicolon comment

GET https://api.example.com/users  # Inline comment
```

### Block Comments
```hit
/*
This is a block comment
that spans multiple lines
*/

GET https://api.example.com/users
```

### Inline Comments in Variables
```hit
DEFINE count=42 # This is the user count
DEFINE enabled=true // Feature flag
DEFINE url="https://api.example.com"; # Base API URL
```

## Advanced Features

### Variable Interpolation in JSON
```hit
DEFINE token="abc123"
DEFINE userId=42

POST https://api.example.com/users
    WITH HEADER {
        "Authorization": "Bearer {{token}}"
    }
    WITH DATA {
        "userId": {{userId}},
        "message": "Hello {{token}}"
    }
```

### Complex JSON Structures
```hit
DEFINE config={
    "database": {
        "host": "localhost",
        "port": 5432,
        "ssl": true
    },
    "features": ["auth", "logging", "metrics"]
}

POST https://api.example.com/configure
    WITH DATA {{config}}
```

### URL Construction
```hit
DEFINE protocol="https"
DEFINE host="api.example.com"
DEFINE version="v1"
DEFINE endpoint="users"

GET {{protocol}}://{{host}}/{{version}}/{{endpoint}}
```

## Request Aliasing and Assertions

### Request Aliasing

Use the `AS` keyword to assign an alias to a request for later reference in assertions:

```hit
GET https://api.example.com/users/1
    AS getUserRequest

POST https://api.example.com/users
    WITH HEADER {"Content-Type": "application/json"}
    WITH DATA {"name": "John", "email": "john@example.com"}
    AS createUserRequest
```

### Assertions

Assertions validate HTTP responses using the `ASSERT` keyword. They reference aliased requests and support various property access patterns.

#### Basic Syntax
```hit
ASSERT alias.property operator expected_value
```

#### Supported Operators
- `==` - Equals
- `!=` - Not equals
- `CONTAINS` - String contains (for body and string values)

#### Property Access Patterns

##### Status Code
```hit
GET https://api.example.com/users
    AS request1

ASSERT request1.status == 200
ASSERT request1.status != 404
```

##### Headers
```hit
GET https://api.example.com/users
    AS request1

ASSERT request1.headers.Content-Type == "application/json"
ASSERT request1.headers.Content-Type CONTAINS "json"
```

##### JSON Response Properties
```hit
GET https://api.example.com/users/1
    AS request1

ASSERT request1.json.id == 1
ASSERT request1.json.name == "John Doe"
ASSERT request1.json.profile.age == 30
ASSERT request1.json.tags.0 == "admin"
```

##### Response Body
```hit
GET https://api.example.com/status
    AS request1

ASSERT request1.body CONTAINS "success"
ASSERT request1.body == "OK"
```

#### Complete Example with Assertions
```hit
DEFINE baseUrl="https://api.example.com"
DEFINE userId=1

# Get user information
GET {{baseUrl}}/users/{{userId}}
    AS getUser

# Validate the response
ASSERT getUser.status == 200
ASSERT getUser.headers.Content-Type CONTAINS "application/json"
ASSERT getUser.json.id == {{userId}}
ASSERT getUser.json.name != null
ASSERT getUser.json.email CONTAINS "@"

# Create a new user
POST {{baseUrl}}/users
    WITH HEADER {"Content-Type": "application/json"}
    WITH DATA {
        "name": "Jane Doe",
        "email": "jane@example.com"
    }
    AS createUser

# Validate creation
ASSERT createUser.status == 201
ASSERT createUser.json.name == "Jane Doe"
ASSERT createUser.json.id != null
```

## Best Practices

### Organization
```hit
# 1. Define all variables at the top
DEFINE baseUrl="https://api.example.com"
DEFINE token="your-token"

# 2. Group related requests
# Authentication
POST {{baseUrl}}/auth/login
    WITH DATA {"username": "admin", "password": "secret"}

# User management
GET {{baseUrl}}/users
POST {{baseUrl}}/users
    WITH DATA {"name": "John"}
```

### Naming Conventions
```hit
# Use camelCase for variables
DEFINE apiKey="key123"
DEFINE baseUrl="https://api.example.com"

# Use descriptive names
DEFINE userCreatePayload={"name": "John", "email": "john@example.com"}
DEFINE authHeaders={"Authorization": "Bearer token"}
```

### Error Handling
```hit
# Test both success and error cases
GET {{baseUrl}}/users/123        # Valid user
GET {{baseUrl}}/users/999999     # Non-existent user
```

## Validation Rules

1. **Variable names** must be valid identifiers
2. **JSON syntax** must be valid in DATA, HEADER, and QUERY clauses
3. **URLs** must be valid HTTP/HTTPS URLs
4. **Variable references** must be defined before use
5. **HTTP methods** must be supported
6. **Clause order** doesn't matter within a request block
7. **Alias names** must contain only alphanumeric characters and underscores
8. **Assertion syntax** must use valid operators (==, !=, CONTAINS)
9. **Assertion references** must point to existing request aliases
10. **Property paths** in assertions must be valid (status, headers.name, json.path, body)
