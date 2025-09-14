# Makefile for search-sdk-rust

.PHONY: help build test test-unit test-integration test-all clean fmt lint check doc bench install run examples

# Default target
help:
	@echo "Available targets:"
	@echo "  build           - Build the project"
	@echo "  test            - Run all tests"
	@echo "  test-unit       - Run unit tests only"
	@echo "  test-integration- Run integration tests only"
	@echo "  test-all        - Run all tests with coverage"
	@echo "  clean           - Clean build artifacts"
	@echo "  fmt             - Format code"
	@echo "  lint            - Run clippy linter"
	@echo "  check           - Check code without building"
	@echo "  doc             - Generate documentation"
	@echo "  bench           - Run benchmarks"
	@echo "  install         - Install the binary"
	@echo "  run             - Run the main binary"
	@echo "  examples        - Run example programs"

# Build targets
build:
	cargo build

build-release:
	cargo build --release

# Test targets
test:
	cargo test

test-unit:
	cargo test --lib

test-integration:
	cargo test --test integration_tests

test-all:
	RUST_LOG=debug cargo test --all-features --verbose

test-with-coverage:
	cargo test --all-features
	@echo "For detailed coverage, install cargo-tarpaulin: cargo install cargo-tarpaulin"
	@echo "Then run: cargo tarpaulin --out Html --output-dir coverage"

# Code quality
fmt:
	cargo fmt

fmt-check:
	cargo fmt -- --check

lint:
	cargo clippy -- -D warnings

lint-fix:
	cargo clippy --fix --allow-dirty

check:
	cargo check

check-all:
	cargo check --all-targets --all-features

# Documentation
doc:
	cargo doc --no-deps --open

doc-private:
	cargo doc --no-deps --document-private-items --open

# Benchmarks
bench:
	cargo bench

# Install and run
install:
	cargo install --path .

run:
	cargo run

run-release:
	cargo run --release

# Examples
examples:
	@echo "Running basic search example..."
	cargo run --example basic_search || echo "Create examples/basic_search.rs first"

# Clean up
clean:
	cargo clean

# Development workflow
dev-setup:
	rustup component add rustfmt clippy
	@echo "Development tools installed"

dev-check: fmt-check lint check test
	@echo "All development checks passed!"

# CI/CD targets
ci-test: fmt-check lint check test-all
	@echo "CI pipeline completed successfully"

# Performance testing
perf-test:
	@echo "Running performance tests..."
	cargo test --release --test integration_tests test_large_number_of_results -- --nocapture
	cargo test --release --test integration_tests test_memory_usage_with_large_content -- --nocapture

# Security audit
audit:
	cargo audit || echo "Install cargo-audit: cargo install cargo-audit"

# Update dependencies
update:
	cargo update

# Show outdated dependencies
outdated:
	cargo outdated || echo "Install cargo-outdated: cargo install cargo-outdated"