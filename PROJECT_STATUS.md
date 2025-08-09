# Bitcoin Transaction Broadcaster via Nym - Project Status

## Overview
This project implements a privacy-preserving Bitcoin transaction broadcaster that uses the Nym mixnet for anonymous transaction submission. It consists of:
- A Rust server that listens for transactions via Nym mixnet and submits them to mempool.space
- A Leptos web client for submitting transactions
- Common types shared between client and server

## Current Status

### Completed Tasks
1. ✅ Created project structure with workspace for client and server
2. ✅ Set up server with Nym service listener (using git version of nym-sdk)
3. ✅ Implemented transaction submission to mempool.space API
4. ✅ Created web server to display Nym address
5. ✅ Built Leptos client with transaction submission form
6. ✅ Created Justfile with clippy command
7. ✅ Switched from OpenSSL to rustls to avoid dependency issues

### Project Structure
```
broadnym/
├── Cargo.toml          # Workspace configuration
├── Justfile            # Build commands
├── README.md           # User documentation
├── PROJECT_STATUS.md   # This file
├── .gitignore
├── common/             # Shared types
│   ├── Cargo.toml
│   └── src/
│       └── lib.rs      # Network enum, TransactionRequest struct
├── server/             # Nym service + web server
│   ├── Cargo.toml
│   └── src/
│       └── main.rs     # Nym listener, transaction submission, web UI
└── client/             # Leptos web app
    ├── Cargo.toml
    ├── Trunk.toml      # Trunk build config
    ├── index.html
    ├── style/
    │   └── main.css
    └── src/
        └── main.rs     # Web UI for transaction submission
```

## TODO / Known Issues

### High Priority
1. **Nym SDK Integration**: The current server implementation needs to be updated to properly use the Nym SDK's mixnet client API. The `MixnetClient` API may have changed in the git version.
2. **WASM Support**: The client currently shows a placeholder message because Nym SDK doesn't have full WASM support. Need to either:
   - Wait for WASM support in nym-sdk
   - Use a desktop client approach
   - Implement a proxy pattern where the web client talks to a local service

### Medium Priority
3. **Error Handling**: Improve error handling and user feedback in both client and server
4. **Configuration**: Add configuration file support for server address, ports, etc.
5. **Logging**: Implement proper structured logging with different verbosity levels
6. **Testing**: Add unit and integration tests

### Low Priority
7. **UI Improvements**: Enhance the web UI with better styling and UX
8. **Transaction Status**: Add transaction status tracking and notifications
9. **Rate Limiting**: Implement rate limiting to prevent abuse
10. **Metrics**: Add prometheus metrics for monitoring

## Dependencies Note
- Using rustls instead of OpenSSL to avoid system dependency issues
- Using git version of nym-sdk as requested
- Leptos 0.6 (newer versions had feature compatibility issues)

## Development Setup

### Prerequisites
- Rust toolchain
- `trunk` for building the web client: `cargo install trunk`
- `just` for running commands: `cargo install just`

### Building
```bash
# Check compilation
just clippy

# Build server
just build-server

# Build client  
just build-client
```

### Running
```bash
# Terminal 1: Run server
just run-server

# Terminal 2: Run client dev server
just run-client
```

## Nix Shell Requirements
If adding to a Nix develop shell, you'll need:
- Rust toolchain
- pkg-config (if switching back to OpenSSL)
- openssl (if switching back to OpenSSL)
- trunk
- just

Current implementation uses rustls to avoid OpenSSL dependency.