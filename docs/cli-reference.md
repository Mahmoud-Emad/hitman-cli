# CLI Reference

Complete reference for all Hitman command-line options and features.

## Basic Usage

```bash
hitman [OPTIONS] <FILE>
hitman [OPTIONS] --stdin
```

## Arguments

### `<FILE>`
Path to the `.hit` file to execute.

**Example:**
```bash
hitman requests.hit
hitman /path/to/api-tests.hit
```

## Options

### Input Options

#### `--stdin`
Read the `.hit` content from standard input instead of a file.

**Example:**
```bash
echo 'GET https://httpbin.org/get' | hitman --stdin
cat requests.hit | hitman --stdin
```

### Output Control

#### `--verbose, -v`
Enable verbose output with detailed request and response information.

**Example:**
```bash
hitman requests.hit --verbose
hitman requests.hit -v
```

**Output includes:**
- Request headers and body
- Response headers and status
- Timing information
- Variable substitution details

#### `--quiet, -q`
Suppress non-essential output. Only shows errors and final summary.

**Example:**
```bash
hitman requests.hit --quiet
hitman requests.hit -q
```

#### `--log <LEVEL>`
Set the log level for debugging and troubleshooting.

**Levels:** `debug`, `info`, `warn`, `error`

**Example:**
```bash
hitman requests.hit --log debug
hitman requests.hit --log error
```

#### `--color <WHEN>`
Control colored output.

**Values:** `auto`, `always`, `never`

**Example:**
```bash
hitman requests.hit --color always
hitman requests.hit --color never
```

### Execution Control

#### `--dry-run`
Validate the `.hit` file syntax and show what would be executed without sending actual HTTP requests.

**Example:**
```bash
hitman requests.hit --dry-run
```

**Useful for:**
- Syntax validation
- Variable substitution testing
- Request preview

#### `--strict`
Stop execution on the first error instead of continuing with remaining requests.

**Example:**
```bash
hitman requests.hit --strict
```

#### `--parallel`
Execute all HTTP requests in parallel instead of sequentially.

**Example:**
```bash
hitman requests.hit --parallel
```

**Benefits:**
- Faster execution for independent requests
- Better performance testing

**Considerations:**
- May overwhelm servers with many concurrent requests
- Order of execution is not guaranteed

#### `--delay <MILLISECONDS>`
Add a delay between sequential requests (ignored in parallel mode).

**Example:**
```bash
hitman requests.hit --delay 1000    # 1 second delay
hitman requests.hit --delay 500     # 500ms delay
```

#### `--timeout <SECONDS>`
Set the timeout for HTTP requests.

**Default:** 30 seconds

**Example:**
```bash
hitman requests.hit --timeout 60    # 60 second timeout
hitman requests.hit --timeout 5     # 5 second timeout
```

### Variable Management

#### `--define KEY=VALUE`
Override or define variables from the command line. Can be used multiple times.

**Example:**
```bash
hitman requests.hit --define baseUrl="https://staging.api.com"
hitman requests.hit --define token="abc123" --define debug=true
```

**Precedence:** Command line variables override file variables.

#### `--env <FILE>`
Load variables from an environment file.

**Example:**
```bash
hitman requests.hit --env .env
hitman requests.hit --env config/staging.env
```

**Environment file format:**
```env
BASE_URL=https://api.example.com
API_TOKEN=your-token-here
DEBUG=true
```

### Reporting

#### `--report <PATH>`
Save a detailed JSON execution report to the specified file.

**Example:**
```bash
hitman requests.hit --report results.json
hitman requests.hit --report reports/$(date +%Y%m%d_%H%M%S).json
```

**Report includes:**
- Request and response details
- Timing information
- Success/failure status
- Variable values
- Error messages

### Help and Information

#### `--help, -h`
Show help information and exit.

```bash
hitman --help
hitman -h
```

#### `--version, -V`
Show version information and exit.

```bash
hitman --version
hitman -V
```

## Exit Codes

Hitman uses standard exit codes to indicate execution status:

- **0**: Success - All requests completed successfully
- **1**: Parse Error - Invalid `.hit` file syntax
- **2**: Request Error - One or more HTTP requests failed
- **3**: System Error - File not found, permission denied, etc.

## Examples

### Basic Usage
```bash
# Simple execution
hitman api-tests.hit

# Verbose output
hitman api-tests.hit --verbose

# Dry run for validation
hitman api-tests.hit --dry-run
```

### Variable Overrides
```bash
# Override environment
hitman api-tests.hit --define baseUrl="https://staging.api.com"

# Multiple overrides
hitman api-tests.hit \
  --define baseUrl="https://staging.api.com" \
  --define token="staging-token" \
  --define debug=true
```

### Environment Files
```bash
# Use staging environment
hitman api-tests.hit --env environments/staging.env

# Combine with overrides
hitman api-tests.hit --env .env --define debug=true
```

### Execution Modes
```bash
# Parallel execution
hitman api-tests.hit --parallel

# Sequential with delay
hitman api-tests.hit --delay 1000

# Strict mode (stop on first error)
hitman api-tests.hit --strict
```

### Reporting
```bash
# Generate report
hitman api-tests.hit --report results.json

# Quiet execution with report
hitman api-tests.hit --quiet --report results.json

# Timestamped report
hitman api-tests.hit --report "report-$(date +%Y%m%d_%H%M%S).json"
```

### Pipeline Usage
```bash
# From stdin
echo 'GET https://httpbin.org/get' | hitman --stdin

# In CI/CD
hitman api-tests.hit --quiet --strict --report results.json
if [ $? -eq 0 ]; then
    echo "All tests passed"
else
    echo "Tests failed"
    exit 1
fi
```

## Configuration Precedence

Variables are resolved in the following order (highest to lowest priority):

1. **Command line** (`--define`)
2. **Environment file** (`--env`)
3. **File definitions** (`DEFINE` statements)

## Performance Tips

- Use `--parallel` for independent requests
- Use `--quiet` in scripts and CI/CD
- Set appropriate `--timeout` for your network
- Use `--delay` to avoid overwhelming servers
- Use `--dry-run` for syntax validation before execution
