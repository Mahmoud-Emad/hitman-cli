//! Hitman - A simple HTTP client for executing .hit files.

use clap::{Parser, ValueEnum};
use hitman::{
    parse_block_with_variables, parse_file_with_variables, ExecutionReport, HitError, HttpExecutor,
};
use std::fs;
use std::io::{self, Read};
use std::process;

/// Color output options
#[derive(Debug, Clone, ValueEnum)]
pub enum ColorOption {
    /// Automatically detect if colors should be used
    Auto,
    /// Always use colors
    Always,
    /// Never use colors
    Never,
}

impl Default for ColorOption {
    fn default() -> Self {
        ColorOption::Auto
    }
}

/// Log level options
#[derive(Debug, Clone, ValueEnum)]
pub enum LogLevel {
    /// Debug level logging
    Debug,
    /// Info level logging
    Info,
    /// Warning level logging
    Warn,
    /// Error level logging
    Error,
}

impl Default for LogLevel {
    fn default() -> Self {
        LogLevel::Info
    }
}

/// A simple HTTP client for executing .hit files
#[derive(Parser)]
#[command(name = "hitman")]
#[command(about = "A simple HTTP client for executing .hit files")]
#[command(version = "0.1.0")]
#[command(
    long_about = "Hitman is a lightweight HTTP testing tool that reads .hit files and executes HTTP requests. \
Each .hit file contains simplified request blocks with support for headers, JSON data, and comments."
)]
struct Cli {
    /// Path to the .hit file to execute, or use --stdin to read from stdin
    #[arg(help = "Path to the .hit file containing HTTP requests")]
    file: Option<String>,

    /// Read from stdin instead of a file
    #[arg(
        long,
        help = "Read .hit content from stdin instead of a file",
        conflicts_with = "file"
    )]
    stdin: bool,

    /// Enable verbose output
    #[arg(
        short,
        long,
        help = "Enable verbose output with detailed request/response information"
    )]
    verbose: bool,

    /// Suppress non-essential output
    #[arg(
        short,
        long,
        help = "Suppress non-essential output, only show errors and final status",
        conflicts_with = "verbose"
    )]
    quiet: bool,

    /// Set log level
    #[arg(
        long,
        value_enum,
        default_value = "info",
        help = "Set logging level (debug|info|warn|error)"
    )]
    log: LogLevel,

    /// Fail immediately on first error
    #[arg(
        long,
        help = "Stop execution on the first syntax, validation, or execution error"
    )]
    strict: bool,

    /// Save execution report to file
    #[arg(
        long,
        value_name = "PATH",
        help = "Save a structured execution report (JSON format) to the specified file"
    )]
    report: Option<String>,

    /// Parse and validate only, don't execute requests
    #[arg(
        long,
        help = "Parse and validate the file without sending HTTP requests"
    )]
    dry_run: bool,

    /// Request timeout in seconds
    #[arg(
        long,
        value_name = "SECONDS",
        default_value = "30",
        help = "Timeout for each HTTP request in seconds"
    )]
    timeout: u64,

    /// Control color output
    #[arg(
        long,
        value_enum,
        default_value = "auto",
        help = "Control ANSI color output"
    )]
    color: ColorOption,

    /// Execute requests in parallel when safe
    #[arg(long, help = "Execute HTTP requests in parallel when possible")]
    parallel: bool,

    /// Delay between requests in milliseconds
    #[arg(
        long,
        value_name = "MS",
        help = "Delay between sequential requests in milliseconds"
    )]
    delay: Option<u64>,

    /// Override or add variables from command line
    #[arg(
        long,
        value_name = "KEY=VALUE",
        help = "Override or add variables (can be used multiple times)",
        action = clap::ArgAction::Append
    )]
    define: Vec<String>,

    /// Load variables from environment file
    #[arg(
        long,
        value_name = "FILE",
        help = "Load variables from environment file (.env format)"
    )]
    env: Option<String>,
}

#[tokio::main]
async fn main() {
    let cli = Cli::parse();

    // Validate CLI arguments
    if !cli.stdin && cli.file.is_none() {
        eprintln!("Error: Must provide either a file path or use --stdin");
        process::exit(3);
    }

    match run(&cli).await {
        Ok(()) => process::exit(0),
        Err(e) => {
            eprintln!("Error: {}", e);
            // Determine exit code based on error type
            let exit_code = match e {
                HitError::ParseError { .. }
                | HitError::MalformedJson { .. }
                | HitError::MalformedHeader { .. }
                | HitError::InvalidMethod { .. }
                | HitError::InvalidUrl { .. }
                | HitError::EmptyBlock
                | HitError::DuplicateDirective { .. }
                | HitError::UnknownDirective { .. } => 1, // Parse/syntax errors
                HitError::RequestError { .. } | HitError::FileRead { .. } => 2, // Runtime/network errors
                HitError::UrlParseError { .. } => 1,                            // Parse errors
            };
            process::exit(exit_code);
        }
    }
}

async fn run(cli: &Cli) -> Result<(), HitError> {
    let start_time = std::time::Instant::now();

    // Determine color usage
    let use_colors = match cli.color {
        ColorOption::Always => true,
        ColorOption::Never => false,
        ColorOption::Auto => atty::is(atty::Stream::Stdout),
    };

    // Read content from file or stdin
    let (content, source_name) = if cli.stdin {
        let mut buffer = String::new();
        io::stdin()
            .read_to_string(&mut buffer)
            .map_err(|e| HitError::FileRead {
                path: "<stdin>".to_string(),
                source: e,
            })?;
        (buffer, "<stdin>".to_string())
    } else {
        let file_path = cli.file.as_ref().unwrap(); // Safe because we validated this in main
        let content = fs::read_to_string(file_path).map_err(|e| HitError::FileRead {
            path: file_path.clone(),
            source: e,
        })?;
        (content, file_path.clone())
    };

    // Parse file with variables
    let (blocks, mut variables) = parse_file_with_variables(&content).map_err(|e| {
        if !cli.quiet {
            eprintln!("❌ Failed to parse file: {}", e);
        }
        e
    })?;

    // Process command line variable overrides
    if !cli.define.is_empty() {
        for define_arg in &cli.define {
            if let Some(eq_pos) = define_arg.find('=') {
                let name = &define_arg[..eq_pos];
                let value = &define_arg[eq_pos + 1..];
                let define_line = format!("DEFINE {}={}", name, value);
                variables.parse_define(&define_line, 0)?;
            } else {
                return Err(HitError::ParseError {
                    line: 0,
                    reason: format!(
                        "Invalid --define format: '{}'. Expected KEY=VALUE",
                        define_arg
                    ),
                });
            }
        }
    }

    // Load environment file if specified
    if let Some(env_file) = &cli.env {
        let env_content = fs::read_to_string(env_file).map_err(|e| HitError::FileRead {
            path: env_file.clone(),
            source: e,
        })?;

        for line in env_content.lines() {
            let line = line.trim();
            if line.is_empty() || line.starts_with('#') {
                continue;
            }
            if let Some(eq_pos) = line.find('=') {
                let name = line[..eq_pos].trim();
                let value = line[eq_pos + 1..].trim();
                let define_line = format!("DEFINE {}={}", name, value);
                variables.parse_define(&define_line, 0)?;
            }
        }
    }

    if blocks.is_empty() {
        if !cli.quiet {
            println!("No HTTP commands found in {}", source_name);
        }
        return Ok(());
    }

    // Initialize report if requested
    let mut report = cli
        .report
        .as_ref()
        .map(|_| ExecutionReport::new(source_name.clone()));

    // Create executor with timeout and color settings
    let executor = HttpExecutor::new(cli.timeout, use_colors);
    let mut successful_requests = 0;
    let mut failed_requests = 0;

    // Determine logging level
    let detailed_logging = cli.verbose || matches!(cli.log, LogLevel::Debug | LogLevel::Info);

    if !cli.quiet {
        println!(
            "Found {} HTTP command block(s) in {}",
            blocks.len(),
            source_name
        );
    }
    println!();

    // Show variables if any were defined
    if !variables.variable_names().is_empty() && detailed_logging {
        println!("📋 Variables defined:");
        for name in variables.variable_names() {
            if let Some(value) = variables.get(name) {
                println!("  {} = {}", name, value.to_string());
            }
        }
        println!();
    }

    // Process each block
    if cli.parallel && !cli.dry_run {
        // Parallel execution
        let mut tasks = Vec::new();

        for (i, block) in blocks.iter().enumerate() {
            let block_number = i + 1;
            let block = block.clone();
            let variables = variables.clone();
            let executor = HttpExecutor::new(cli.timeout, use_colors);

            let task = tokio::spawn(async move {
                (
                    block_number,
                    parse_block_with_variables(&block, &variables),
                    executor,
                )
            });
            tasks.push(task);
        }

        // Wait for all tasks to complete
        for task in tasks {
            let (block_number, parse_result, executor) = task.await.unwrap();

            if !cli.quiet {
                println!("=== Block {} ===", block_number);
            }

            match parse_result {
                Ok(command) => {
                    if detailed_logging {
                        executor.log_request(&command, detailed_logging);
                    }

                    let request_start = std::time::Instant::now();
                    match executor.execute(&command).await {
                        Ok(response) => {
                            if detailed_logging {
                                executor.log_response(&response, detailed_logging);
                            }
                            successful_requests += 1;

                            if let Some(ref mut report) = report {
                                report.add_success(block_number, &command, &response);
                            }
                        }
                        Err(e) => {
                            if !cli.quiet {
                                eprintln!("  ❌ Request failed: {}", e);
                            }
                            failed_requests += 1;

                            if let Some(ref mut report) = report {
                                let duration = request_start.elapsed();
                                report.add_execution_error(
                                    block_number,
                                    Some(&command),
                                    &e,
                                    Some(duration),
                                );
                            }

                            if cli.strict {
                                eprintln!(
                                    "❌ Strict mode: stopping execution due to request failure"
                                );
                                process::exit(2);
                            }
                        }
                    }
                }
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("❌ Failed to parse block {}: {}", block_number, e);
                    }
                    failed_requests += 1;

                    if let Some(ref mut report) = report {
                        report.add_parse_error(block_number, &e);
                    }

                    if cli.strict {
                        eprintln!("❌ Strict mode: stopping execution due to parse error");
                        process::exit(1);
                    }
                }
            }
        }
    } else {
        // Sequential execution
        for (i, block) in blocks.iter().enumerate() {
            let block_number = i + 1;

            if !cli.quiet {
                println!("=== Block {} ===", block_number);
            }

            match parse_block_with_variables(block, &variables) {
                Ok(command) => {
                    // Log the request
                    if detailed_logging {
                        executor.log_request(&command, detailed_logging);
                    }

                    if !cli.dry_run {
                        // Execute the HTTP request
                        let request_start = std::time::Instant::now();
                        match executor.execute(&command).await {
                            Ok(response) => {
                                if detailed_logging {
                                    executor.log_response(&response, detailed_logging);
                                }
                                successful_requests += 1;

                                // Add to report
                                if let Some(ref mut report) = report {
                                    report.add_success(block_number, &command, &response);
                                }
                            }
                            Err(e) => {
                                if !cli.quiet {
                                    eprintln!("  ❌ Request failed: {}", e);
                                }
                                failed_requests += 1;

                                // Add to report
                                if let Some(ref mut report) = report {
                                    let duration = request_start.elapsed();
                                    report.add_execution_error(
                                        block_number,
                                        Some(&command),
                                        &e,
                                        Some(duration),
                                    );
                                }

                                // Exit immediately if strict mode is enabled
                                if cli.strict {
                                    eprintln!(
                                        "❌ Strict mode: stopping execution due to request failure"
                                    );
                                    process::exit(2);
                                }
                            }
                        }

                        // Add delay between requests if specified
                        if let Some(delay_ms) = cli.delay {
                            if i < blocks.len() - 1 {
                                // Don't delay after the last request
                                tokio::time::sleep(tokio::time::Duration::from_millis(delay_ms))
                                    .await;
                            }
                        }
                    } else {
                        if !cli.quiet {
                            println!("  🔍 Dry run - request not executed");
                        }

                        // Add to report as skipped
                        if let Some(ref mut report) = report {
                            report.add_skipped(block_number, &command);
                        }
                    }
                }
                Err(e) => {
                    if !cli.quiet {
                        eprintln!("❌ Failed to parse block {}: {}", block_number, e);
                        if detailed_logging {
                            eprintln!("   Block content: {:?}", block);
                        }
                    }
                    failed_requests += 1;

                    // Add to report
                    if let Some(ref mut report) = report {
                        report.add_parse_error(block_number, &e);
                    }

                    // Exit immediately if strict mode is enabled
                    if cli.strict {
                        eprintln!("❌ Strict mode: stopping execution due to parse error");
                        process::exit(1);
                    }
                }
            }

            if !cli.quiet {
                println!();
            }
        }
    }

    // Print summary
    let total_duration = start_time.elapsed();
    if !cli.quiet {
        if !cli.dry_run {
            println!("📊 Summary:");
            println!("  ✅ Successful requests: {}", successful_requests);
            if failed_requests > 0 {
                println!("  ❌ Failed requests: {}", failed_requests);
            }
            println!("  📁 Total blocks processed: {}", blocks.len());
            println!("  ⏱️  Total execution time: {:.2?}", total_duration);
        } else {
            println!("🔍 Dry run completed:");
            println!("  📁 Total blocks validated: {}", blocks.len());
            println!("  ❌ Parse errors: {}", failed_requests);
            println!("  ⏱️  Validation time: {:.2?}", total_duration);
        }
    }

    // Save report if requested
    if let Some(ref mut report) = report {
        report.finalize(blocks.len());
        if let Some(ref report_path) = cli.report {
            match report.save_to_file(report_path) {
                Ok(()) => {
                    if !cli.quiet {
                        println!("📄 Report saved to: {}", report_path);
                    }
                }
                Err(e) => eprintln!("❌ Failed to save report: {}", e),
            }
        }
    }

    Ok(())
}
