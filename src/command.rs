//! HTTP command representation and validation.

use crate::error::{HitError, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fmt;
use url::Url;

/// Represents a single HTTP command parsed from a .hit file block.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct HitCommand {
    /// HTTP method (GET, POST, PUT, DELETE, etc.)
    pub method: HttpMethod,
    /// Target URL for the request
    pub url: String,
    /// HTTP headers as key-value pairs
    pub headers: HashMap<String, String>,
    /// Query parameters as key-value pairs
    pub query: HashMap<String, String>,
    /// Optional request body (typically JSON)
    pub body: Option<String>,
}

/// Supported HTTP methods (restricted to the allowed set).
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum HttpMethod {
    Get,
    Post,
    Put,
    Patch,
    Delete,
    Head,
    Options,
    Trace,
    Connect,
    Propfind,
    Proppatch,
    Mkcol,
    Copy,
    Move,
    Lock,
    Unlock,
}

impl std::str::FromStr for HttpMethod {
    type Err = HitError;

    fn from_str(method: &str) -> std::result::Result<Self, Self::Err> {
        match method.to_uppercase().as_str() {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "PATCH" => Ok(HttpMethod::Patch),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "TRACE" => Ok(HttpMethod::Trace),
            "CONNECT" => Ok(HttpMethod::Connect),
            "PROPFIND" => Ok(HttpMethod::Propfind),
            "PROPPATCH" => Ok(HttpMethod::Proppatch),
            "MKCOL" => Ok(HttpMethod::Mkcol),
            "COPY" => Ok(HttpMethod::Copy),
            "MOVE" => Ok(HttpMethod::Move),
            "LOCK" => Ok(HttpMethod::Lock),
            "UNLOCK" => Ok(HttpMethod::Unlock),
            _ => Err(HitError::InvalidMethod {
                method: method.to_string(),
            }),
        }
    }
}

impl fmt::Display for HttpMethod {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let method_str = match self {
            HttpMethod::Get => "GET",
            HttpMethod::Post => "POST",
            HttpMethod::Put => "PUT",
            HttpMethod::Delete => "DELETE",
            HttpMethod::Patch => "PATCH",
            HttpMethod::Head => "HEAD",
            HttpMethod::Options => "OPTIONS",
            HttpMethod::Trace => "TRACE",
            HttpMethod::Connect => "CONNECT",
            HttpMethod::Propfind => "PROPFIND",
            HttpMethod::Proppatch => "PROPPATCH",
            HttpMethod::Mkcol => "MKCOL",
            HttpMethod::Copy => "COPY",
            HttpMethod::Move => "MOVE",
            HttpMethod::Lock => "LOCK",
            HttpMethod::Unlock => "UNLOCK",
        };
        write!(f, "{}", method_str)
    }
}

impl HitCommand {
    /// Create a new HTTP command with validation.
    pub fn new(
        method: &str,
        url: &str,
        headers: HashMap<String, String>,
        query: HashMap<String, String>,
        body: Option<String>,
    ) -> Result<Self> {
        let method = method.parse::<HttpMethod>()?;

        // Validate URL using the url crate
        let parsed_url = Url::parse(url).map_err(|e| HitError::UrlParseError { source: e })?;

        // Ensure it's HTTP or HTTPS
        if parsed_url.scheme() != "http" && parsed_url.scheme() != "https" {
            return Err(HitError::InvalidUrl {
                url: url.to_string(),
            });
        }

        // Validate JSON body if present
        if let Some(ref body_content) = body {
            if !body_content.trim().is_empty() {
                serde_json::from_str::<serde_json::Value>(body_content)
                    .map_err(|e| HitError::MalformedJson { line: 0, source: e })?;
            }
        }

        Ok(HitCommand {
            method,
            url: url.to_string(),
            headers,
            query,
            body,
        })
    }
}

impl fmt::Display for HitCommand {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{} {}", self.method, self.url)?;

        if !self.headers.is_empty() {
            write!(f, "  Headers: ")?;
            for (i, (key, value)) in self.headers.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}: {}", key, value)?;
            }
            writeln!(f)?;
        }

        if !self.query.is_empty() {
            write!(f, "  Query: ")?;
            for (i, (key, value)) in self.query.iter().enumerate() {
                if i > 0 {
                    write!(f, ", ")?;
                }
                write!(f, "{}={}", key, value)?;
            }
            writeln!(f)?;
        }

        if let Some(ref body) = self.body {
            writeln!(f, "  Body: {}", body)?;
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_http_method_from_str() {
        assert_eq!("GET".parse::<HttpMethod>().unwrap(), HttpMethod::Get);
        assert_eq!("POST".parse::<HttpMethod>().unwrap(), HttpMethod::Post);
        assert_eq!("put".parse::<HttpMethod>().unwrap(), HttpMethod::Put);
        assert_eq!("Delete".parse::<HttpMethod>().unwrap(), HttpMethod::Delete);

        assert!("INVALID".parse::<HttpMethod>().is_err());
    }

    #[test]
    fn test_http_method_display() {
        assert_eq!(HttpMethod::Get.to_string(), "GET");
        assert_eq!(HttpMethod::Post.to_string(), "POST");
        assert_eq!(HttpMethod::Put.to_string(), "PUT");
        assert_eq!(HttpMethod::Delete.to_string(), "DELETE");
        assert_eq!(HttpMethod::Patch.to_string(), "PATCH");
    }

    #[test]
    fn test_hit_command_new_valid() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());

        let cmd = HitCommand::new(
            "POST",
            "https://api.example.com/users",
            headers.clone(),
            HashMap::new(),
            Some(r#"{"name": "John"}"#.to_string()),
        )
        .unwrap();

        assert_eq!(cmd.method, HttpMethod::Post);
        assert_eq!(cmd.url, "https://api.example.com/users");
        assert_eq!(cmd.headers, headers);
        assert_eq!(cmd.body, Some(r#"{"name": "John"}"#.to_string()));
    }

    #[test]
    fn test_hit_command_new_invalid_method() {
        let result = HitCommand::new(
            "INVALID",
            "https://api.example.com/users",
            HashMap::new(),
            HashMap::new(),
            None,
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::InvalidMethod { method } => assert_eq!(method, "INVALID"),
            _ => panic!("Expected InvalidMethod error"),
        }
    }

    #[test]
    fn test_hit_command_new_invalid_url() {
        let result = HitCommand::new("GET", "invalid-url", HashMap::new(), HashMap::new(), None);

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::UrlParseError { .. } => {} // URL parsing error from url crate
            HitError::InvalidUrl { .. } => {}    // Invalid scheme error
            _ => panic!("Expected URL-related error"),
        }
    }

    #[test]
    fn test_hit_command_new_invalid_json() {
        let result = HitCommand::new(
            "POST",
            "https://api.example.com/users",
            HashMap::new(),
            HashMap::new(),
            Some("invalid json".to_string()),
        );

        assert!(result.is_err());
        match result.unwrap_err() {
            HitError::MalformedJson { .. } => {}
            _ => panic!("Expected MalformedJson error"),
        }
    }

    #[test]
    fn test_hit_command_display() {
        let mut headers = HashMap::new();
        headers.insert("Content-Type".to_string(), "application/json".to_string());
        headers.insert("Authorization".to_string(), "Bearer token".to_string());

        let cmd = HitCommand::new(
            "POST",
            "https://api.example.com/users",
            headers,
            HashMap::new(),
            Some(r#"{"name": "John"}"#.to_string()),
        )
        .unwrap();

        let display_output = format!("{}", cmd);
        assert!(display_output.contains("POST https://api.example.com/users"));
        assert!(display_output.contains("Content-Type: application/json"));
        assert!(display_output.contains("Authorization: Bearer token"));
        assert!(display_output.contains(r#"{"name": "John"}"#));
    }
}
