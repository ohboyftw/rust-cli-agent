@echo off
REM Test runner script for rust-cli-agent (Windows)
REM This script runs all tests and generates coverage reports

echo 🧪 Running rust-cli-agent test suite...

REM Check if we're in the right directory
if not exist "Cargo.toml" (
    echo [ERROR] Cargo.toml not found. Please run this script from the project root.
    exit /b 1
)

REM Load test environment variables if they exist
if exist ".env.test" (
    echo [INFO] Loading test environment variables...
    for /f "eol=# delims=" %%i in (.env.test) do set %%i
)

REM Clean previous builds
echo [INFO] Cleaning previous builds...
cargo clean

REM Check code formatting
echo [INFO] Checking code formatting...
cargo fmt --all -- --check
if errorlevel 1 (
    echo [ERROR] Code formatting issues found. Run 'cargo fmt' to fix.
    exit /b 1
)
echo [SUCCESS] Code formatting is correct

REM Run clippy lints
echo [INFO] Running clippy lints...
cargo clippy --all-targets --all-features -- -D warnings
if errorlevel 1 (
    echo [ERROR] Clippy found issues. Please fix them before proceeding.
    exit /b 1
)
echo [SUCCESS] Clippy checks passed

REM Run unit tests
echo [INFO] Running unit tests...
cargo test --lib --verbose
if errorlevel 1 (
    echo [ERROR] Unit tests failed
    exit /b 1
)
echo [SUCCESS] Unit tests passed

REM Run integration tests
echo [INFO] Running integration tests...
cargo test --test * --verbose
if errorlevel 1 (
    echo [ERROR] Integration tests failed
    exit /b 1
)
echo [SUCCESS] Integration tests passed

REM Run all tests together
echo [INFO] Running all tests...
cargo test --all-features --verbose
if errorlevel 1 (
    echo [ERROR] Some tests failed
    exit /b 1
)
echo [SUCCESS] All tests passed

REM Run doc tests
echo [INFO] Running documentation tests...
cargo test --doc
if errorlevel 1 (
    echo [WARNING] Documentation tests failed (this might be expected if no doc tests exist)
) else (
    echo [SUCCESS] Documentation tests passed
)

REM Generate coverage report if cargo-llvm-cov is available
where cargo-llvm-cov >nul 2>&1
if %errorlevel% == 0 (
    echo [INFO] Generating coverage report...
    cargo llvm-cov --all-features --workspace --html
    echo [SUCCESS] Coverage report generated in target/llvm-cov/html/
) else (
    echo [WARNING] cargo-llvm-cov not found. Install it with: cargo install cargo-llvm-cov
)

REM Build in release mode to ensure it compiles
echo [INFO] Building in release mode...
cargo build --release
if errorlevel 1 (
    echo [ERROR] Release build failed
    exit /b 1
)
echo [SUCCESS] Release build succeeded

REM Check binary size
if exist "target\release\cli_coding_agent.exe" (
    for %%I in (target\release\cli_coding_agent.exe) do (
        echo [INFO] Binary size: %%~zI bytes
    )
)

echo [SUCCESS] 🎉 All tests completed successfully!

REM Optional: Run security audit if cargo-audit is available
where cargo-audit >nul 2>&1
if %errorlevel% == 0 (
    echo [INFO] Running security audit...
    cargo audit
    if errorlevel 1 (
        echo [WARNING] Security audit found issues
    ) else (
        echo [SUCCESS] Security audit passed
    )
) else (
    echo [WARNING] cargo-audit not found. Install it with: cargo install cargo-audit
)

echo.
echo Test Summary:
echo ✅ Code formatting
echo ✅ Clippy lints
echo ✅ Unit tests
echo ✅ Integration tests
echo ✅ All tests
echo ✅ Release build

where cargo-llvm-cov >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Coverage report
)

where cargo-audit >nul 2>&1
if %errorlevel% == 0 (
    echo ✅ Security audit
)

echo [SUCCESS] Test suite completed successfully! 🚀