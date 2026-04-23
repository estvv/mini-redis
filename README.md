# rust-kvs-multithreaded

A lightweight, thread-safe key-value store built in Rust with async I/O and pub/sub messaging.

[![License](https://img.shields.io/badge/license-MIT%2FApache--2.0-blue)](LICENSE)

## Architecture

```
┌───────────────────────────────────┐
│  Client (telnet / netcat)          │
│  Publisher / Subscriber            │
└──────────┬────────────────────────┘
           │ TCP (line-based protocol)
           ▼
┌───────────────────────────────────┐
│  main.rs                           │
│  Connection Handler                │
│  Client ID Assignment              │
└──────────┬────────────────────────┘
           │ Arc<Mutex<...>>
           ▼
┌───────────────────────────────────┐
│  Dispatcher                        │
│  Request Router                    │
│  Client Subscription Tracking      │
└──────────┬────────────────────────┘
           │
    ┌──────┴──────┐
    ▼             ▼
┌─────────┐  ┌─────────────────┐
│ Stock   │  │ ChannelManager  │
│ HashMap │  │ Broadcast       │
│ Expiry  │  │ Channels        │
└─────────┘  └─────────────────┘
```

## Modules

| Module | Description |
|--------|-------------|
| `main.rs` | TCP server, connection handling, client ID assignment, graceful shutdown |
| `dispatcher.rs` | Request routing, command execution, subscription tracking |
| `request.rs` | Request parsing (GET, SET, DEL, SAVE, LOAD, DROP, PUB, SUB, UNSUB) |
| `stock.rs` | Key-value storage with expiration support and JSON persistence |
| `channel_manager.rs` | Pub/sub channel management with broadcast channels |
| `returns.rs` | Return types (Ok, Err, NotFound, Subscribe, Unsubscribe) |

## Features

- **Thread-safe key-value store** — `Arc<Mutex<T>>` pattern for safe concurrent access across multiple clients
- **Key expiration** — SET with EXP parameter for TTL (lazy expiration on read)
- **Pub/Sub messaging** — broadcast channels for real-time message distribution to subscribers
- **Client tracking** — unique IDs for subscription management and cleanup
- **JSON persistence** — SAVE/LOAD commands for state snapshots
- **Graceful shutdown** — Ctrl+C handling with clean termination
- **Async I/O** — Tokio runtime with non-blocking TCP listener

## Quick Start

```bash
# Build and run
cargo run

# The server starts on 127.0.0.1:6379
```

In another terminal:

```bash
nc localhost 6379
# or
telnet localhost 6379
```

## Commands

| Command | Description | Example |
|---------|-------------|---------|
| `GET <key>` | Retrieve a value by key | `GET mykey` |
| `SET <key> <value> [EXP <ms>]` | Set a key-value pair with optional expiration | `SET mykey hello EXP 5000` |
| `DEL <key>` | Delete a key | `DEL mykey` |
| `SAVE <file.json>` | Save state to `./data/<file.json>` | `SAVE dump.json` |
| `LOAD <file.json>` | Load state from `./data/<file.json>` | `LOAD dump.json` |
| `DROP` | Clear all keys | `DROP` |
| `PUB <channel> <message>` | Publish a message to a channel | `PUB news Hello World` |
| `SUB <channel>` | Subscribe to a channel | `SUB news` |
| `UNSUB <channel>` | Unsubscribe from a channel | `UNSUB news` |

**Expiration**: TTL in milliseconds. Expired keys are removed lazily on `GET`.

**Pub/Sub**: Each client can subscribe to one channel at a time. Messages are broadcast to all subscribers.

## Examples

### Basic Key-Value Operations

```
SET username alice
OK
GET username
alice
SET temp data EXP 3000
OK
GET temp
data
# (after 3 seconds)
GET temp
Key 'temp' not found
SAVE mydata.json
OK
LOAD mydata.json
OK
```

### Pub/Sub Messaging

**Terminal 1 (Subscriber):**
```
SUB news
Subscribed
MESSAGE news Breaking: Rust 2.0 released!
MESSAGE news Update: Performance improvements
UNSUB news
Unsubscribed
```

**Terminal 2 (Publisher):**
```
PUB news Breaking: Rust 2.0 released!
Published to 1 subscriber(s)
PUB news Update: Performance improvements
Published to 1 subscriber(s)
```

### Error Handling

```
UNSUB news
Error: Not subscribed to any channel

SUB news
Subscribed
UNSUB sports
Error: Not subscribed to channel 'sports'

UNSUB nonexistent
Error: Channel 'nonexistent' does not exist
```

## Data Model

```rust
struct Data {
    value: String,
    expiration: Option<u64>,  // Unix timestamp in milliseconds
}

struct Stock {
    map: HashMap<String, Data>,
}
```

## Implementation Details

### Concurrency Model

- **Single dispatcher lock** — All operations share one `Arc<Mutex<Dispatcher>>`
- **Tokio tasks** — Each client spawns an async task
- **Client IDs** — `AtomicU64` counter for unique IDs
- **Broadcast channels** — `tokio::sync::broadcast` for pub/sub (capacity: 16)

### Expiration Strategy

- **Lazy expiration** — Keys are checked and removed on `GET`
- **No background cleanup** — Expired keys remain in memory until accessed

### Message Flow

```
Publisher                  Dispatcher                 Subscriber
    │                          │                           │
    │  PUB channel msg         │                           │
    │ ───────────────────────► │                           │
    │                          │  broadcast::send()        │
    │                          │ ─────────────────────────►│
    │                          │                           │  MESSAGE ...
```

## Configuration

| Parameter | Default | Description |
|-----------|---------|-------------|
| Bind address | `127.0.0.1:6379` | Hardcoded in `main.rs` |
| Broadcast capacity | 16 | Buffer size per channel |

## Dependencies

| Crate | Usage |
|-------|-------|
| `tokio` | Async runtime, TCP, sync primitives |
| `serde` | Serialization |
| `serde_json` | JSON persistence |

## Project Structure

```
src/
├── main.rs            # Server entry point, connection handling
├── dispatcher.rs      # Request routing, command execution
├── request.rs         # Request parsing
├── channel_manager.rs # Pub/sub channel management
├── stock.rs           # Key-value storage with expiration
└── returns.rs         # Return types
```

## Roadmap

See [FEATURES.md](FEATURES.md) for planned and completed features.

## License

Licensed under either of [MIT](LICENSE-MIT) or [Apache-2.0](LICENSE-APACHE) at your option.