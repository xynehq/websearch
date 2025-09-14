#!/bin/bash

# Comprehensive test script for search-sdk-rust

set -e

echo "🔍 Search SDK Rust - Comprehensive Test Suite"
echo "============================================="
echo

# Function to print step headers
print_step() {
    echo "📋 $1"
    echo "$(printf '%.0s-' {1..40})"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    echo "❌ Error: Please run this script from the project root directory"
    exit 1
fi

# Step 1: Code formatting check
print_step "Step 1: Checking code formatting"
if cargo fmt -- --check; then
    echo "✅ Code formatting is correct"
else
    echo "❌ Code formatting issues found. Run 'cargo fmt' to fix."
    exit 1
fi
echo

# Step 2: Linting with clippy
print_step "Step 2: Running clippy linter"
if cargo clippy -- -D warnings; then
    echo "✅ No clippy warnings found"
else
    echo "❌ Clippy warnings found. Please fix them."
    exit 1
fi
echo

# Step 3: Build check
print_step "Step 3: Building project"
if cargo build; then
    echo "✅ Project builds successfully"
else
    echo "❌ Build failed"
    exit 1
fi
echo

# Step 4: Unit tests
print_step "Step 4: Running unit tests"
if cargo test --lib; then
    echo "✅ All unit tests passed"
else
    echo "❌ Unit tests failed"
    exit 1
fi
echo

# Step 5: Integration tests
print_step "Step 5: Running integration tests"
if cargo test --test integration_tests; then
    echo "✅ All integration tests passed"
else
    echo "❌ Integration tests failed"
    exit 1
fi
echo

# Step 6: All tests (excluding doc tests that need real API keys)
print_step "Step 6: Running all tests (excluding doc tests)"
if cargo test --lib --tests; then
    echo "✅ All tests passed"
else
    echo "❌ Some tests failed"
    exit 1
fi
echo

# Step 7: Release build
print_step "Step 7: Building release version"
if cargo build --release; then
    echo "✅ Release build successful"
else
    echo "❌ Release build failed"
    exit 1
fi
echo

# Step 8: Example compilation
print_step "Step 8: Checking examples"
if cargo check --examples; then
    echo "✅ Examples compile successfully"
else
    echo "❌ Example compilation failed"
    exit 1
fi
echo

# Test summary
echo "🎉 All tests completed successfully!"
echo
echo "📊 Test Summary:"
echo "  ✅ Code formatting: PASSED"
echo "  ✅ Linting: PASSED"
echo "  ✅ Build: PASSED"
echo "  ✅ Unit tests: PASSED"
echo "  ✅ Integration tests: PASSED"
echo "  ✅ Release build: PASSED"
echo "  ✅ Examples: PASSED"
echo
echo "🚀 The search-sdk-rust is ready for use!"
echo
echo "📖 Quick usage:"
echo "  • Run tests: cargo test --lib --tests"
echo "  • Run example: cargo run --example basic_search"
echo "  • Build release: cargo build --release"
echo "  • Generate docs: cargo doc --open"
echo