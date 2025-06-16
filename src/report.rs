//! Execution report generation for Hitman.

use crate::assertion::AssertionResult;
use crate::command::HitCommand;
use crate::error::HitError;
use crate::executor::ResponseInfo;
use serde::{Deserialize, Serialize};
use std::fs;
use std::time::{Duration, SystemTime, UNIX_EPOCH};

/// Execution report containing all request results.
#[derive(Debug, Serialize, Deserialize)]
pub struct ExecutionReport {
    /// Timestamp when the execution started
    pub timestamp: u64,
    /// Path to the .hit file that was executed
    pub file_path: String,
    /// Total number of blocks processed
    pub total_blocks: usize,
    /// Number of successful requests
    pub successful_requests: usize,
    /// Number of failed requests
    pub failed_requests: usize,
    /// Total execution time
    pub total_duration_ms: u64,
    /// Individual request results
    pub requests: Vec<RequestResult>,
    /// Number of passed assertions
    pub passed_assertions: usize,
    /// Number of failed assertions
    pub failed_assertions: usize,
    /// Individual assertion results
    pub assertions: Vec<SerializableAssertionResult>,
}

/// Result of a single HTTP request.
#[derive(Debug, Serialize, Deserialize)]
pub struct RequestResult {
    /// Block number (1-based)
    pub block_number: usize,
    /// The HTTP command that was executed
    pub command: Option<SerializableCommand>,
    /// Execution result
    pub result: RequestExecutionResult,
    /// Duration of the request
    pub duration_ms: Option<u64>,
}

/// Serializable version of HitCommand for reports.
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableCommand {
    pub method: String,
    pub url: String,
    pub headers: std::collections::HashMap<String, String>,
    pub query: std::collections::HashMap<String, String>,
    pub body: Option<String>,
    pub alias: Option<String>,
}

/// Serializable version of AssertionResult for reports.
#[derive(Debug, Serialize, Deserialize)]
pub struct SerializableAssertionResult {
    pub alias: String,
    pub property_path: String,
    pub operator: String,
    pub expected_value: String,
    pub actual_value: Option<String>,
    pub passed: bool,
    pub error_message: Option<String>,
    pub line_number: usize,
}

impl From<&HitCommand> for SerializableCommand {
    fn from(cmd: &HitCommand) -> Self {
        SerializableCommand {
            method: cmd.method.to_string(),
            url: cmd.url.clone(),
            headers: cmd.headers.clone(),
            query: cmd.query.clone(),
            body: cmd.body.clone(),
            alias: cmd.alias.clone(),
        }
    }
}

impl From<&AssertionResult> for SerializableAssertionResult {
    fn from(result: &AssertionResult) -> Self {
        SerializableAssertionResult {
            alias: result.assertion.alias.clone(),
            property_path: result.assertion.property_path.clone(),
            operator: match result.assertion.operator {
                crate::assertion::AssertionOperator::Equals => "==".to_string(),
                crate::assertion::AssertionOperator::NotEquals => "!=".to_string(),
                crate::assertion::AssertionOperator::Contains => "CONTAINS".to_string(),
            },
            expected_value: format!("{:?}", result.assertion.expected_value),
            actual_value: result.actual_value.clone(),
            passed: result.passed,
            error_message: result.error_message.clone(),
            line_number: result.assertion.line_number,
        }
    }
}

/// Result of executing a request.
#[derive(Debug, Serialize, Deserialize)]
#[serde(tag = "status", content = "data")]
pub enum RequestExecutionResult {
    /// Request was successful
    Success {
        status_code: u16,
        status_text: String,
        response_size: usize,
        response_headers: Vec<(String, String)>,
        response_body_preview: String, // First 500 chars
    },
    /// Request failed during execution
    ExecutionError { error: String },
    /// Request failed during parsing
    ParseError { error: String },
    /// Request was skipped (dry run)
    Skipped,
}

impl ExecutionReport {
    /// Create a new execution report.
    pub fn new(file_path: String) -> Self {
        let timestamp = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();

        ExecutionReport {
            timestamp,
            file_path,
            total_blocks: 0,
            successful_requests: 0,
            failed_requests: 0,
            total_duration_ms: 0,
            requests: Vec::new(),
            passed_assertions: 0,
            failed_assertions: 0,
            assertions: Vec::new(),
        }
    }

    /// Add a successful request result.
    pub fn add_success(
        &mut self,
        block_number: usize,
        command: &HitCommand,
        response: &ResponseInfo,
    ) {
        let response_body_preview = if response.body.len() > 500 {
            format!("{}...", &response.body[..500])
        } else {
            response.body.clone()
        };

        let result = RequestResult {
            block_number,
            command: Some(command.into()),
            result: RequestExecutionResult::Success {
                status_code: response.status,
                status_text: response.status_text.clone(),
                response_size: response.size,
                response_headers: response.headers.clone(),
                response_body_preview,
            },
            duration_ms: Some(response.duration.as_millis() as u64),
        };

        self.requests.push(result);
        self.successful_requests += 1;
        self.total_duration_ms += response.duration.as_millis() as u64;
    }

    /// Add a failed request result.
    pub fn add_execution_error(
        &mut self,
        block_number: usize,
        command: Option<&HitCommand>,
        error: &HitError,
        duration: Option<Duration>,
    ) {
        let result = RequestResult {
            block_number,
            command: command.map(|c| c.into()),
            result: RequestExecutionResult::ExecutionError {
                error: error.to_string(),
            },
            duration_ms: duration.map(|d| d.as_millis() as u64),
        };

        self.requests.push(result);
        self.failed_requests += 1;
        if let Some(d) = duration {
            self.total_duration_ms += d.as_millis() as u64;
        }
    }

    /// Add a parse error result.
    pub fn add_parse_error(&mut self, block_number: usize, error: &HitError) {
        let result = RequestResult {
            block_number,
            command: None,
            result: RequestExecutionResult::ParseError {
                error: error.to_string(),
            },
            duration_ms: None,
        };

        self.requests.push(result);
        self.failed_requests += 1;
    }

    /// Add a skipped request (dry run).
    pub fn add_skipped(&mut self, block_number: usize, command: &HitCommand) {
        let result = RequestResult {
            block_number,
            command: Some(command.into()),
            result: RequestExecutionResult::Skipped,
            duration_ms: None,
        };

        self.requests.push(result);
    }

    /// Add an assertion result.
    pub fn add_assertion_result(&mut self, result: &AssertionResult) {
        if result.passed {
            self.passed_assertions += 1;
        } else {
            self.failed_assertions += 1;
        }
        self.assertions.push(result.into());
    }

    /// Finalize the report with total counts.
    pub fn finalize(&mut self, total_blocks: usize) {
        self.total_blocks = total_blocks;
    }

    /// Save the report to a JSON file.
    pub fn save_to_file(&self, path: &str) -> Result<(), Box<dyn std::error::Error>> {
        let json = serde_json::to_string_pretty(self)?;
        fs::write(path, json)?;
        Ok(())
    }
}
