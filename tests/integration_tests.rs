//! Integration tests for the Hitman HTTP client.

use hitman::{parse_block, parse_file_into_blocks, HitError};

#[test]
fn test_parse_real_hit_file() {
    let content = r#"GET https://jsonplaceholder.typicode.com/posts

GET https://httpbin.org/get

POST https://reqres.in/api/users
    WITH HEADER {Content-Type: application/json}
    WITH DATA {"name": "Mahmoud", "job": "Rust Developer"}

POST https://reqres.in/api/login
    WITH HEADER {Content-Type: application/json}
    WITH DATA {"email": "eve.holt@reqres.in", "password": "cityslicka"}
"#;

    let blocks = parse_file_into_blocks(content);
    assert_eq!(blocks.len(), 4);

    // Test first block (simple GET)
    let cmd1 = parse_block(&blocks[0]).unwrap();
    assert_eq!(cmd1.method.to_string(), "GET");
    assert_eq!(cmd1.url, "https://jsonplaceholder.typicode.com/posts");
    assert!(cmd1.headers.is_empty());
    assert!(cmd1.body.is_none());

    // Test second block (simple GET)
    let cmd2 = parse_block(&blocks[1]).unwrap();
    assert_eq!(cmd2.method.to_string(), "GET");
    assert_eq!(cmd2.url, "https://httpbin.org/get");

    // Test third block (POST with header and data)
    let cmd3 = parse_block(&blocks[2]).unwrap();
    assert_eq!(cmd3.method.to_string(), "POST");
    assert_eq!(cmd3.url, "https://reqres.in/api/users");
    assert_eq!(
        cmd3.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(
        cmd3.body,
        Some(r#"{"name": "Mahmoud", "job": "Rust Developer"}"#.to_string())
    );

    // Test fourth block (POST with header and data)
    let cmd4 = parse_block(&blocks[3]).unwrap();
    assert_eq!(cmd4.method.to_string(), "POST");
    assert_eq!(cmd4.url, "https://reqres.in/api/login");
    assert_eq!(
        cmd4.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(
        cmd4.body,
        Some(r#"{"email": "eve.holt@reqres.in", "password": "cityslicka"}"#.to_string())
    );
}

#[test]
fn test_different_order_parsing() {
    let content = r#"POST https://reqres.in/api/users
    WITH DATA {"name": "Mahmoud", "job": "Rust Developer"}
    WITH HEADER {Content-Type: application/json}

POST https://reqres.in/api/login
    WITH HEADER {Content-Type: application/json}
    WITH DATA {"email": "eve.holt@reqres.in", "password": "cityslicka"}
"#;

    let blocks = parse_file_into_blocks(content);
    assert_eq!(blocks.len(), 2);

    // Both blocks should parse successfully regardless of order
    let cmd1 = parse_block(&blocks[0]).unwrap();
    assert_eq!(cmd1.method.to_string(), "POST");
    assert_eq!(
        cmd1.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert!(cmd1.body.is_some());

    let cmd2 = parse_block(&blocks[1]).unwrap();
    assert_eq!(cmd2.method.to_string(), "POST");
    assert_eq!(
        cmd2.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert!(cmd2.body.is_some());
}

#[test]
fn test_complex_headers() {
    let content = r#"POST https://api.example.com/users
    WITH HEADER {Content-Type: application/json, Authorization: Bearer token123}
    WITH DATA {"name": "John", "age": 30}

PUT https://api.example.com/users/1
    WITH DATA {"name": "Jane", "age": 25}
    WITH HEADER {Content-Type: application/json, X-Custom-Header: custom-value}

DELETE https://api.example.com/users/1
    WITH HEADER {Authorization: Bearer token456}
"#;

    let blocks = parse_file_into_blocks(content);
    assert_eq!(blocks.len(), 3);

    // Test POST with multiple headers
    let cmd1 = parse_block(&blocks[0]).unwrap();
    assert_eq!(cmd1.method.to_string(), "POST");
    assert_eq!(cmd1.headers.len(), 2);
    assert_eq!(
        cmd1.headers.get("Content-Type"),
        Some(&"application/json".to_string())
    );
    assert_eq!(
        cmd1.headers.get("Authorization"),
        Some(&"Bearer token123".to_string())
    );

    // Test PUT with different order
    let cmd2 = parse_block(&blocks[1]).unwrap();
    assert_eq!(cmd2.method.to_string(), "PUT");
    assert_eq!(cmd2.headers.len(), 2);
    assert_eq!(
        cmd2.headers.get("X-Custom-Header"),
        Some(&"custom-value".to_string())
    );

    // Test DELETE with only header
    let cmd3 = parse_block(&blocks[2]).unwrap();
    assert_eq!(cmd3.method.to_string(), "DELETE");
    assert_eq!(cmd3.headers.len(), 1);
    assert_eq!(
        cmd3.headers.get("Authorization"),
        Some(&"Bearer token456".to_string())
    );
    assert!(cmd3.body.is_none());
}

#[test]
fn test_error_cases() {
    // Test invalid HTTP method
    let invalid_method = vec!["INVALID https://api.example.com/users".to_string()];
    let result = parse_block(&invalid_method);
    assert!(result.is_err());
    match result.unwrap_err() {
        HitError::InvalidMethod { method } => assert_eq!(method, "INVALID"),
        _ => panic!("Expected InvalidMethod error"),
    }

    // Test invalid URL
    let invalid_url = vec!["GET invalid-url".to_string()];
    let result = parse_block(&invalid_url);
    assert!(result.is_err());
    match result.unwrap_err() {
        HitError::UrlParseError { .. } => {} // URL parsing error from url crate
        HitError::InvalidUrl { .. } => {}    // Invalid scheme error
        _ => panic!("Expected URL-related error"),
    }

    // Test malformed JSON
    let malformed_json = vec![
        "POST https://api.example.com/users".to_string(),
        "    WITH DATA {invalid json}".to_string(),
    ];
    let result = parse_block(&malformed_json);
    assert!(result.is_err());
    match result.unwrap_err() {
        HitError::MalformedJson { line, .. } => assert_eq!(line, 2),
        _ => panic!("Expected MalformedJson error"),
    }

    // Test empty block
    let empty_block: Vec<String> = vec![];
    let result = parse_block(&empty_block);
    assert!(result.is_err());
    match result.unwrap_err() {
        HitError::EmptyBlock => {}
        _ => panic!("Expected EmptyBlock error"),
    }
}

#[test]
fn test_whitespace_handling() {
    let content = r#"
GET https://api.example.com/users


POST https://api.example.com/users
    WITH HEADER {Content-Type: application/json}
    WITH DATA {"name": "Test"}


"#;

    let blocks = parse_file_into_blocks(content);
    // Should ignore leading/trailing empty lines and multiple empty lines between blocks
    assert_eq!(blocks.len(), 2);

    let cmd1 = parse_block(&blocks[0]).unwrap();
    assert_eq!(cmd1.method.to_string(), "GET");

    let cmd2 = parse_block(&blocks[1]).unwrap();
    assert_eq!(cmd2.method.to_string(), "POST");
}

#[test]
fn test_command_display() {
    let lines = vec![
        "POST https://api.example.com/users".to_string(),
        "    WITH HEADER {Content-Type: application/json, Authorization: Bearer token}".to_string(),
        "    WITH DATA {\"name\": \"John\", \"age\": 30}".to_string(),
    ];

    let command = parse_block(&lines).unwrap();
    let display_output = format!("{}", command);

    assert!(display_output.contains("POST https://api.example.com/users"));
    assert!(display_output.contains("Content-Type: application/json"));
    assert!(display_output.contains("Authorization: Bearer token"));
    assert!(display_output.contains(r#"{"name": "John", "age": 30}"#));
}
