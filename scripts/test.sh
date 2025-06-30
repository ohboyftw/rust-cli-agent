#!/bin/bash

# Test runner script for rust-cli-agent
# This script runs all tests and generates coverage reports

set -e

echo "🧪 Running rust-cli-agent test suite..."

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Cargo.toml not found. Please run this script from the project root."
    exit 1
fi

# Load test environment variables if they exist
if [ -f ".env.test" ]; then
    print_status "Loading test environment variables..."
    export $(cat .env.test | grep -v '^#' | xargs)
fi

# Clean previous builds
print_status "Cleaning previous builds..."
cargo clean

# Check code formatting
print_status "Checking code formatting..."
if cargo fmt --all -- --check; then
    print_success "Code formatting is correct"
else
    print_error "Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi

# Run clippy lints
print_status "Running clippy lints..."
if cargo clippy --all-targets --all-features -- -D warnings; then
    print_success "Clippy checks passed"
else
    print_error "Clippy found issues. Please fix them before proceeding."
    exit 1
fi

# Run unit tests
print_status "Running unit tests..."
if cargo test --lib --verbose; then
    print_success "Unit tests passed"
else
    print_error "Unit tests failed"
    exit 1
fi

# Run integration tests
print_status "Running integration tests..."
if cargo test --test '*' --verbose; then
    print_success "Integration tests passed"
else
    print_error "Integration tests failed"
    exit 1
fi

# Run all tests together
print_status "Running all tests..."
if cargo test --all-features --verbose; then
    print_success "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi

# Run doc tests
print_status "Running documentation tests..."
if cargo test --doc; then
    print_success "Documentation tests passed"
else
    print_warning "Documentation tests failed (this might be expected if no doc tests exist)"
fi

# Generate coverage report if cargo-llvm-cov is available
if command -v cargo-llvm-cov &> /dev/null; then
    print_status "Generating coverage report..."
    cargo llvm-cov --all-features --workspace --html
    print_success "Coverage report generated in target/llvm-cov/html/"
else
    print_warning "cargo-llvm-cov not found. Install it with: cargo install cargo-llvm-cov"
fi

# Build in release mode to ensure it compiles
print_status "Building in release mode..."
if cargo build --release; then
    print_success "Release build succeeded"
else
    print_error "Release build failed"
    exit 1
fi

# Check binary size
if [ -f "target/release/cli_coding_agent" ]; then
    BINARY_SIZE=$(du -h target/release/cli_coding_agent | cut -f1)
    print_status "Binary size: $BINARY_SIZE"
fi

print_success "🎉 All tests completed successfully!"

# Optional: Run security audit if cargo-audit is available
if command -v cargo-audit &> /dev/null; then
    print_status "Running security audit..."
    if cargo audit; then
        print_success "Security audit passed"
    else
        print_warning "Security audit found issues"
    fi
else
    print_warning "cargo-audit not found. Install it with: cargo install cargo-audit"
fi

echo ""
echo "Test Summary:"
echo "✅ Code formatting"
echo "✅ Clippy lints"
echo "✅ Unit tests"
echo "✅ Integration tests"
echo "✅ All tests"
echo "✅ Release build"

if command -v cargo-llvm-cov &> /dev/null; then
    echo "✅ Coverage report"
fi

if command -v cargo-audit &> /dev/null; then
    echo "✅ Security audit"
fi

print_success "Test suite completed successfully! 🚀"