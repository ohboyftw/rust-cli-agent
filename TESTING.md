# Testing Guide for rust-cli-agent

This document provides comprehensive information about testing the rust-cli-agent project.

## Table of Contents

- [Test Structure](#test-structure)
- [Running Tests](#running-tests)
- [Test Coverage](#test-coverage)
- [Test Categories](#test-categories)
- [Mocking and Test Utilities](#mocking-and-test-utilities)
- [Continuous Integration](#continuous-integration)
- [Contributing to Tests](#contributing-to-tests)

## Test Structure

The project follows Rust's standard testing conventions with additional integration tests:

```
rust-cli-agent/
├── src/
│   ├── config.rs           # Unit tests included
│   ├── error.rs            # Unit tests included
│   ├── state.rs            # Unit tests included
│   ├── agents/
│   │   ├── planner.rs      # Unit tests included
│   │   └── coder.rs        # Unit tests included
│   └── ...
├── tests/
│   ├── llm_integration_tests.rs    # LLM provider integration tests
│   ├── tools_tests.rs              # Tools functionality tests
│   └── orchestrator_tests.rs       # Orchestrator workflow tests
└── scripts/
    ├── test.sh             # Unix test runner
    └── test.bat            # Windows test runner
```

## Running Tests

### Quick Test Run

```bash
# Run all tests
cargo test

# Run tests with output
cargo test -- --nocapture

# Run specific test
cargo test test_config_load_with_env_vars
```

### Comprehensive Test Suite

Use the provided test scripts for a complete test run:

**Unix/Linux/macOS:**
```bash
./scripts/test.sh
```

**Windows:**
```batch
scripts\test.bat
```

### Test Categories

#### Unit Tests
Run individual module tests:
```bash
cargo test --lib
```

#### Integration Tests
Run integration tests:
```bash
cargo test --test '*'
```

#### Documentation Tests
Run doc tests:
```bash
cargo test --doc
```

## Test Coverage

### Prerequisites

Install coverage tools:
```bash
cargo install cargo-llvm-cov
```

### Generate Coverage Report

```bash
# Generate HTML coverage report
cargo llvm-cov --all-features --workspace --html

# Generate LCOV format for CI
cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
```

Coverage reports are generated in `target/llvm-cov/html/`.

### Coverage Targets

- **Unit Tests**: Aim for >90% coverage
- **Integration Tests**: Critical path coverage
- **Error Handling**: All error paths tested

## Test Categories

### 1. Configuration Tests (`src/config.rs`)

Tests configuration loading with various environment scenarios:

```rust
#[test]
#[serial]
fn test_config_load_with_env_vars() {
    // Tests loading config from environment variables
}

#[test]
#[serial] 
fn test_config_load_with_defaults() {
    // Tests default values when no env vars set
}
```

**Key Features:**
- Environment variable handling
- Default value fallbacks
- Configuration serialization

### 2. Error Handling Tests (`src/error.rs`)

Comprehensive error type testing:

```rust
#[test]
fn test_error_display() {
    // Tests error message formatting
}

#[test]
fn test_io_error_conversion() {
    // Tests automatic error conversions
}
```

**Key Features:**
- Error message formatting
- Error type conversions
- Error chain handling

### 3. State Management Tests (`src/state.rs`)

Application state management:

```rust
#[test]
fn test_get_context_with_history() {
    // Tests context generation with history
}

#[test]
fn test_get_context_with_long_content() {
    // Tests content truncation
}
```

**Key Features:**
- State persistence
- Context generation
- History management
- Content truncation

### 4. Agent Tests (`src/agents/`)

AI agent functionality:

```rust
#[tokio::test]
async fn test_create_plan_success() {
    // Tests plan creation with mock LLM
}

#[test]
fn test_parse_plan_numbered() {
    // Tests plan parsing logic
}
```

**Key Features:**
- Plan generation and parsing
- Code generation
- Prompt building
- Response parsing

### 5. LLM Integration Tests (`tests/llm_integration_tests.rs`)

LLM provider integration with mocking:

```rust
#[tokio::test]
async fn test_ollama_client_success() {
    // Tests successful API calls
}

#[tokio::test]
async fn test_ollama_client_error_response() {
    // Tests error handling
}
```

**Key Features:**
- HTTP mocking with wiremock
- API error handling
- Provider switching
- Authentication testing

### 6. Tools Tests (`tests/tools_tests.rs`)

Tool execution and functionality:

```rust
#[tokio::test]
async fn test_read_file_success() {
    // Tests file reading with temporary files
}

#[tokio::test]
async fn test_run_command_success() {
    // Tests command execution
}
```

**Key Features:**
- File I/O operations
- Command execution
- Search functionality
- Tool serialization

### 7. Orchestrator Tests (`tests/orchestrator_tests.rs`)

High-level workflow orchestration:

```rust
#[tokio::test]
async fn test_orchestrator_components_integration() {
    // Tests agent coordination
}
```

**Key Features:**
- Workflow orchestration
- Agent coordination
- State management integration
- Decision making

## Mocking and Test Utilities

### Mock LLM Client

```rust
struct MockLLMClient {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLLMClient {
    fn new(responses: Vec<String>) -> Self {
        // Predefined responses for testing
    }
}
```

### HTTP Mocking

Using `wiremock` for HTTP API testing:

```rust
let mock_server = MockServer::start().await;

Mock::given(method("POST"))
    .and(path("/api/generate"))
    .respond_with(ResponseTemplate::new(200).set_body_json(/* ... */))
    .mount(&mock_server)
    .await;
```

### Temporary Files

Using `tempfile` for file system testing:

```rust
let temp_dir = tempdir().unwrap();
let file_path = temp_dir.path().join("test.txt");
```

## Environment Variables for Testing

Create `.env.test` file for test configuration:

```bash
# Test API keys (non-functional)
OPENAI_API_KEY=test_openai_key_for_testing
ANTHROPIC_API_KEY=test_anthropic_key_for_testing
GOOGLE_API_KEY=test_google_key_for_testing
DEEPSEEK_API_KEY=test_deepseek_key_for_testing
BRAVE_SEARCH_API_KEY=test_brave_search_key_for_testing

# Test configuration
OLLAMA_BASE_URL=http://localhost:11434
OLLAMA_MODEL=llama3:8b
TEST_TIMEOUT_MS=30000
TEST_LOG_LEVEL=debug
```

## Continuous Integration

### GitHub Actions Workflow

The CI pipeline includes:

1. **Code Quality Checks**
   - `cargo fmt --check`
   - `cargo clippy`

2. **Test Execution**
   - Unit tests
   - Integration tests
   - Documentation tests

3. **Cross-Platform Testing**
   - Ubuntu, Windows, macOS
   - Stable and beta Rust

4. **Coverage Reporting**
   - Code coverage with `cargo-llvm-cov`
   - Upload to Codecov

5. **Security Auditing**
   - `cargo audit` for vulnerabilities

### Local CI Simulation

Run the same checks locally:

```bash
# Format check
cargo fmt --all -- --check

# Lint check
cargo clippy --all-targets --all-features -- -D warnings

# All tests
cargo test --all-features --verbose

# Security audit
cargo audit

# Release build
cargo build --release
```

## Performance Testing

### Benchmarking

For performance-critical components:

```bash
cargo install cargo-criterion
cargo bench
```

### Load Testing

Test with multiple concurrent operations:

```rust
#[tokio::test]
async fn test_concurrent_llm_calls() {
    // Test concurrent API calls
}
```

## Contributing to Tests

### Adding New Tests

1. **Unit Tests**: Add to the same file as the code being tested
2. **Integration Tests**: Add to appropriate file in `tests/` directory
3. **Mock Objects**: Extend existing mocks or create new ones as needed

### Test Guidelines

1. **Naming**: Use descriptive test names that explain what is being tested
2. **Isolation**: Tests should not depend on external services (use mocks)
3. **Cleanup**: Clean up temporary files and environment variables
4. **Documentation**: Document complex test scenarios
5. **Coverage**: Aim to test both success and failure cases

### Test Patterns

```rust
#[tokio::test]
async fn test_feature_success() {
    // Arrange
    let mock_client = setup_mock();
    
    // Act
    let result = feature_under_test(mock_client).await;
    
    // Assert
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), expected_value);
}

#[tokio::test]
async fn test_feature_error_handling() {
    // Test error scenarios
}
```

## Debugging Tests

### Test Output

```bash
# Show test output
cargo test -- --nocapture

# Show test names
cargo test -- --list

# Run single test with output
cargo test test_name -- --nocapture --exact
```

### Environment Variables

```bash
# Enable debug logging
RUST_LOG=debug cargo test

# Test-specific logging
TEST_LOG_LEVEL=trace cargo test
```

### IDE Integration

Most Rust IDEs support:
- Running individual tests
- Debugging tests with breakpoints
- Test coverage visualization

## Test Dependencies

The test suite uses these key dependencies:

```toml
[dev-dependencies]
tokio-test = "0.4"      # Async test utilities
mockall = "0.12"        # Mock generation
tempfile = "3.8"        # Temporary file handling
serial_test = "3.0"     # Sequential test execution
wiremock = "0.5"        # HTTP mocking
test-log = "0.2"        # Test logging
```

## Best Practices

1. **Test Independence**: Each test should be able to run independently
2. **Deterministic**: Tests should produce the same results every time
3. **Fast**: Unit tests should complete quickly
4. **Readable**: Test code should be as clear as production code
5. **Maintainable**: Update tests when changing functionality

## Troubleshooting

### Common Issues

1. **Environment Variables**: Use `serial_test` for tests that modify env vars
2. **File Permissions**: Use `tempfile` for temporary file testing
3. **Network Tests**: Use `wiremock` instead of real network calls
4. **Async Tests**: Use `tokio::test` for async test functions

### Test Failures

1. Check test logs for specific error messages
2. Verify environment setup
3. Ensure no external dependencies
4. Check for resource leaks (files, network connections)

## Future Improvements

- [ ] Property-based testing with `proptest`
- [ ] Mutation testing with `cargo-mutagen`
- [ ] Performance regression testing
- [ ] End-to-end integration tests
- [ ] Visual test reporting dashboard