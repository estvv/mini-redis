# Multi-threaded Key-Value Store

A multi-threaded key-value store server implementation in Rust with persistence and TTL support.

## Overview

This project implements a TCP-based key-value store server with thread-safe concurrent access. Built in Rust, it demonstrates concurrent programming concepts including shared state management with `Arc<Mutex<T>>`, non-blocking I/O, and graceful shutdown handling.

## Features

- **Multi-threaded Architecture**: Each client connection is handled in a separate thread, allowing concurrent client access
- **Thread-safe State**: Uses `Arc<Mutex<Dispatcher>>` pattern to safely share the key-value store across threads
- **TTL Support**: Keys can be set with optional expiration times (in milliseconds)
- **Persistence**: Save and load the key-value store state to/from JSON files
- **Graceful Shutdown**: Handles Ctrl+C (SIGINT) for clean server termination

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `GET <key>` | Retrieve a value by key | `GET mykey` |
| `SET <key> <value> [EXP <ms>]` | Set a key-value pair with optional expiration | `SET mykey hello` or `SET mykey hello EXP 5000` |
| `DEL <key>` | Delete a key | `DEL mykey` |
| `SAVE <filename.json>` | Save current state to `./data/<filename.json>` | `SAVE dump.json` |
| `LOAD <filename.json>` | Load state from `./data/<filename.json>` | `LOAD dump.json` |
| `DROP` | Clear all keys from the store | `DROP` |

Expirations are specified in milliseconds. Keys with expired TTL are automatically removed when accessed via `GET`.

## Project Structure

```
src/
├── main.rs        # Server entry point, connection handling, request processing
├── dispatcher.rs  # Request routing and command execution
├── request.rs     # Request parsing (GET, SET, DEL, SAVE, LOAD, DROP)
├── stock.rs       # Key-value storage with expiration support
└── returns.rs     # Return types (Ok, Err, NotFound)
```

## Implementation Details

### Architecture

- **Server**: Non-blocking TCP listener on `127.0.0.1:6379` with read timeouts
- **Concurrency**: Uses `Arc<AtomicBool>` for shutdown signal across threads
- **State Management**: `Dispatcher` wraps `Stock` (the data store) in `Arc<Mutex<...>>` for safe concurrent access
- **Client Handler**: Each connection spawns a thread that reads line-by-line and dispatches commands

### Data Storage

```rust
struct Data {
    value: String,
    expiration: Option<u64>,  // Unix timestamp in milliseconds
}

struct Stock {
    map: HashMap<String, Data>,
}
```

Data is persisted as JSON using `serde_json`.

### Execution Flow

1. Server binds to port 6379 and listens for connections
2. On new connection, spawns a thread for the client
3. Client thread reads commands line-by-line
4. Commands are parsed into `Request` enum variants
5. `Dispatcher` routes to appropriate handler
6. `Stock` performs the operation (with mutex lock held)
7. Response is written back to client

## Dependencies

- `serde` & `serde_json` - JSON serialization for persistence
- `ctrlc` - Graceful shutdown handling

## Running

```bash
# Build and run
cargo run

# The server starts on 127.0.0.1:6379
```

In another terminal, connect with:

```bash
nc localhost 6379
# or
telnet localhost 6379
```

Example session:
```
SET username alice
OK
GET username
alice
SET temp data EXP 3000
OK
GET temp
data
# After 3 seconds:
GET temp
Key 'temp' not found
SAVE mydata.json
OK
```

## Graceful Shutdown

Press `Ctrl+C` to gracefully shutdown the server. The server will:
1. Stop accepting new connections
2. Signal all client threads to shut down
3. Exit cleanly

## Project History

This project was developed incrementally:

1. Basic client-server with `GET`, `SET`, `DEL` commands
2. Multi-threading with mutex for concurrent client access
3. `SAVE`, `LOAD`, and `DROP` commands for persistence
4. Graceful shutdown and dynamic reading with non-blocking I/O