.PHONY: build run clean check test release install

# Default target
all: build

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
	@echo "  help       - Show this help"