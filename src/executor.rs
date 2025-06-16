//! HTTP request execution using reqwest.

use crate::command::{HitCommand, HttpMethod};
use crate::error::{HitError, Result};
use colored::*;
use reqwest::Client;
use serde_json::Value;
use std::collections::HashMap;
use std::time::{Duration, Instant};

/// HTTP request executor.
pub struct HttpExecutor {
    client: Client,
    use_colors: bool,
}

/// Response information for logging.
#[derive(Debug)]
pub struct ResponseInfo {
    pub status: u16,
    pub status_text: String,
    pub headers: Vec<(String, String)>,
    pub body: String,
    pub duration: Duration,
    pub size: usize,
}

/// Response information for assertions and storage.
#[derive(Debug, Clone)]
pub struct HitmanResponse {
    pub status: u16,
    pub headers: HashMap<String, String>,
    pub body_raw: String,
    pub body_json: Option<Value>,
    pub elapsed_ms: u64,
}

impl From<&ResponseInfo> for HitmanResponse {
    fn from(response: &ResponseInfo) -> Self {
        // Convert headers from Vec<(String, String)> to HashMap<String, String>
        let headers: HashMap<String, String> = response.headers.iter().cloned().collect();

        // Try to parse body as JSON
        let body_json = serde_json::from_str::<Value>(&response.body).ok();

        Self {
            status: response.status,
            headers,
            body_raw: response.body.clone(),
            body_json,
            elapsed_ms: response.duration.as_millis() as u64,
        }
    }
}

impl HttpExecutor {
    /// Create a new HTTP executor with custom timeout.
    pub fn new(timeout_secs: u64, use_colors: bool) -> Self {
        let client = Client::builder()
            .timeout(Duration::from_secs(timeout_secs))
            .user_agent("Hitman/0.1.0")
            .build()
            .expect("Failed to create HTTP client");

        Self { client, use_colors }
    }

    /// Execute an HTTP command and return response information.
    pub async fn execute(&self, command: &HitCommand) -> Result<ResponseInfo> {
        let start_time = Instant::now();

        // Build URL with query parameters
        let mut url = url::Url::parse(&command.url).map_err(|e| HitError::ParseError {
            line: 0,
            reason: format!("Invalid URL: {}", e),
        })?;

        // Add query parameters
        for (key, value) in &command.query {
            url.query_pairs_mut().append_pair(key, value);
        }

        let final_url = url.as_str();

        // Build the request
        let mut request_builder = match &command.method {
            HttpMethod::Get => self.client.get(final_url),
            HttpMethod::Post => self.client.post(final_url),
            HttpMethod::Put => self.client.put(final_url),
            HttpMethod::Patch => self.client.patch(final_url),
            HttpMethod::Delete => self.client.delete(final_url),
            HttpMethod::Head => self.client.head(final_url),
            HttpMethod::Options => self.client.request(reqwest::Method::OPTIONS, final_url),
            HttpMethod::Trace => self.client.request(reqwest::Method::TRACE, final_url),
            HttpMethod::Connect => self.client.request(reqwest::Method::CONNECT, final_url),
            HttpMethod::Propfind => self
                .client
                .request(reqwest::Method::from_bytes(b"PROPFIND").unwrap(), final_url),
            HttpMethod::Proppatch => self.client.request(
                reqwest::Method::from_bytes(b"PROPPATCH").unwrap(),
                final_url,
            ),
            HttpMethod::Mkcol => self
                .client
                .request(reqwest::Method::from_bytes(b"MKCOL").unwrap(), final_url),
            HttpMethod::Copy => self
                .client
                .request(reqwest::Method::from_bytes(b"COPY").unwrap(), final_url),
            HttpMethod::Move => self
                .client
                .request(reqwest::Method::from_bytes(b"MOVE").unwrap(), final_url),
            HttpMethod::Lock => self
                .client
                .request(reqwest::Method::from_bytes(b"LOCK").unwrap(), final_url),
            HttpMethod::Unlock => self
                .client
                .request(reqwest::Method::from_bytes(b"UNLOCK").unwrap(), final_url),
        };

        // Add headers
        for (key, value) in &command.headers {
            request_builder = request_builder.header(key, value);
        }

        // Add body if present
        if let Some(ref body) = command.body {
            request_builder = request_builder.body(body.clone());
        }

        // Execute the request
        let response = request_builder
            .send()
            .await
            .map_err(|e| HitError::RequestError { source: e })?;

        let duration = start_time.elapsed();
        let status = response.status().as_u16();
        let status_text = response.status().to_string();

        // Collect response headers
        let headers: Vec<(String, String)> = response
            .headers()
            .iter()
            .map(|(name, value)| {
                (
                    name.to_string(),
                    value.to_str().unwrap_or("<invalid>").to_string(),
                )
            })
            .collect();

        // Get response body
        let body = response
            .text()
            .await
            .map_err(|e| HitError::RequestError { source: e })?;

        let size = body.len();

        Ok(ResponseInfo {
            status,
            status_text,
            headers,
            body,
            duration,
            size,
        })
    }

    /// Log request information.
    pub fn log_request(&self, command: &HitCommand, verbose: bool) {
        // Build URL with query parameters for display
        let display_url = if command.query.is_empty() {
            command.url.clone()
        } else {
            let mut url = url::Url::parse(&command.url).unwrap_or_else(|_| {
                // Fallback if URL parsing fails
                url::Url::parse("http://invalid").unwrap()
            });
            for (key, value) in &command.query {
                url.query_pairs_mut().append_pair(key, value);
            }
            url.to_string()
        };

        let request_line = format!("→ {} {}", command.method, display_url);
        if self.use_colors {
            println!("{}", request_line.cyan().bold());
        } else {
            println!("{}", request_line);
        }

        if verbose {
            if !command.query.is_empty() {
                if self.use_colors {
                    println!("  {}:", "Query".yellow());
                    for (key, value) in &command.query {
                        println!("    {}={}", key.green(), value);
                    }
                } else {
                    println!("  Query:");
                    for (key, value) in &command.query {
                        println!("    {}={}", key, value);
                    }
                }
            }

            if !command.headers.is_empty() {
                if self.use_colors {
                    println!("  {}:", "Headers".yellow());
                    for (key, value) in &command.headers {
                        println!("    {}: {}", key.green(), value);
                    }
                } else {
                    println!("  Headers:");
                    for (key, value) in &command.headers {
                        println!("    {}: {}", key, value);
                    }
                }
            }

            if let Some(ref body) = command.body {
                if self.use_colors {
                    println!("  {}:", "Body".yellow());
                } else {
                    println!("  Body:");
                }
                // Pretty print JSON if possible
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(body) {
                    if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                        for line in pretty_json.lines() {
                            println!("    {}", line);
                        }
                    } else {
                        println!("    {}", body);
                    }
                } else {
                    println!("    {}", body);
                }
            }
        } else {
            let mut info_parts = Vec::new();
            if !command.query.is_empty() {
                info_parts.push(format!("{} query param(s)", command.query.len()));
            }
            if !command.headers.is_empty() {
                info_parts.push(format!("{} header(s)", command.headers.len()));
            }
            if command.body.is_some() {
                info_parts.push("has body".to_string());
            }
            if !info_parts.is_empty() {
                if self.use_colors {
                    println!("  {}", info_parts.join(", ").dimmed());
                } else {
                    println!("  {}", info_parts.join(", "));
                }
            }
        }
    }

    /// Log response information.
    pub fn log_response(&self, response: &ResponseInfo, verbose: bool) {
        let status_color = if response.status >= 200 && response.status < 300 {
            "green"
        } else if response.status >= 400 {
            "red"
        } else {
            "yellow"
        };

        let response_line = format!(
            "← {} {} ({:.2?}, {} bytes)",
            response.status, response.status_text, response.duration, response.size
        );

        if self.use_colors {
            println!("{}", response_line.color(status_color).bold());
        } else {
            println!("{}", response_line);
        }

        if verbose {
            if !response.headers.is_empty() {
                if self.use_colors {
                    println!("  {}:", "Response Headers".yellow());
                    for (key, value) in &response.headers {
                        println!("    {}: {}", key.green(), value);
                    }
                } else {
                    println!("  Response Headers:");
                    for (key, value) in &response.headers {
                        println!("    {}: {}", key, value);
                    }
                }
            }

            if !response.body.is_empty() {
                if self.use_colors {
                    println!("  {}:", "Response Body".yellow());
                } else {
                    println!("  Response Body:");
                }
                // Try to pretty print JSON
                if let Ok(json_value) = serde_json::from_str::<serde_json::Value>(&response.body) {
                    if let Ok(pretty_json) = serde_json::to_string_pretty(&json_value) {
                        for line in pretty_json.lines() {
                            println!("    {}", line);
                        }
                    } else {
                        // Truncate very long responses
                        let truncated = if response.body.len() > 1000 {
                            format!("{}...", &response.body[..1000])
                        } else {
                            response.body.clone()
                        };
                        for line in truncated.lines() {
                            println!("    {}", line);
                        }
                    }
                } else {
                    // Truncate very long responses
                    let truncated = if response.body.len() > 1000 {
                        format!("{}...", &response.body[..1000])
                    } else {
                        response.body.clone()
                    };
                    for line in truncated.lines() {
                        println!("    {}", line);
                    }
                }
            }
        }
    }
}

impl Default for HttpExecutor {
    fn default() -> Self {
        Self::new(30, true) // Default: 30 second timeout, colors enabled
    }
}
