# Justfile for search-sdk-rust
# A modern alternative to Makefile using the `just` command runner

# Default recipe to display help
default:
    @just --list

# Build the project
build:
    cargo build

# Build in release mode
build-release:
    cargo build --release

# Run all tests
test:
    cargo test

# Run only unit tests (lib tests)
test-unit:
    cargo test --lib

# Run only integration tests
test-integration:
    cargo test --test integration_tests

# Run all tests with verbose output and debug logging
test-all:
    RUST_LOG=debug cargo test --all-features --verbose

# Run tests with coverage (requires cargo-tarpaulin)
test-coverage:
    cargo tarpaulin --out Html --output-dir coverage

# Format code
fmt:
    cargo fmt

# Check formatting without applying changes
fmt-check:
    cargo fmt -- --check

# Run clippy linter
lint:
    cargo clippy -- -D warnings

# Fix linting issues automatically
lint-fix:
    cargo clippy --fix --allow-dirty

# Check code without building
check:
    cargo check

# Check all targets and features
check-all:
    cargo check --all-targets --all-features

# Generate and open documentation
doc:
    cargo doc --no-deps --open

# Generate documentation including private items
doc-private:
    cargo doc --no-deps --document-private-items --open

# Run benchmarks
bench:
    cargo bench

# Install the binary
install:
    cargo install --path .

# Run the main binary
run *args:
    cargo run {{args}}

# Run in release mode
run-release *args:
    cargo run --release {{args}}

# Clean build artifacts
clean:
    cargo clean

# Setup development environment
dev-setup:
    rustup component add rustfmt clippy
    @echo "Development tools installed"

# Run all development checks
dev-check:
    just fmt-check
    just lint
    just check
    just test
    @echo "All development checks passed!"

# Run CI/CD pipeline
ci:
    just fmt-check
    just lint
    just check-all
    just test-all
    @echo "CI pipeline completed successfully"

# Run performance tests
perf-test:
    @echo "Running performance tests..."
    cargo test --release --test integration_tests test_large_number_of_results -- --nocapture
    cargo test --release --test integration_tests test_memory_usage_with_large_content -- --nocapture

# Security audit
audit:
    cargo audit

# Update dependencies
update:
    cargo update

# Show outdated dependencies
outdated:
    cargo outdated

# Watch for changes and run tests
watch-test:
    cargo watch -x test

# Watch for changes and run specific test
watch-test-name name:
    cargo watch -x "test {{name}}"

# Quick development cycle: format, check, test
quick:
    just fmt
    just check
    just test-unit

# Create a new example
new-example name:
    @echo 'use search_sdk::*;\n\nfn main() {\n    println!("Example: {{name}}");\n}' > examples/{{name}}.rs
    @echo "Created examples/{{name}}.rs"

# Run a specific example
example name:
    cargo run --example {{name}}

# Generate test coverage report
coverage:
    #!/usr/bin/env bash
    if command -v cargo-tarpaulin &> /dev/null; then
        cargo tarpaulin --out Html --output-dir coverage
        @echo "Coverage report generated in coverage/"
    else
        @echo "cargo-tarpaulin not found. Install with: cargo install cargo-tarpaulin"
    fi

# Profile the application
profile:
    cargo build --release
    @echo "Run with profiler: perf record -g target/release/search-sdk-rust"

# Memory check with valgrind (Linux only)
valgrind:
    cargo build
    valgrind --tool=memcheck --leak-check=full target/debug/search-sdk-rust