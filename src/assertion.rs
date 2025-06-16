//! Assertion parsing and evaluation for Hitman.

use crate::error::{HitError, Result};
use crate::executor::HitmanResponse;
use crate::variables::VariableStore;
use serde_json::Value;
use std::collections::HashMap;

/// Represents a parsed assertion statement.
#[derive(Debug, Clone, PartialEq)]
pub struct Assertion {
    /// The alias of the request being asserted on
    pub alias: String,
    /// The property path being accessed (e.g., "status", "headers.Content-Type", "json.0.userId")
    pub property_path: String,
    /// The comparison operator
    pub operator: AssertionOperator,
    /// The expected value
    pub expected_value: AssertionValue,
    /// Original line number for error reporting
    pub line_number: usize,
}

/// Supported assertion operators.
#[derive(Debug, Clone, PartialEq)]
pub enum AssertionOperator {
    Equals,
    NotEquals,
    Contains,
}

/// Values that can be compared in assertions.
#[derive(Debug, Clone, PartialEq)]
pub enum AssertionValue {
    String(String),
    Number(f64),
    Boolean(bool),
    Null,
}

/// Result of evaluating an assertion.
#[derive(Debug, Clone)]
pub struct AssertionResult {
    pub assertion: Assertion,
    pub passed: bool,
    pub actual_value: Option<String>,
    pub error_message: Option<String>,
}

/// Parse an ASSERT statement from a line.
pub fn parse_assertion(
    line: &str,
    line_number: usize,
    variables: &VariableStore,
) -> Result<Assertion> {
    let trimmed = line.trim();

    if !trimmed.starts_with("ASSERT ") {
        return Err(HitError::ParseError {
            line: line_number,
            reason: "Line does not start with ASSERT".to_string(),
        });
    }

    // Remove "ASSERT " prefix
    let assertion_content = &trimmed[7..].trim();

    // Apply variable substitution
    let substituted_content = variables.substitute(assertion_content)?;

    // Parse the assertion: alias.property operator expected_value
    let (left_side, operator, right_side) =
        parse_assertion_parts(&substituted_content, line_number)?;

    // Parse the left side to extract alias and property path
    let (alias, property_path) = parse_property_access(&left_side, line_number)?;

    // Parse the expected value
    let expected_value = parse_assertion_value(&right_side, line_number)?;

    Ok(Assertion {
        alias,
        property_path,
        operator,
        expected_value,
        line_number,
    })
}

/// Parse assertion parts: left_side operator right_side
fn parse_assertion_parts(
    content: &str,
    line_number: usize,
) -> Result<(String, AssertionOperator, String)> {
    // Look for operators in order of precedence (longest first to avoid conflicts)
    if let Some(pos) = content.find(" CONTAINS ") {
        let left = content[..pos].trim().to_string();
        let right = content[pos + 10..].trim().to_string(); // " CONTAINS " is 10 chars
        return Ok((left, AssertionOperator::Contains, right));
    }

    if let Some(pos) = content.find(" != ") {
        let left = content[..pos].trim().to_string();
        let right = content[pos + 4..].trim().to_string(); // " != " is 4 chars
        return Ok((left, AssertionOperator::NotEquals, right));
    }

    if let Some(pos) = content.find(" == ") {
        let left = content[..pos].trim().to_string();
        let right = content[pos + 4..].trim().to_string(); // " == " is 4 chars
        return Ok((left, AssertionOperator::Equals, right));
    }

    Err(HitError::ParseError {
        line: line_number,
        reason: "No valid assertion operator found. Supported operators: ==, !=, CONTAINS"
            .to_string(),
    })
}

/// Parse property access: alias.property.path
fn parse_property_access(left_side: &str, line_number: usize) -> Result<(String, String)> {
    if let Some(dot_pos) = left_side.find('.') {
        let alias = left_side[..dot_pos].trim().to_string();
        let property_path = left_side[dot_pos + 1..].trim().to_string();

        if alias.is_empty() || property_path.is_empty() {
            return Err(HitError::ParseError {
                line: line_number,
                reason: "Invalid property access format. Expected: alias.property".to_string(),
            });
        }

        Ok((alias, property_path))
    } else {
        Err(HitError::ParseError {
            line: line_number,
            reason: "Property access must include alias and property separated by dot (e.g., request1.status)".to_string(),
        })
    }
}

/// Parse assertion value from string
fn parse_assertion_value(value_str: &str, _line_number: usize) -> Result<AssertionValue> {
    let trimmed = value_str.trim();

    // Handle quoted strings
    if (trimmed.starts_with('"') && trimmed.ends_with('"'))
        || (trimmed.starts_with('\'') && trimmed.ends_with('\''))
    {
        let unquoted = &trimmed[1..trimmed.len() - 1];
        return Ok(AssertionValue::String(unquoted.to_string()));
    }

    // Handle null
    if trimmed == "null" {
        return Ok(AssertionValue::Null);
    }

    // Handle booleans
    if trimmed == "true" {
        return Ok(AssertionValue::Boolean(true));
    }
    if trimmed == "false" {
        return Ok(AssertionValue::Boolean(false));
    }

    // Handle numbers
    if let Ok(num) = trimmed.parse::<f64>() {
        return Ok(AssertionValue::Number(num));
    }

    // If none of the above, treat as unquoted string
    Ok(AssertionValue::String(trimmed.to_string()))
}

/// Evaluate an assertion against a response.
pub fn evaluate_assertion(
    assertion: &Assertion,
    responses: &HashMap<String, HitmanResponse>,
    _variables: &VariableStore,
) -> AssertionResult {
    // Get the response for the alias
    let response = match responses.get(&assertion.alias) {
        Some(resp) => resp,
        None => {
            return AssertionResult {
                assertion: assertion.clone(),
                passed: false,
                actual_value: None,
                error_message: Some(format!("No response found for alias '{}'. Make sure the request was executed successfully and has an AS clause.", assertion.alias)),
            };
        }
    };

    // Extract the actual value from the response
    let actual_value = match extract_property_value(response, &assertion.property_path) {
        Ok(value) => value,
        Err(err) => {
            return AssertionResult {
                assertion: assertion.clone(),
                passed: false,
                actual_value: None,
                error_message: Some(format!(
                    "Failed to extract property '{}': {}",
                    assertion.property_path, err
                )),
            };
        }
    };

    // Perform the comparison
    let passed = match &assertion.operator {
        AssertionOperator::Equals => compare_values(&actual_value, &assertion.expected_value),
        AssertionOperator::NotEquals => !compare_values(&actual_value, &assertion.expected_value),
        AssertionOperator::Contains => contains_value(&actual_value, &assertion.expected_value),
    };

    AssertionResult {
        assertion: assertion.clone(),
        passed,
        actual_value: Some(format!("{:?}", actual_value)),
        error_message: if passed {
            None
        } else {
            Some(format!(
                "Expected {} {} {:?}, but got {:?}",
                assertion.property_path,
                operator_to_string(&assertion.operator),
                assertion.expected_value,
                actual_value
            ))
        },
    }
}

/// Extract a property value from a response using a property path.
fn extract_property_value(
    response: &HitmanResponse,
    property_path: &str,
) -> Result<AssertionValue> {
    let parts: Vec<&str> = property_path.split('.').collect();

    if parts.is_empty() {
        return Err(HitError::ParseError {
            line: 0,
            reason: "Empty property path".to_string(),
        });
    }

    match parts[0] {
        "status" => {
            if parts.len() != 1 {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: "Status property does not support sub-properties".to_string(),
                });
            }
            Ok(AssertionValue::Number(response.status as f64))
        }
        "headers" => {
            if parts.len() != 2 {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: "Headers property requires exactly one sub-property (header name)"
                        .to_string(),
                });
            }
            let header_name = parts[1];
            match response.headers.get(header_name) {
                Some(value) => Ok(AssertionValue::String(value.clone())),
                None => Ok(AssertionValue::Null),
            }
        }
        "body" => {
            if parts.len() != 1 {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: "Body property does not support sub-properties".to_string(),
                });
            }
            Ok(AssertionValue::String(response.body_raw.clone()))
        }
        "json" => {
            if parts.len() < 2 {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: "JSON property requires at least one sub-property".to_string(),
                });
            }

            let json_value = match &response.body_json {
                Some(json) => json,
                None => return Ok(AssertionValue::Null),
            };

            extract_json_value(json_value, &parts[1..])
        }
        _ => Err(HitError::ParseError {
            line: 0,
            reason: format!(
                "Unknown property '{}'. Supported properties: status, headers, body, json",
                parts[0]
            ),
        }),
    }
}

/// Extract a value from JSON using a path.
fn extract_json_value(json: &Value, path: &[&str]) -> Result<AssertionValue> {
    let mut current = json;

    for part in path {
        match current {
            Value::Object(obj) => {
                current = match obj.get(*part) {
                    Some(value) => value,
                    None => return Ok(AssertionValue::Null),
                };
            }
            Value::Array(arr) => {
                let index: usize = part.parse().map_err(|_| HitError::ParseError {
                    line: 0,
                    reason: format!(
                        "Invalid array index '{}'. Array indices must be numbers",
                        part
                    ),
                })?;

                current = match arr.get(index) {
                    Some(value) => value,
                    None => return Ok(AssertionValue::Null),
                };
            }
            _ => return Ok(AssertionValue::Null),
        }
    }

    // Convert JSON value to AssertionValue
    match current {
        Value::String(s) => Ok(AssertionValue::String(s.clone())),
        Value::Number(n) => Ok(AssertionValue::Number(n.as_f64().unwrap_or(0.0))),
        Value::Bool(b) => Ok(AssertionValue::Boolean(*b)),
        Value::Null => Ok(AssertionValue::Null),
        _ => Ok(AssertionValue::String(current.to_string())),
    }
}

/// Compare two assertion values for equality.
fn compare_values(actual: &AssertionValue, expected: &AssertionValue) -> bool {
    match (actual, expected) {
        (AssertionValue::String(a), AssertionValue::String(e)) => a == e,
        (AssertionValue::Number(a), AssertionValue::Number(e)) => (a - e).abs() < f64::EPSILON,
        (AssertionValue::Boolean(a), AssertionValue::Boolean(e)) => a == e,
        (AssertionValue::Null, AssertionValue::Null) => true,
        // Allow number-string comparisons
        (AssertionValue::Number(a), AssertionValue::String(e)) => a.to_string() == *e,
        (AssertionValue::String(a), AssertionValue::Number(e)) => *a == e.to_string(),
        _ => false,
    }
}

/// Check if actual value contains expected value (for CONTAINS operator).
fn contains_value(actual: &AssertionValue, expected: &AssertionValue) -> bool {
    match (actual, expected) {
        (AssertionValue::String(a), AssertionValue::String(e)) => a.contains(e),
        _ => false, // CONTAINS only works with strings
    }
}

/// Convert operator to string for error messages.
fn operator_to_string(op: &AssertionOperator) -> &'static str {
    match op {
        AssertionOperator::Equals => "==",
        AssertionOperator::NotEquals => "!=",
        AssertionOperator::Contains => "CONTAINS",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::variables::VariableStore;
    use std::collections::HashMap;

    #[test]
    fn test_parse_assertion_basic() {
        let variables = VariableStore::new();
        let assertion = parse_assertion("ASSERT request1.status == 200", 1, &variables).unwrap();

        assert_eq!(assertion.alias, "request1");
        assert_eq!(assertion.property_path, "status");
        assert_eq!(assertion.operator, AssertionOperator::Equals);
        assert_eq!(assertion.expected_value, AssertionValue::Number(200.0));
        assert_eq!(assertion.line_number, 1);
    }

    #[test]
    fn test_parse_assertion_string_value() {
        let variables = VariableStore::new();
        let assertion = parse_assertion(
            "ASSERT request1.headers.Content-Type == \"application/json\"",
            1,
            &variables,
        )
        .unwrap();

        assert_eq!(assertion.alias, "request1");
        assert_eq!(assertion.property_path, "headers.Content-Type");
        assert_eq!(assertion.operator, AssertionOperator::Equals);
        assert_eq!(
            assertion.expected_value,
            AssertionValue::String("application/json".to_string())
        );
    }

    #[test]
    fn test_parse_assertion_contains() {
        let variables = VariableStore::new();
        let assertion =
            parse_assertion("ASSERT request1.body CONTAINS \"success\"", 1, &variables).unwrap();

        assert_eq!(assertion.alias, "request1");
        assert_eq!(assertion.property_path, "body");
        assert_eq!(assertion.operator, AssertionOperator::Contains);
        assert_eq!(
            assertion.expected_value,
            AssertionValue::String("success".to_string())
        );
    }

    #[test]
    fn test_parse_assertion_not_equals() {
        let variables = VariableStore::new();
        let assertion = parse_assertion("ASSERT request1.status != 404", 1, &variables).unwrap();

        assert_eq!(assertion.alias, "request1");
        assert_eq!(assertion.property_path, "status");
        assert_eq!(assertion.operator, AssertionOperator::NotEquals);
        assert_eq!(assertion.expected_value, AssertionValue::Number(404.0));
    }

    #[test]
    fn test_parse_assertion_with_variables() {
        let mut variables = VariableStore::new();
        variables
            .parse_define("DEFINE expectedStatus=200", 1)
            .unwrap();

        let assertion = parse_assertion(
            "ASSERT request1.status == {{expectedStatus}}",
            1,
            &variables,
        )
        .unwrap();

        assert_eq!(assertion.alias, "request1");
        assert_eq!(assertion.property_path, "status");
        assert_eq!(assertion.operator, AssertionOperator::Equals);
        assert_eq!(assertion.expected_value, AssertionValue::Number(200.0));
    }

    #[test]
    fn test_evaluate_assertion_status_success() {
        let mut responses = HashMap::new();
        let response = HitmanResponse {
            status: 200,
            headers: HashMap::new(),
            body_raw: "{}".to_string(),
            body_json: Some(serde_json::json!({})),
            elapsed_ms: 100,
        };
        responses.insert("request1".to_string(), response);

        let assertion = Assertion {
            alias: "request1".to_string(),
            property_path: "status".to_string(),
            operator: AssertionOperator::Equals,
            expected_value: AssertionValue::Number(200.0),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(result.passed);
        assert!(result.error_message.is_none());
    }

    #[test]
    fn test_evaluate_assertion_status_failure() {
        let mut responses = HashMap::new();
        let response = HitmanResponse {
            status: 404,
            headers: HashMap::new(),
            body_raw: "{}".to_string(),
            body_json: Some(serde_json::json!({})),
            elapsed_ms: 100,
        };
        responses.insert("request1".to_string(), response);

        let assertion = Assertion {
            alias: "request1".to_string(),
            property_path: "status".to_string(),
            operator: AssertionOperator::Equals,
            expected_value: AssertionValue::Number(200.0),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(!result.passed);
        assert!(result.error_message.is_some());
    }

    #[test]
    fn test_evaluate_assertion_header() {
        let mut responses = HashMap::new();
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let response = HitmanResponse {
            status: 200,
            headers,
            body_raw: "{}".to_string(),
            body_json: Some(serde_json::json!({})),
            elapsed_ms: 100,
        };
        responses.insert("request1".to_string(), response);

        let assertion = Assertion {
            alias: "request1".to_string(),
            property_path: "headers.Content-Type".to_string(),
            operator: AssertionOperator::Equals,
            expected_value: AssertionValue::String("application/json".to_string()),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(result.passed);
    }

    #[test]
    fn test_evaluate_assertion_json_property() {
        let mut responses = HashMap::new();
        let json_body = serde_json::json!({
            "userId": 123,
            "name": "John Doe"
        });

        let response = HitmanResponse {
            status: 200,
            headers: HashMap::new(),
            body_raw: json_body.to_string(),
            body_json: Some(json_body),
            elapsed_ms: 100,
        };
        responses.insert("request1".to_string(), response);

        let assertion = Assertion {
            alias: "request1".to_string(),
            property_path: "json.userId".to_string(),
            operator: AssertionOperator::Equals,
            expected_value: AssertionValue::Number(123.0),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(result.passed);
    }

    #[test]
    fn test_evaluate_assertion_contains() {
        let mut responses = HashMap::new();
        let response = HitmanResponse {
            status: 200,
            headers: HashMap::new(),
            body_raw: "Operation completed successfully".to_string(),
            body_json: None,
            elapsed_ms: 100,
        };
        responses.insert("request1".to_string(), response);

        let assertion = Assertion {
            alias: "request1".to_string(),
            property_path: "body".to_string(),
            operator: AssertionOperator::Contains,
            expected_value: AssertionValue::String("success".to_string()),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(result.passed);
    }

    #[test]
    fn test_evaluate_assertion_undefined_alias() {
        let responses = HashMap::new(); // Empty - no responses

        let assertion = Assertion {
            alias: "nonexistent".to_string(),
            property_path: "status".to_string(),
            operator: AssertionOperator::Equals,
            expected_value: AssertionValue::Number(200.0),
            line_number: 1,
        };

        let variables = VariableStore::new();
        let result = evaluate_assertion(&assertion, &responses, &variables);

        assert!(!result.passed);
        assert!(result.error_message.is_some());
        assert!(result
            .error_message
            .unwrap()
            .contains("No response found for alias"));
    }
}
