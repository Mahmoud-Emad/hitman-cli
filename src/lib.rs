//! # Hitman
//!
//! A simple HTTP client for executing `.hit` files containing HTTP requests.
//!
//! ## File Format
//!
//! The `.hit` file format supports HTTP requests in blocks separated by empty lines:
//!
//! ```text
//! # Get all users
//! GET https://api.example.com/users
//!
//! # Create a new user
//! POST https://api.example.com/users
//!     WITH HEADER {Content-Type: application/json}
//!     # User data
//!     WITH DATA {"name": "John", "age": 30}
//! ```
//!
//! ## Features
//! - Comments: Lines starting with `#` are treated as comments
//! - Flexible order: `WITH HEADER` and `WITH DATA` can appear in any order
//! - Multiple headers: Comma-separated key-value pairs in `WITH HEADER`
//! - JSON validation: `WITH DATA` content is validated as JSON

pub mod assertion;
pub mod command;
pub mod error;
pub mod executor;
pub mod parser;
pub mod report;
pub mod variables;

pub use assertion::{evaluate_assertion, parse_assertion, Assertion, AssertionResult};
pub use command::HitCommand;
pub use error::{HitError, Result};
pub use executor::{HitmanResponse, HttpExecutor, ResponseInfo};
pub use parser::{
    parse_block, parse_block_with_variables, parse_file_into_blocks, parse_file_with_assertions,
    parse_file_with_variables,
};
pub use report::ExecutionReport;
pub use variables::{VariableStore, VariableValue};
