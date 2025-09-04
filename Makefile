.PHONY: build run clean check test release install tokei

# Default target
all: help

# Set PATH to include cargo binaries
export PATH := $(HOME)/.cargo/bin:$(PATH)

# Rust toolchain to use (can be overridden with `make RUST_TOOLCHAIN=stable build`)
RUST_TOOLCHAIN ?= nightly

# Build the project
build:
	rustup run $(RUST_TOOLCHAIN) cargo build

# Build and run the project
run:
	rustup run $(RUST_TOOLCHAIN) cargo run

# Build release version
release:
	rustup run $(RUST_TOOLCHAIN) cargo build --release

# Run release version
run-release:
	rustup run $(RUST_TOOLCHAIN) cargo run --release

# Check code without building
check:
	rustup run $(RUST_TOOLCHAIN) cargo check

# Run tests
test:
	rustup run $(RUST_TOOLCHAIN) cargo test

# Clean build artifacts
clean:
	rustup run $(RUST_TOOLCHAIN) cargo clean

# Install dependencies
install:
	rustup run $(RUST_TOOLCHAIN) cargo fetch

# Format code
fmt:
	rustup run $(RUST_TOOLCHAIN) cargo fmt

# Run clippy linter
clippy:
	rustup run $(RUST_TOOLCHAIN) cargo clippy

# Full check (format, clippy, test, build)
full-check: fmt clippy test build

# AI simulation testing
sim-test: build
	rustup run $(RUST_TOOLCHAIN) cargo run -- --sim-ticks=500 --output-file=simulation_test.txt

sim-short: build
	rustup run $(RUST_TOOLCHAIN) cargo run -- --sim-ticks=100

sim-long: build
	rustup run $(RUST_TOOLCHAIN) cargo run -- --sim-ticks=1000 --output-file=long_simulation.txt

# Count lines of code
tokei:
	$(HOME)/.cargo/bin/tokei --exclude=target --files --sort lines

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
	@echo "  tokei      - Count lines of code"
	@echo "  help       - Show this help"
	@echo ""
	@echo "Variables:"
	@echo "  RUST_TOOLCHAIN - Rust toolchain to use (default: nightly)"
	@echo "                   Example: make RUST_TOOLCHAIN=stable build"
