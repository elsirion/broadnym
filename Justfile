# Run clippy on all workspace members
clippy:
    cargo clippy -p broadnym_server -- -D warnings
    cargo clippy -p broadnym_client --target wasm32-unknown-unknown -- -D warnings

# Build the server
build-server:
    cargo build --package server --release

# Build the client
build-client:
    cargo build --package client --release

# Build all packages
build: build-server build-client

# Run the server
run-server:
    cargo run --package server

# Run the client dev server
run-client:
    cd client && trunk serve

# Check all packages
check:
    cargo check --workspace --all-targets

# Format code
fmt:
    cargo fmt --all

# Run tests
test:
    cargo test --workspace

# Clean build artifacts
clean:
    cargo clean

# Run both client and server concurrently
dev:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Starting server and client..."
    cargo run --package broadnym_server &
    SERVER_PID=$!
    cd client && trunk serve &
    CLIENT_PID=$!
    trap "kill $SERVER_PID $CLIENT_PID 2>/dev/null || true" EXIT
    wait
