//! Variable definition and substitution for Hitman.

use crate::error::{HitError, Result};
use serde_json::Value;
use std::collections::HashMap;

/// Variable storage and substitution engine.
#[derive(Debug, Clone)]
pub struct VariableStore {
    variables: HashMap<String, VariableValue>,
}

/// A variable value that can be a string, number, boolean, or JSON object.
#[derive(Debug, Clone)]
pub enum VariableValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Json(Value),
}

impl std::fmt::Display for VariableValue {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            VariableValue::String(s) => write!(f, "{}", s),
            VariableValue::Number(n) => {
                if n.fract() == 0.0 {
                    write!(f, "{}", *n as i64)
                } else {
                    write!(f, "{}", n)
                }
            }
            VariableValue::Boolean(b) => write!(f, "{}", b),
            VariableValue::Json(v) => write!(f, "{}", v),
        }
    }
}

impl VariableValue {
    /// Get the JSON value representation.
    pub fn to_json(&self) -> Value {
        match self {
            VariableValue::String(s) => Value::String(s.clone()),
            VariableValue::Number(n) => Value::Number(serde_json::Number::from_f64(*n).unwrap()),
            VariableValue::Boolean(b) => Value::Bool(*b),
            VariableValue::Json(v) => v.clone(),
        }
    }
}

impl VariableStore {
    /// Create a new empty variable store.
    pub fn new() -> Self {
        Self {
            variables: HashMap::new(),
        }
    }

    /// Parse a DEFINE line and add the variable to the store.
    pub fn parse_define(&mut self, line: &str, line_number: usize) -> Result<()> {
        let line = line.trim();
        if !line.starts_with("DEFINE ") {
            return Err(HitError::ParseError {
                line: line_number,
                reason: "Expected DEFINE statement".to_string(),
            });
        }

        let definition = &line[7..]; // Remove "DEFINE "
        let eq_pos = definition.find('=').ok_or_else(|| HitError::ParseError {
            line: line_number,
            reason: "Missing '=' in DEFINE statement".to_string(),
        })?;

        let name = definition[..eq_pos].trim();
        let value_part = definition[eq_pos + 1..].trim();

        // Handle inline comments (#, //, or ;) but only if not inside JSON
        let value_str = if value_part.starts_with('{') || value_part.starts_with('[') {
            // For JSON, don't strip comments as they might be inside strings
            value_part
        } else {
            // For simple values, strip inline comments
            // Look for comment markers and find the earliest one, but be careful about // in URLs
            let hash_pos = value_part.find('#');
            let semicolon_pos = value_part.find(';');

            // For //, we need to be more careful - it could be part of a URL
            let slash_pos = find_comment_slash(value_part);

            // Find the earliest comment marker
            let mut earliest_pos = None;

            if let Some(pos) = hash_pos {
                earliest_pos = Some(pos);
            }

            if let Some(pos) = slash_pos {
                if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                    earliest_pos = Some(pos);
                }
            }

            if let Some(pos) = semicolon_pos {
                if earliest_pos.is_none() || pos < earliest_pos.unwrap() {
                    // Check if what comes after semicolon looks like a comment
                    let after_semicolon = value_part[pos + 1..].trim();
                    if after_semicolon.is_empty()
                        || after_semicolon.starts_with('#')
                        || after_semicolon.starts_with("//")
                    {
                        earliest_pos = Some(pos);
                    }
                }
            }

            if let Some(pos) = earliest_pos {
                value_part[..pos].trim()
            } else {
                value_part
            }
        };

        if name.is_empty() {
            return Err(HitError::ParseError {
                line: line_number,
                reason: "Variable name cannot be empty".to_string(),
            });
        }

        // Validate variable name according to identifier rules
        if !is_valid_variable_name(name) {
            return Err(HitError::ParseError {
                line: line_number,
                reason: format!(
                    "Invalid variable name '{}'. Variable names must start with a letter or underscore, and contain only letters, numbers, and underscores",
                    name
                ),
            });
        }

        let value = self.parse_value(value_str, line_number)?;
        self.variables.insert(name.to_string(), value);

        Ok(())
    }

    /// Parse a value from a string.
    fn parse_value(&self, value_str: &str, line_number: usize) -> Result<VariableValue> {
        let value_str = value_str.trim();

        // Try to parse as JSON first (for objects and arrays)
        if value_str.starts_with('{') || value_str.starts_with('[') {
            match serde_json::from_str::<Value>(value_str) {
                Ok(json) => return Ok(VariableValue::Json(json)),
                Err(_) => {
                    return Err(HitError::ParseError {
                        line: line_number,
                        reason: format!("Invalid JSON in variable definition: {}", value_str),
                    });
                }
            }
        }

        // Try to parse as quoted string
        if value_str.starts_with('"') && value_str.ends_with('"') && value_str.len() >= 2 {
            let unquoted = &value_str[1..value_str.len() - 1];
            return Ok(VariableValue::String(unquoted.to_string()));
        }

        // Try to parse as boolean
        if value_str == "true" {
            return Ok(VariableValue::Boolean(true));
        }
        if value_str == "false" {
            return Ok(VariableValue::Boolean(false));
        }

        // Try to parse as number
        if let Ok(num) = value_str.parse::<f64>() {
            return Ok(VariableValue::Number(num));
        }

        // Default to string (unquoted)
        Ok(VariableValue::String(value_str.to_string()))
    }

    /// Substitute variables in a string using {{variable_name}} syntax.
    pub fn substitute(&self, text: &str) -> Result<String> {
        let mut result = text.to_string();

        // Use a while let loop to handle multiple variables in the same string
        while let Some(open_pos) = result.find("{{") {
            if let Some(close_pos) = result[open_pos + 2..].find("}}") {
                let close_pos = open_pos + 2 + close_pos;
                let var_name = &result[open_pos + 2..close_pos];

                if let Some(var_value) = self.variables.get(var_name) {
                    let replacement = var_value.to_string();
                    result.replace_range(open_pos..close_pos + 2, &replacement);
                    // Continue the loop to find more variables
                } else {
                    return Err(HitError::ParseError {
                        line: 0, // Line number not available in this context
                        reason: format!("Undefined variable: {}", var_name),
                    });
                }
            } else {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: "Unclosed variable substitution: missing '}}'".to_string(),
                });
            }
        }

        Ok(result)
    }

    /// Get a variable value by name.
    pub fn get(&self, name: &str) -> Option<&VariableValue> {
        self.variables.get(name)
    }

    /// Check if a variable exists.
    pub fn contains(&self, name: &str) -> bool {
        self.variables.contains_key(name)
    }

    /// Get all variable names.
    pub fn variable_names(&self) -> Vec<&String> {
        self.variables.keys().collect()
    }
}

impl Default for VariableStore {
    fn default() -> Self {
        Self::new()
    }
}

/// Find // comment marker, but avoid treating // in URLs as comments.
/// Returns the position of // if it's likely a comment, None otherwise.
fn find_comment_slash(text: &str) -> Option<usize> {
    let mut pos = 0;
    while let Some(found_pos) = text[pos..].find("//") {
        let absolute_pos = pos + found_pos;

        // Check if this // is likely part of a URL
        // Look for http:// or https:// patterns
        if absolute_pos >= 5 {
            let before = &text[absolute_pos - 5..absolute_pos];
            if before.ends_with("http:") || before.ends_with("ttps:") {
                // This is likely part of a URL, skip it
                pos = absolute_pos + 2;
                continue;
            }
        }

        // Check if there's a space or other separator before //
        // Comments usually have whitespace before them
        if absolute_pos > 0 {
            let char_before = text.chars().nth(absolute_pos - 1);
            if let Some(ch) = char_before {
                if ch.is_whitespace() {
                    // This looks like a comment
                    return Some(absolute_pos);
                }
            }
        } else {
            // // at the beginning of the string is definitely a comment
            return Some(absolute_pos);
        }

        // Continue searching
        pos = absolute_pos + 2;
    }

    None
}

/// Validate variable name according to identifier rules.
/// Variable names must start with letter (a-z, A-Z) or underscore (_)
/// and can contain letters, numbers, underscores.
fn is_valid_variable_name(name: &str) -> bool {
    if name.is_empty() {
        return false;
    }

    let mut chars = name.chars();

    // First character must be letter or underscore
    if let Some(first_char) = chars.next() {
        if !first_char.is_ascii_alphabetic() && first_char != '_' {
            return false;
        }
    } else {
        return false;
    }

    // Remaining characters must be letters, numbers, or underscores
    for ch in chars {
        if !ch.is_ascii_alphanumeric() && ch != '_' {
            return false;
        }
    }

    true
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_string_variable() {
        let mut store = VariableStore::new();
        store
            .parse_define("DEFINE token=\"hello world\"", 1)
            .unwrap();

        assert!(store.contains("token"));
        if let Some(VariableValue::String(s)) = store.get("token") {
            assert_eq!(s, "hello world");
        } else {
            panic!("Expected string variable");
        }
    }

    #[test]
    fn test_parse_number_variable() {
        let mut store = VariableStore::new();
        store.parse_define("DEFINE count=42", 1).unwrap();

        if let Some(VariableValue::Number(n)) = store.get("count") {
            assert_eq!(*n, 42.0);
        } else {
            panic!("Expected number variable");
        }
    }

    #[test]
    fn test_parse_boolean_variable() {
        let mut store = VariableStore::new();
        store.parse_define("DEFINE enabled=true", 1).unwrap();

        if let Some(VariableValue::Boolean(b)) = store.get("enabled") {
            assert_eq!(*b, true);
        } else {
            panic!("Expected boolean variable");
        }
    }

    #[test]
    fn test_parse_json_variable() {
        let mut store = VariableStore::new();
        store
            .parse_define("DEFINE config={\"key\": \"value\"}", 1)
            .unwrap();

        if let Some(VariableValue::Json(json)) = store.get("config") {
            assert_eq!(json["key"], "value");
        } else {
            panic!("Expected JSON variable");
        }
    }

    #[test]
    fn test_variable_substitution() {
        let mut store = VariableStore::new();
        store.parse_define("DEFINE name=\"John\"", 1).unwrap();
        store.parse_define("DEFINE age=30", 2).unwrap();

        let result = store
            .substitute("Hello {{name}}, you are {{age}} years old")
            .unwrap();
        assert_eq!(result, "Hello John, you are 30 years old");
    }

    #[test]
    fn test_undefined_variable_error() {
        let store = VariableStore::new();
        let result = store.substitute("Hello {{undefined}}");
        assert!(result.is_err());
    }

    #[test]
    fn test_unclosed_variable_error() {
        let store = VariableStore::new();
        let result = store.substitute("Hello {{name");
        assert!(result.is_err());
    }

    #[test]
    fn test_multiple_variables_in_json() {
        let mut store = VariableStore::new();
        store
            .parse_define("DEFINE token=\"test_token\"", 1)
            .unwrap();
        store.parse_define("DEFINE number=123", 2).unwrap();
        store.parse_define("DEFINE boolean=true", 3).unwrap();

        let input =
            r#"WITH QUERY { "token": "{{token}}", "number": {{number}}, "boolean": {{boolean}} }"#;
        let result = store.substitute(input).unwrap();
        println!("Input: {}", input);
        println!("Result: {}", result);

        let expected = r#"WITH QUERY { "token": "test_token", "number": 123, "boolean": true }"#;
        assert_eq!(result, expected);
    }

    #[test]
    fn test_query_inside_content() {
        let mut store = VariableStore::new();
        store
            .parse_define("DEFINE token=\"this is a token\"", 1)
            .unwrap();
        store.parse_define("DEFINE number=123", 2).unwrap();
        store.parse_define("DEFINE boolean=true", 3).unwrap();

        // This is what the parse_query_line function extracts as "inside"
        let inside_content =
            r#""token": "{{token}}", "number": {{number}}, "boolean": {{boolean}}"#;
        let result = store.substitute(inside_content);

        println!("Inside content: {}", inside_content);
        match &result {
            Ok(r) => println!("Result: {}", r),
            Err(e) => println!("Error: {}", e),
        }

        assert!(result.is_ok());
        let expected = r#""token": "this is a token", "number": 123, "boolean": true"#;
        assert_eq!(result.unwrap(), expected);
    }

    #[test]
    fn test_inline_comments() {
        let mut store = VariableStore::new();

        // Test with # comments
        store
            .parse_define("DEFINE number=123 # this is a comment", 1)
            .unwrap();
        if let Some(VariableValue::Number(n)) = store.get("number") {
            assert_eq!(*n, 123.0);
        } else {
            panic!("Expected number variable");
        }

        // Test with // comments
        store
            .parse_define("DEFINE count=456 // another comment", 2)
            .unwrap();
        if let Some(VariableValue::Number(n)) = store.get("count") {
            assert_eq!(*n, 456.0);
        } else {
            panic!("Expected number variable");
        }

        // Test with ; comments
        store
            .parse_define("DEFINE active=true; # another comment", 3)
            .unwrap();
        if let Some(VariableValue::Boolean(b)) = store.get("active") {
            assert_eq!(*b, true);
        } else {
            panic!("Expected boolean variable, got: {:?}", store.get("active"));
        }

        // Test with ; only (no comment after)
        store.parse_define("DEFINE flag=false;", 4).unwrap();
        if let Some(VariableValue::Boolean(b)) = store.get("flag") {
            assert_eq!(*b, false);
        } else {
            panic!("Expected boolean variable, got: {:?}", store.get("flag"));
        }
    }

    #[test]
    fn test_variable_name_validation() {
        let mut store = VariableStore::new();

        // Valid variable names
        assert!(store.parse_define("DEFINE validName=123", 1).is_ok());
        assert!(store.parse_define("DEFINE _private=456", 2).is_ok());
        assert!(store.parse_define("DEFINE var123=789", 3).is_ok());
        assert!(store.parse_define("DEFINE apiKey=\"test\"", 4).is_ok());

        // Invalid variable names
        assert!(store.parse_define("DEFINE 123invalid=123", 5).is_err());
        assert!(store.parse_define("DEFINE invalid-name=123", 6).is_err());
        assert!(store.parse_define("DEFINE invalid name=123", 7).is_err());
        assert!(store.parse_define("DEFINE invalid.name=123", 8).is_err());
    }

    #[test]
    fn test_valid_variable_name() {
        assert!(is_valid_variable_name("validName"));
        assert!(is_valid_variable_name("_private"));
        assert!(is_valid_variable_name("var123"));
        assert!(is_valid_variable_name("apiKey"));
        assert!(is_valid_variable_name("a"));
        assert!(is_valid_variable_name("_"));

        assert!(!is_valid_variable_name("123invalid"));
        assert!(!is_valid_variable_name("invalid-name"));
        assert!(!is_valid_variable_name("invalid name"));
        assert!(!is_valid_variable_name("invalid.name"));
        assert!(!is_valid_variable_name(""));
        assert!(!is_valid_variable_name("invalid@name"));
    }
}
