.PHONY: build run clean check test release install

# Default target
all: help

# Build the project
build:
	cargo build

# Build and run the project
run:
	cargo run

# Build release version
release:
	cargo build --release

# Run release version
run-release:
	cargo run --release

# Check code without building
check:
	cargo check

# Run tests
test:
	cargo test

# Clean build artifacts
clean:
	cargo clean

# Install dependencies
install:
	cargo fetch

# Format code
fmt:
	cargo fmt

# Run clippy linter
clippy:
	cargo clippy

# Full check (format, clippy, test, build)
full-check: fmt clippy test build

# AI simulation testing
sim-test: build
	cargo run -- --sim-ticks=500 --output-file=simulation_test.txt

sim-short: build
	cargo run -- --sim-ticks=100

sim-long: build
	cargo run -- --sim-ticks=1000 --output-file=long_simulation.txt

# Help
help:
	@echo "Available targets:"
	@echo "  build      - Build the project"
	@echo "  run        - Build and run the project"
	@echo "  release    - Build release version"
	@echo "  run-release- Run release version"
	@echo "  check      - Check code without building"
	@echo "  test       - Run tests"
	@echo "  clean      - Clean build artifacts"
	@echo "  install    - Install dependencies"
	@echo "  fmt        - Format code"
	@echo "  clippy     - Run clippy linter"
	@echo "  full-check - Run fmt, clippy, test, and build"
	@echo "  sim-test   - Run 500-tick simulation and save to file"
	@echo "  sim-short  - Run 100-tick simulation to console"
	@echo "  sim-long   - Run 1000-tick simulation and save to file"
	@echo "  help       - Show this help"
