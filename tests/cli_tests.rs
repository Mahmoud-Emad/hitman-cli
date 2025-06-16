//! CLI integration tests for the Hitman HTTP client.

use std::process::Command;

fn get_binary_path() -> String {
    // Get the path to the compiled binary
    let output = Command::new("cargo")
        .args(["build", "--quiet"])
        .output()
        .expect("Failed to build binary");

    if !output.status.success() {
        panic!(
            "Failed to build binary: {}",
            String::from_utf8_lossy(&output.stderr)
        );
    }

    "./target/debug/hitman".to_string()
}

#[test]
fn test_cli_help() {
    let output = Command::new(get_binary_path())
        .arg("--help")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Hitman is a lightweight HTTP testing tool"));
    assert!(stdout.contains("--log"));
    assert!(stdout.contains("--strict"));
    assert!(stdout.contains("--report"));
    assert!(stdout.contains("--dry-run"));
    assert!(stdout.contains("--timeout"));
    assert!(stdout.contains("--color"));
}

#[test]
fn test_cli_version() {
    let output = Command::new(get_binary_path())
        .arg("--version")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("hitman 0.2.0"));
}

#[test]
fn test_cli_simple_file() {
    let output = Command::new(get_binary_path())
        .arg("tests/fixtures/simple.hit")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 4 HTTP command block(s)"));
    assert!(stdout.contains("GET https://jsonplaceholder.typicode.com/posts"));
    assert!(stdout.contains("POST https://reqres.in/api/users"));
}

#[test]
fn test_cli_verbose_mode() {
    let output = Command::new(get_binary_path())
        .args(["-v", "tests/fixtures/simple.hit"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // In verbose mode, we should see the request arrows and response details
    assert!(stdout.contains("→")); // Request arrow
    assert!(stdout.contains("←")); // Response arrow
    assert!(stdout.contains("Headers:"));
    assert!(stdout.contains("Body:"));
}

#[test]
fn test_cli_dry_run() {
    let output = Command::new(get_binary_path())
        .args(["--dry-run", "tests/fixtures/simple.hit"])
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    // Should contain dry run messages
    assert!(stdout.contains("🔍 Dry run - request not executed"));
    assert!(stdout.contains("🔍 Dry run completed:"));
}

#[test]
fn test_cli_nonexistent_file() {
    let output = Command::new(get_binary_path())
        .arg("nonexistent.hit")
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Failed to read file"));
}

#[test]
fn test_cli_invalid_file() {
    let output = Command::new(get_binary_path())
        .arg("tests/fixtures/invalid.hit")
        .output()
        .expect("Failed to execute command");

    // Should succeed but show parsing errors
    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    let stderr = String::from_utf8_lossy(&output.stderr);

    // Check that it found the blocks (now 6 blocks due to our updated invalid.hit)
    assert!(stdout.contains("Found 6 HTTP command block(s)"));

    // Check for error messages in both stdout and stderr
    let combined_output = format!("{}{}", stdout, stderr);
    assert!(
        combined_output.contains("Failed to parse block")
            || combined_output.contains("Invalid HTTP method")
            || combined_output.contains("Invalid URL format")
            || combined_output.contains("Malformed JSON")
            || combined_output.contains("Unexpected line content")
            || combined_output.contains("Duplicate WITH HEADER")
            || combined_output.contains("Unknown directive")
    );
}

#[test]
fn test_cli_complex_file() {
    let output = Command::new(get_binary_path())
        .arg("tests/fixtures/complex.hit")
        .output()
        .expect("Failed to execute command");

    assert!(output.status.success());
    let stdout = String::from_utf8_lossy(&output.stdout);
    assert!(stdout.contains("Found 6 HTTP command block(s)"));
    assert!(stdout.contains("POST https://reqres.in/api/users"));
    assert!(stdout.contains("PUT https://reqres.in/api/users/2"));
    assert!(stdout.contains("DELETE https://reqres.in/api/users/2"));
    assert!(stdout.contains("PATCH https://reqres.in/api/users/2"));
    assert!(stdout.contains("HEAD https://jsonplaceholder.typicode.com/posts/1"));
    assert!(stdout.contains("OPTIONS https://jsonplaceholder.typicode.com/posts"));
}

#[test]
fn test_cli_no_arguments() {
    let output = Command::new(get_binary_path())
        .output()
        .expect("Failed to execute command");

    assert!(!output.status.success());
    assert_eq!(output.status.code(), Some(3)); // Exit code 3 for invalid arguments
    let stderr = String::from_utf8_lossy(&output.stderr);
    assert!(stderr.contains("Must provide either a file path or use --stdin"));
}
