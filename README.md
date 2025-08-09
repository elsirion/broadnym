# Bitcoin Transaction Broadcaster via Nym

A privacy-preserving Bitcoin transaction broadcaster that uses the Nym mixnet for anonymous transaction submission.

## Architecture

- **Server**: Rust service that listens for transactions via Nym mixnet and submits them to mempool.space
- **Client**: Leptos web application for submitting transactions
- **Common**: Shared types between client and server

## Building

```bash
# Install dependencies
cargo install trunk just

# Run clippy checks
just clippy

# Build server
just build-server

# Build client
just build-client
```

## Running

### Server
```bash
just run-server
# Or with custom port:
cargo run --package server -- --port 3000
```

The server will:
1. Start a Nym mixnet client and display its address
2. Run a web server showing the Nym address
3. Listen for transaction requests via Nym
4. Submit received transactions to mempool.space

### Client
```bash
just run-client
```

Then open http://localhost:8080 in your browser.

## Usage

1. Start the server and note the Nym address displayed
2. Open the client web app
3. Enter the server's Nym address
4. Paste your raw Bitcoin transaction hex
5. Select the network (mainnet or testnet)
6. Click "Submit Transaction"

The transaction will be sent anonymously through the Nym mixnet to the server, which will then submit it to the Bitcoin network via mempool.space.

## Security Notes

- All communication between client and server goes through the Nym mixnet for privacy
- The server does not log transaction details
- No direct connection is established between client and server