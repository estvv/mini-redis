# rust-mini-redis

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
| `request.rs` | Request parsing (GET, SET, DEL, INCR, DECR, SAVE, LOAD, DROP, PUB, SUB, UNSUB, TTL) |
| `stock.rs` | Key-value storage with expiration support and JSON persistence |
| `channel_manager.rs` | Pub/sub channel management with broadcast channels |
| `returns.rs` | Return types (Ok, Err, NotFound, Subscribe, Unsubscribe) |

## Features

### Core Logic & Data Operations
- **GET/SET/DEL** - Basic key-value store operations
- **SET EXP** - Key expiration with TTL parameter
- **TTL** - Get remaining time to live for a key
- **INCR/DECR** - Atomic increment and decrement for counters
- **DROP** - Clear all data from the store
- **ASYNC SERVER** - Multi-threaded async server with Tokio runtime

### Persistence & Recovery
- **SAVE/LOAD** - JSON-based persistence to file
- **GRACEFUL SHUTDOWN** - Signal handling for clean Ctrl+C termination

### Protocol & Networking
- **PUB/SUB/UNSUB** - Pub/Sub messaging with broadcast channels
- **CLIENT TRACKING** - Connection tracking and cleanup

See [FEATURES.md](FEATURES.md) for the full roadmap with planned features.

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
| `TTL <key>` | Get remaining time to live for a key (in ms) | `TTL mykey` |
| `INCR <key>` | Increment value by 1 (creates key with value 1 if not exists) | `INCR counter` |
| `DECR <key>` | Decrement value by 1 (creates key with value -1 if not exists) | `DECR counter` |
| `SAVE <file.json>` | Save state to `./data/<file.json>` | `SAVE dump.json` |
| `LOAD <file.json>` | Load state from `./data/<file.json>` | `LOAD dump.json` |
| `DROP` | Clear all keys | `DROP` |
| `PUB <channel> <message>` | Publish a message to a channel | `PUB news Hello World` |
| `SUB <channel>` | Subscribe to a channel | `SUB news` |
| `UNSUB <channel>` | Unsubscribe from a channel | `UNSUB news` |

**Expiration**: TTL in milliseconds. Expired keys are removed lazily on `GET` or `TTL`.

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
TTL temp
3000
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

### Counter Operations

```
INCR views
1
INCR views
2
INCR views
3
DECR views
2
SET counter 10
OK
INCR counter
11
DECR counter
10
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

- **Single dispatcher lock** - All operations share one `Arc<Mutex<Dispatcher>>`
- **Tokio tasks** - Each client spawns an async task
- **Client IDs** - `AtomicU64` counter for unique IDs
- **Broadcast channels** - `tokio::sync::broadcast` for pub/sub (capacity: 16)

### Expiration Strategy

- **Lazy expiration** - Keys are checked and removed on `GET`
- **No background cleanup** - Expired keys remain in memory until accessed

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

Licensed under [MIT](LICENSE-MIT)
