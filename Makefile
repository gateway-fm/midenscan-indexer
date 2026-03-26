# Variables
IMAGE_NAME := gatewayfm/midenscan-indexer
RELEASE_TAG := latest

.PHONY: help build run release test check fmt lint clean docker-build docker-run

# Default target
all: build

# Display this help
help:
	@echo "Available targets:"
	@echo "  build         - Build the project in debug mode"
	@echo "  run           - Run the project in debug mode"
	@echo "  release       - Build the project in release mode"
	@echo "  test          - Run tests"
	@echo "  check         - Check the code for compilation errors"
	@echo "  fmt           - Format the code base"
	@echo "  lint          - Lint the code base using Clippy"
	@echo "  clean         - Clean build artifacts"
	@echo "  docker-build  - Build the Docker image"
	@echo "  docker-run    - Run the Docker container locally"

# Build the project in debug mode
build:
	cargo build

# Run the project in debug mode
run:
	cargo run

# Build the project in release mode
release:
	cargo build --release

# Run tests
test:
	cargo test

# Check the code for compilation errors
check:
	cargo check

# Format the code base
fmt:
	cargo fmt --all

# Lint the code base using Clippy
lint:
	cargo clippy --all-targets --all-features -- -D warnings

# Clean the build artifacts
clean:
	cargo clean

# Build the Docker image
docker-build:
	docker build -t $(IMAGE_NAME):$(RELEASE_TAG) .

# Run the Docker container locally
docker-run:
	docker run --rm -it $(IMAGE_NAME):$(RELEASE_TAG)
