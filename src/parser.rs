//! Parser for .hit file format.

use crate::command::HitCommand;
use crate::error::{HitError, Result};
use crate::variables::VariableStore;
use serde_json;
use std::collections::HashMap;

/// Parse a single block of lines into an HTTP command.
pub fn parse_block(lines: &[String]) -> Result<HitCommand> {
    parse_block_with_variables(lines, &VariableStore::new())
}

/// Parse a single block of lines into an HTTP command with variable substitution.
pub fn parse_block_with_variables(
    lines: &[String],
    variables: &VariableStore,
) -> Result<HitCommand> {
    if lines.is_empty() {
        return Err(HitError::EmptyBlock);
    }

    // Find the first non-comment line for the HTTP method and URL
    let mut method_line_idx = 0;
    let mut method_line = None;

    for (idx, line) in lines.iter().enumerate() {
        let trimmed = line.trim_start();
        if !is_comment_line(trimmed) && !trimmed.is_empty() {
            method_line = Some(line);
            method_line_idx = idx;
            break;
        }
    }

    let first_line = method_line.ok_or_else(|| HitError::ParseError {
        line: 1,
        reason: "No HTTP method line found (only comments or empty lines)".to_string(),
    })?;

    let mut headers = HashMap::new();
    let mut query = HashMap::new();
    let mut body = None;
    let mut has_header_directive = false;
    let mut has_data_directive = false;
    let mut has_query_directive = false;

    // Extract method and URL from the first non-comment line
    let mut parts = first_line.split_whitespace();
    let method = parts.next().ok_or_else(|| HitError::ParseError {
        line: method_line_idx + 1,
        reason: "Missing HTTP method".to_string(),
    })?;
    let url = parts.next().ok_or_else(|| HitError::ParseError {
        line: method_line_idx + 1,
        reason: "Missing URL".to_string(),
    })?;

    // Parse subsequent lines for headers and data (starting after the method line)
    let mut i = method_line_idx + 1;
    while i < lines.len() {
        let line = &lines[i];
        let line_number = i + 1; // +1 because line numbers are 1-based
        let trimmed_line = line.trim_start();

        // Skip comments (lines starting with # or //)
        if is_comment_line(trimmed_line) {
            i += 1;
            continue;
        }

        if trimmed_line.starts_with("WITH HEADER") {
            if has_header_directive {
                return Err(HitError::DuplicateDirective {
                    directive: "WITH HEADER".to_string(),
                    line: line_number,
                });
            }
            has_header_directive = true;
            let (consumed_lines, complete_directive) = parse_multiline_directive(lines, i)?;
            parse_header_line(&complete_directive, line_number, &mut headers, variables)?;
            i += consumed_lines;
        } else if trimmed_line.starts_with("WITH DATA") {
            if has_data_directive {
                return Err(HitError::DuplicateDirective {
                    directive: "WITH DATA".to_string(),
                    line: line_number,
                });
            }
            has_data_directive = true;
            let (consumed_lines, complete_directive) = parse_multiline_directive(lines, i)?;
            body = Some(parse_data_line(
                &complete_directive,
                line_number,
                variables,
            )?);
            i += consumed_lines;
        } else if trimmed_line.starts_with("WITH QUERY") {
            if has_query_directive {
                return Err(HitError::DuplicateDirective {
                    directive: "WITH QUERY".to_string(),
                    line: line_number,
                });
            }
            has_query_directive = true;
            let (consumed_lines, complete_directive) = parse_multiline_directive(lines, i)?;
            parse_query_line(&complete_directive, line_number, &mut query, variables)?;
            i += consumed_lines;
        } else if trimmed_line.starts_with("WITH ") {
            // Unknown directive starting with "WITH "
            let directive = trimmed_line
                .split_whitespace()
                .take(2)
                .collect::<Vec<_>>()
                .join(" ");
            return Err(HitError::UnknownDirective {
                directive,
                line: line_number,
            });
        } else if !line.trim().is_empty() {
            return Err(HitError::ParseError {
                line: line_number,
                reason: format!("Unexpected line content: '{}'", line),
            });
        } else {
            i += 1;
        }
    }

    // Apply variable substitution to method and URL
    let method = variables.substitute(method)?;
    let url = variables.substitute(url)?;

    HitCommand::new(&method, &url, headers, query, body)
}

/// Parse a header line in the format "WITH HEADER {key: value, key2: value2}".
fn parse_header_line(
    line: &str,
    line_number: usize,
    headers: &mut HashMap<String, String>,
    variables: &VariableStore,
) -> Result<()> {
    // Apply variable substitution to the entire line first
    let substituted_line = variables.substitute(line)?;

    let start = substituted_line
        .find('{')
        .ok_or_else(|| HitError::MalformedHeader {
            line: line_number,
            content: line.to_string(),
        })?;

    // Find the matching closing brace by counting braces
    let mut brace_count = 0;
    let mut end = None;

    for (i, ch) in substituted_line[start..].char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    end = Some(start + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end.ok_or_else(|| HitError::MalformedHeader {
        line: line_number,
        content: line.to_string(),
    })?;

    let inside = &substituted_line[start + 1..end];

    // Try to parse as JSON first (for new format with quoted keys)
    let json_str = format!("{{{}}}", inside);
    if let Ok(serde_json::Value::Object(obj)) = serde_json::from_str::<serde_json::Value>(&json_str)
    {
        for (key, value) in obj {
            let value_str = match value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            headers.insert(key, value_str);
        }
        return Ok(());
    }

    // Fall back to legacy format (unquoted keys)
    for header in inside.split(',') {
        let mut kv = header.splitn(2, ':');
        let key = kv.next().ok_or_else(|| HitError::MalformedHeader {
            line: line_number,
            content: header.to_string(),
        })?;
        let value = kv.next().ok_or_else(|| HitError::MalformedHeader {
            line: line_number,
            content: header.to_string(),
        })?;

        headers.insert(key.trim().to_string(), value.trim().to_string());
    }

    Ok(())
}

/// Parse a data line in the format "WITH DATA {json}".
fn parse_data_line(line: &str, line_number: usize, variables: &VariableStore) -> Result<String> {
    let start = line.find('{').ok_or_else(|| HitError::ParseError {
        line: line_number,
        reason: "Missing opening brace for JSON data".to_string(),
    })?;

    // Find the matching closing brace by counting braces
    let mut brace_count = 0;
    let mut end = None;

    for (i, ch) in line[start..].char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    end = Some(start + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end.ok_or_else(|| HitError::ParseError {
        line: line_number,
        reason: "Missing closing brace for JSON data".to_string(),
    })?;

    let json_content = &line[start..=end];

    // Apply variable substitution
    let substituted_content = variables.substitute(json_content)?;

    // Validate JSON syntax
    serde_json::from_str::<serde_json::Value>(&substituted_content).map_err(|e| {
        HitError::MalformedJson {
            line: line_number,
            source: e,
        }
    })?;

    Ok(substituted_content)
}

/// Parse a query line in the format "WITH QUERY {key: value, key2: value2}".
fn parse_query_line(
    line: &str,
    line_number: usize,
    query: &mut HashMap<String, String>,
    variables: &VariableStore,
) -> Result<()> {
    let start = line.find('{').ok_or_else(|| HitError::ParseError {
        line: line_number,
        reason: "Missing opening brace for query parameters".to_string(),
    })?;

    // Find the matching closing brace by counting braces
    let mut brace_count = 0;
    let mut end = None;

    for (i, ch) in line[start..].char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    end = Some(start + i);
                    break;
                }
            }
            _ => {}
        }
    }

    let end = end.ok_or_else(|| HitError::ParseError {
        line: line_number,
        reason: "Missing closing brace for query parameters".to_string(),
    })?;

    let inside = &line[start + 1..end];

    // Apply variable substitution to the entire query block
    let substituted_inside = variables.substitute(inside)?;

    // Parse as JSON to handle proper key-value pairs
    let json_str = format!("{{{}}}", substituted_inside);
    let parsed: serde_json::Value =
        serde_json::from_str(&json_str).map_err(|e| HitError::ParseError {
            line: line_number,
            reason: format!("Invalid JSON in query parameters: {}", e),
        })?;

    if let serde_json::Value::Object(obj) = parsed {
        for (key, value) in obj {
            let value_str = match value {
                serde_json::Value::String(s) => s,
                serde_json::Value::Number(n) => n.to_string(),
                serde_json::Value::Bool(b) => b.to_string(),
                _ => value.to_string(),
            };
            query.insert(key, value_str);
        }
    } else {
        return Err(HitError::ParseError {
            line: line_number,
            reason: "Query parameters must be a JSON object".to_string(),
        });
    }

    Ok(())
}

/// Parse a .hit file content into blocks of lines and extract variables.
pub fn parse_file_with_variables(content: &str) -> Result<(Vec<Vec<String>>, VariableStore)> {
    // First, remove block comments while preserving line structure
    let content_without_block_comments = remove_block_comments(content);
    let lines: Vec<String> = content_without_block_comments
        .lines()
        .map(|s| s.to_string())
        .collect();
    let mut blocks = Vec::new();
    let mut current_block = Vec::new();
    let mut variables = VariableStore::new();
    let mut i = 0;

    while i < lines.len() {
        let line = &lines[i];
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Empty line - end current block if it has content
            if !current_block.is_empty() {
                // Only add blocks that contain at least one non-comment line
                if has_non_comment_line(&current_block) {
                    blocks.push(current_block);
                }
                current_block = Vec::new();
            }
            i += 1;
        } else if trimmed.starts_with("DEFINE ") {
            // Variable definition - might be multi-line JSON
            let (consumed_lines, define_result) = parse_multiline_define(&lines, i)?;
            variables.parse_define(&define_result, i + 1)?;
            i += consumed_lines;
        } else {
            // Non-empty line (including comments) - add to current block
            current_block.push(line.clone());
            i += 1;
        }
    }

    // Don't forget the last block if file doesn't end with empty line
    if !current_block.is_empty() && has_non_comment_line(&current_block) {
        blocks.push(current_block);
    }

    Ok((blocks, variables))
}

/// Parse a potentially multi-line DEFINE statement.
/// Returns (number_of_lines_consumed, complete_define_statement)
fn parse_multiline_define(lines: &[String], start_index: usize) -> Result<(usize, String)> {
    let first_line = &lines[start_index];
    let trimmed = first_line.trim();

    if !trimmed.starts_with("DEFINE ") {
        return Err(HitError::ParseError {
            line: start_index + 1,
            reason: "Expected DEFINE statement".to_string(),
        });
    }

    // Check if this is a simple single-line define
    if !trimmed.contains('{') {
        return Ok((1, first_line.clone()));
    }

    // Check if the JSON is complete on the first line
    let definition_part = &trimmed[7..]; // Remove "DEFINE "
    if let Some(eq_pos) = definition_part.find('=') {
        let value_part = definition_part[eq_pos + 1..].trim();
        if value_part.starts_with('{') {
            let open_braces = value_part.chars().filter(|&c| c == '{').count();
            let close_braces = value_part.chars().filter(|&c| c == '}').count();

            if open_braces == close_braces && open_braces > 0 {
                // Complete JSON on single line
                return Ok((1, first_line.clone()));
            }
        }
    }

    // Multi-line JSON - collect until braces are balanced
    let mut result = first_line.clone();
    let mut open_braces = first_line.chars().filter(|&c| c == '{').count();
    let mut close_braces = first_line.chars().filter(|&c| c == '}').count();
    let mut lines_consumed = 1;

    while open_braces > close_braces && start_index + lines_consumed < lines.len() {
        let next_line = &lines[start_index + lines_consumed];
        result.push(' ');
        result.push_str(next_line.trim());

        open_braces += next_line.chars().filter(|&c| c == '{').count();
        close_braces += next_line.chars().filter(|&c| c == '}').count();
        lines_consumed += 1;
    }

    if open_braces != close_braces {
        return Err(HitError::ParseError {
            line: start_index + 1,
            reason: "Unbalanced braces in multi-line DEFINE statement".to_string(),
        });
    }

    Ok((lines_consumed, result))
}

/// Parse a potentially multi-line directive (WITH HEADER, WITH DATA, WITH QUERY).
/// Returns (number_of_lines_consumed, complete_directive_statement)
fn parse_multiline_directive(lines: &[String], start_index: usize) -> Result<(usize, String)> {
    let first_line = &lines[start_index];

    // Check if this is a simple single-line directive
    if !first_line.contains('{') {
        return Ok((1, first_line.clone()));
    }

    // Find the position of the first opening brace (this is the start of the JSON)
    let json_start = first_line.find('{').unwrap();

    // Check if the JSON is complete on the first line by looking for the matching closing brace
    let mut brace_count = 0;
    let mut found_complete = false;

    for (_i, ch) in first_line[json_start..].char_indices() {
        match ch {
            '{' => brace_count += 1,
            '}' => {
                brace_count -= 1;
                if brace_count == 0 {
                    // Found the matching closing brace
                    found_complete = true;
                    break;
                }
            }
            _ => {}
        }
    }

    if found_complete {
        // Complete JSON on single line
        return Ok((1, first_line.clone()));
    }

    // Multi-line JSON - collect until we find the matching closing brace
    let mut result = first_line.clone();
    let mut brace_count = 0;
    let mut lines_consumed = 1;

    // Count braces in the first line
    for ch in first_line[json_start..].chars() {
        match ch {
            '{' => brace_count += 1,
            '}' => brace_count -= 1,
            _ => {}
        }
    }

    while brace_count > 0 && start_index + lines_consumed < lines.len() {
        let next_line = &lines[start_index + lines_consumed];
        result.push(' ');
        result.push_str(next_line.trim());

        // Count braces in this line
        for ch in next_line.chars() {
            match ch {
                '{' => brace_count += 1,
                '}' => brace_count -= 1,
                _ => {}
            }
        }

        lines_consumed += 1;
    }

    if brace_count != 0 {
        return Err(HitError::ParseError {
            line: start_index + 1,
            reason: "Unbalanced braces in multi-line directive".to_string(),
        });
    }

    Ok((lines_consumed, result))
}

/// Parse a .hit file content into blocks of lines (legacy function for backward compatibility).
pub fn parse_file_into_blocks(content: &str) -> Vec<Vec<String>> {
    let lines: Vec<String> = content.lines().map(|s| s.to_string()).collect();
    let mut blocks = Vec::new();
    let mut current_block = Vec::new();

    for line in lines {
        let trimmed = line.trim();

        if trimmed.is_empty() {
            // Empty line - end current block if it has content
            if !current_block.is_empty() {
                // Only add blocks that contain at least one non-comment line
                if has_non_comment_line(&current_block) {
                    blocks.push(current_block);
                }
                current_block = Vec::new();
            }
        } else {
            // Non-empty line (including comments) - add to current block
            current_block.push(line);
        }
    }

    // Don't forget the last block if file doesn't end with empty line
    if !current_block.is_empty() && has_non_comment_line(&current_block) {
        blocks.push(current_block);
    }

    blocks
}

/// Check if a line is a comment line.
fn is_comment_line(line: &str) -> bool {
    let trimmed = line.trim_start();
    trimmed.starts_with('#') || trimmed.starts_with("//") || trimmed.starts_with("/*")
}

/// Remove block comments from content.
fn remove_block_comments(content: &str) -> String {
    let mut result = String::new();
    let mut chars = content.chars().peekable();
    let mut in_block_comment = false;

    while let Some(ch) = chars.next() {
        if !in_block_comment {
            if ch == '/' && chars.peek() == Some(&'*') {
                // Start of block comment
                chars.next(); // consume the '*'
                in_block_comment = true;
                // Add a space to separate tokens that might be adjacent after comment removal
                result.push(' ');
            } else {
                result.push(ch);
            }
        } else {
            // Inside block comment
            if ch == '*' && chars.peek() == Some(&'/') {
                // End of block comment
                chars.next(); // consume the '/'
                in_block_comment = false;
                // Add a space to separate tokens that might be adjacent after comment removal
                result.push(' ');
            }
            // Skip all other characters in block comments (including newlines)
        }
    }

    result
}

/// Check if a block has at least one non-comment line.
fn has_non_comment_line(block: &[String]) -> bool {
    block.iter().any(|line| {
        let trimmed = line.trim_start();
        !is_comment_line(trimmed) && !trimmed.is_empty()
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_get() {
        let lines = vec!["GET https://api.example.com/users".to_string()];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "GET");
        assert_eq!(result.url, "https://api.example.com/users");
        assert!(result.headers.is_empty());
        assert!(result.body.is_none());
    }

    #[test]
    fn test_parse_post_with_header_and_data() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "    WITH HEADER {Content-Type: application/json}".to_string(),
            "    WITH DATA {\"name\": \"John\", \"age\": 30}".to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "POST");
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            result.body,
            Some("{\"name\": \"John\", \"age\": 30}".to_string())
        );
    }

    #[test]
    fn test_parse_different_order() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "    WITH DATA {\"name\": \"Jane\"}".to_string(),
            "    WITH HEADER {Authorization: Bearer token}".to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "POST");
        assert_eq!(
            result.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
        assert_eq!(result.body, Some("{\"name\": \"Jane\"}".to_string()));
    }

    #[test]
    fn test_parse_multiple_headers() {
        let lines = vec![
            "PUT https://api.example.com/users/1".to_string(),
            "    WITH HEADER {Content-Type: application/json, Authorization: Bearer token}"
                .to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.headers.len(), 2);
        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            result.headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_parse_header_line() {
        let mut headers = HashMap::new();
        let line = "    WITH HEADER {Content-Type: application/json, Authorization: Bearer token}";
        let variables = VariableStore::new();

        parse_header_line(line, 1, &mut headers, &variables).unwrap();

        assert_eq!(headers.len(), 2);
        assert_eq!(
            headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            headers.get("Authorization"),
            Some(&"Bearer token".to_string())
        );
    }

    #[test]
    fn test_parse_data_line() {
        let line = "    WITH DATA {\"name\": \"John\", \"age\": 30}";
        let variables = VariableStore::new();
        let result = parse_data_line(line, 1, &variables).unwrap();

        assert_eq!(result, "{\"name\": \"John\", \"age\": 30}");
    }

    #[test]
    fn test_parse_file_into_blocks() {
        let content = "GET https://api.example.com/users\n\nPOST https://api.example.com/users\n    WITH DATA {\"name\": \"John\"}\n\n";
        let blocks = parse_file_into_blocks(content);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0].len(), 1);
        assert_eq!(blocks[1].len(), 2);
    }

    #[test]
    fn test_parse_file_with_extra_whitespace() {
        let content = "\n\nGET https://api.example.com/users\n\n\n\nPOST https://api.example.com/users\n    WITH DATA {\"name\": \"John\"}\n\n\n";
        let blocks = parse_file_into_blocks(content);

        assert_eq!(blocks.len(), 2);
        assert_eq!(blocks[0], vec!["GET https://api.example.com/users"]);
        assert_eq!(
            blocks[1],
            vec![
                "POST https://api.example.com/users",
                "    WITH DATA {\"name\": \"John\"}"
            ]
        );
    }

    #[test]
    fn test_parse_empty_block() {
        let lines: Vec<String> = vec![];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::EmptyBlock => {}
            _ => panic!("Expected EmptyBlock error"),
        }
    }

    #[test]
    fn test_parse_missing_method() {
        let lines = vec!["".to_string()];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::ParseError { line, reason } => {
                assert_eq!(line, 1);
                assert!(reason.contains("No HTTP method line found"));
            }
            _ => panic!("Expected ParseError for missing method"),
        }
    }

    #[test]
    fn test_parse_missing_url() {
        let lines = vec!["GET".to_string()];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::ParseError { line, reason } => {
                assert_eq!(line, 1);
                assert!(reason.contains("Missing URL"));
            }
            _ => panic!("Expected ParseError for missing URL"),
        }
    }

    #[test]
    fn test_parse_malformed_header() {
        let lines = vec![
            "GET https://api.example.com/users".to_string(),
            "    WITH HEADER {malformed".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::ParseError { line, reason } => {
                assert_eq!(line, 2);
                assert!(reason.contains("Unbalanced braces"));
            }
            _ => panic!("Expected ParseError for unbalanced braces"),
        }
    }

    #[test]
    fn test_parse_malformed_json_data() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "    WITH DATA {invalid json}".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::MalformedJson { line, .. } => assert_eq!(line, 2),
            _ => panic!("Expected MalformedJson error"),
        }
    }

    #[test]
    fn test_parse_unexpected_line() {
        let lines = vec![
            "GET https://api.example.com/users".to_string(),
            "    INVALID_CLAUSE {something: wrong}".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::ParseError { line, reason } => {
                assert_eq!(line, 2);
                assert!(reason.contains("Unexpected line content"));
            }
            _ => panic!("Expected ParseError for unexpected line"),
        }
    }

    #[test]
    fn test_duplicate_header_directive() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "    WITH HEADER {Content-Type: application/json}".to_string(),
            "    WITH HEADER {Authorization: Bearer token}".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::DuplicateDirective { directive, line } => {
                assert_eq!(directive, "WITH HEADER");
                assert_eq!(line, 3);
            }
            _ => panic!("Expected DuplicateDirective error"),
        }
    }

    #[test]
    fn test_duplicate_data_directive() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "    WITH DATA {\"name\": \"John\"}".to_string(),
            "    WITH DATA {\"age\": 30}".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::DuplicateDirective { directive, line } => {
                assert_eq!(directive, "WITH DATA");
                assert_eq!(line, 3);
            }
            _ => panic!("Expected DuplicateDirective error"),
        }
    }

    #[test]
    fn test_unknown_directive() {
        let lines = vec![
            "GET https://api.example.com/users".to_string(),
            "    WITH CAT {meow: purr}".to_string(),
        ];
        let result = parse_block(&lines);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::UnknownDirective { directive, line } => {
                assert_eq!(directive, "WITH CAT");
                assert_eq!(line, 2);
            }
            _ => panic!("Expected UnknownDirective error"),
        }
    }

    #[test]
    fn test_comments_support() {
        let lines = vec![
            "GET https://api.example.com/users".to_string(),
            "# This is a comment".to_string(),
            "    # Another comment with indentation".to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "GET");
        assert_eq!(result.url, "https://api.example.com/users");
        assert!(result.headers.is_empty());
        assert!(result.body.is_none());
    }

    #[test]
    fn test_comments_with_directives() {
        let lines = vec![
            "POST https://api.example.com/users".to_string(),
            "# This request creates a new user".to_string(),
            "    WITH HEADER {Content-Type: application/json}".to_string(),
            "    # The request body contains user data".to_string(),
            "    WITH DATA {\"name\": \"John\", \"age\": 30}".to_string(),
            "# End of request".to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "POST");
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
        assert_eq!(
            result.body,
            Some("{\"name\": \"John\", \"age\": 30}".to_string())
        );
    }

    #[test]
    fn test_double_slash_comments() {
        let lines = vec![
            "GET https://api.example.com/users".to_string(),
            "// This is a double slash comment".to_string(),
            "    // Another comment with indentation".to_string(),
        ];
        let result = parse_block(&lines).unwrap();

        assert_eq!(result.method.to_string(), "GET");
        assert_eq!(result.url, "https://api.example.com/users");
        assert!(result.headers.is_empty());
        assert!(result.body.is_none());
    }

    #[test]
    fn test_block_comments() {
        let content = r#"GET /* inline comment */ https://api.example.com/users
    WITH HEADER {Content-Type: application/json} /* another comment */"#;

        let (blocks, _) = parse_file_with_variables(content).unwrap();
        assert_eq!(blocks.len(), 1);

        let result = parse_block(&blocks[0]).unwrap();
        assert_eq!(result.method.to_string(), "GET");
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_block_comments_multiline() {
        // Test with block comments that don't create empty lines
        let content = r#"GET https://api.example.com/users /* comment
        continues here */
    WITH HEADER {Content-Type: application/json}"#;

        let (blocks, _) = parse_file_with_variables(content).unwrap();
        assert_eq!(blocks.len(), 1);

        let result = parse_block(&blocks[0]).unwrap();
        assert_eq!(result.method.to_string(), "GET");
        assert_eq!(result.url, "https://api.example.com/users");
        assert_eq!(
            result.headers.get("Content-Type"),
            Some(&"application/json".to_string())
        );
    }

    #[test]
    fn test_file_parsing_with_comments() {
        let content = r#"# First request - get all users
GET https://api.example.com/users

# Second request - create a user
POST https://api.example.com/users
    # Set content type
    WITH HEADER {Content-Type: application/json}
    # User data
    WITH DATA {"name": "John"}
"#;
        let blocks = parse_file_into_blocks(content);

        assert_eq!(blocks.len(), 2);

        // First block should have 2 lines (GET + comment)
        assert_eq!(blocks[0].len(), 2);
        assert!(blocks[0][0].contains("# First request"));
        assert!(blocks[0][1].contains("GET"));

        // Second block should have 6 lines (comment + POST + comment + WITH HEADER + comment + WITH DATA)
        assert_eq!(blocks[1].len(), 6);
        assert!(blocks[1][0].contains("# Second request"));
        assert!(blocks[1][1].contains("POST"));
    }

    #[test]
    fn test_multiline_directive_parsing() {
        let lines = vec![
            "WITH QUERY {".to_string(),
            "    \"token\": \"{{token}}\",".to_string(),
            "    \"number\": {{number}},".to_string(),
            "    \"boolean\": {{boolean}}".to_string(),
            "}".to_string(),
        ];

        let (consumed, result) = parse_multiline_directive(&lines, 0).unwrap();
        println!("Consumed: {}", consumed);
        println!("Result: {}", result);

        assert_eq!(consumed, 5);
        assert!(result.contains("WITH QUERY"));
        assert!(result.contains("{{token}}"));
        assert!(result.contains("{{number}}"));
        assert!(result.contains("{{boolean}}"));
    }
}
