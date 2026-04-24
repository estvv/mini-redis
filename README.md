# rust-mini-redis

A lightweight, thread-safe key-value store built in Rust with async I/O and pub/sub messaging.

[![License](https://img.shields.io/badge/license-MIT-blue)](LICENSE)

## Architecture

```
┌───────────────────────────────────┐
│  Client (telnet / netcat)         │
│  Publisher / Subscriber           │
└──────────┬────────────────────────┘
           │ TCP (line-based protocol)
           ▼
┌───────────────────────────────────┐
│  main.rs                          │
│  Connection Handler               │
│  Client ID Assignment             │
└──────────┬────────────────────────┘
           │ Arc<Db>
           ▼
┌───────────────────────────────────┐
│  Db                               │
│  Arc<Mutex<DbInner>>              │
│  ├─ Stock (HashMap)               │
│  ├─ ChannelManager                │
│  └─ Client Subscriptions          │
└──────────┬────────────────────────┘
           │
           ▼
┌───────────────────────────────────┐
│  Command Trait                    │
│  ├─ Get, Set, Del                 │
│  ├─ Incr, Decr                    │
│  ├─ Save, Load, Drop              │
│  ├─ Publish, Subscribe            │
│  ├─ Ttl, Exists                   │
│  └─ Each implements execute()     │
└───────────────────────────────────┘
```

## Architecture: TUI and TCP Server

**Single Process Architecture:**

When running with `--ui`, there is ONE process that runs BOTH the TCP server AND the TUI dashboard:

```
┌─────────────────────────────────────────────────────────────┐
│              Single Process (cargo run -- --ui)             │
│                                                             │
│  ┌──────────────────┐         ┌──────────────────────┐      │
│  │   TCP Server     │         │    TUI Dashboard     │      │
│  │  (port 6379)     │         │   (interactive UI)   │      │
│  │                  │         │                      │      │
│  │  • Netcat/telnet │         │  • Key browser       │      │
│  │  • redis-cli     │         │  • Command input     │      │
│  │  • Other clients │         │  • History view      │      │
│  └────────┬─────────┘         └──────────┬───────────┘      │
│           │                              │                  │
│           │         Arc<Db>              │                  │
│           │     (shared reference)       │                  │
│           │                              │                  │
│           └──────────────┬───────────────┘                  │
│                          ▼                                  │
│                 ┌─────────────────┐                         │
│                 │   Arc<Mutex<    │                         │
│                 │     DbInner     │                         │
│                 │   >             │                         │
│                 │                 │                         │
│                 │  ├─ Stock       │                         │
│                 │  ├─ ChannelMgr  │                         │
│                 │  └─ ClientSubs  │                         │
│                 └─────────────────┘                         │
│                        In-Memory Database                   │
└─────────────────────────────────────────────────────────────┘
                          │
                          │ TCP Protocol (line-based)
                          ▼
              ┌───────────────────────────┐
              │   External Clients        │
              │  (netcat, telnet, etc.)   │
              └───────────────────────────┘
```

**Key Points:**
- **The TUI is NOT a client** - It accesses the database directly in memory
- **The TUI does NOT connect via TCP** - It shares the same `Arc<Db>` reference
- **External clients connect via TCP** - On port 6379 (same as Redis default)
- **Both share the same data** - Changes from TUI or external clients are instantly visible to both

**Two Modes:**
1. **Server-only mode** (`cargo run`): Just the TCP server, no TUI
2. **Combined mode** (`cargo run -- --ui`): TCP server + TUI dashboard in one process

## Modules

| Module | Description |
|--------|-------------|
| `main.rs` | TCP server, connection handling, client ID assignment, graceful shutdown |
| `db.rs` | Database layer with interior mutability pattern (Arc<Mutex<DbInner>>) |
| `command.rs` | Command trait definition and re-exports |
| `commands/` | Individual command implementations (Get, Set, Del, etc.) |
| `request.rs` | Request parsing (GET, SET, DEL, EXISTS, INCR, DECR, SAVE, LOAD, DROP, PUB, SUB, UNSUB, TTL) |
| `stock.rs` | Key-value storage with expiration support and JSON persistence |
| `channel_manager.rs` | Pub/sub channel management with broadcast channels |
| `returns.rs` | Return types (Ok, Err, NotFound, Subscribe, Unsubscribe) |

## Features

### Core Logic & Data Operations
- **GET/SET/DEL** - Basic key-value store operations
- **EXISTS** - Check if multiple keys exist
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
# Build and run (TCP server mode)
cargo run

# The server starts on 127.0.0.1:6379
# Run with TUI dashboard:
cargo run -- --ui
```

### Two Modes:

**1. TCP Server Mode (Default)**
```bash
cargo run
# Starts TCP server on port 6379
# Use with telnet/netcat
```

**2. TUI Dashboard Mode**
```bash
cargo run -- --ui
# Starts TCP server + Interactive dashboard
# Perfect for testing and monitoring
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
| `SET <key> <value> [EXP <sec>]` | Set a key-value pair with optional expiration | `SET mykey hello EXP 10` |
| `DEL <key>` | Delete a key | `DEL mykey` |
| `EXISTS <key> [<key> ...]` | Check if one or more keys exist | `EXISTS key1 key2 key3` |
| `TTL <key>` | Get remaining time to live for a key (in ms) | `TTL mykey` |
| `INCR <key>` | Increment value by 1 (creates key with value 1 if not exists) | `INCR counter` |
| `DECR <key>` | Decrement value by 1 (creates key with value -1 if not exists) | `DECR counter` |
| `SAVE <file.json>` | Save state to `./data/<file.json>` | `SAVE dump.json` |
| `LOAD <file.json>` | Load state from `./data/<file.json>` | `LOAD dump.json` |
| `DROP` | Clear all keys | `DROP` |
| `PUB <channel> <message>` | Publish a message to a channel | `PUB news Hello World` |
| `SUB <channel>` | Subscribe to a channel | `SUB news` |
| `UNSUB <channel>` | Unsubscribe from a channel | `UNSUB news` |

**Expiration**: TTL in seconds. Expired keys are removed lazily on `GET`, `TTL`, or `EXISTS`.

**EXISTS**: Returns existence status for each key in format `key -> true/false`.

**Pub/Sub**: Each client can subscribe to one channel at a time. Messages are broadcast to all subscribers.

**Interactive Dashboard**: Run with `--ui` flag for a beautiful TUI interface with real-time key monitoring, command history, and live updates.

## Interactive Dashboard (TUI)

```bash
# Start with interactive dashboard
cargo run -- --ui
```

### Dashboard Features

**Tab 1: Key-Value Store**
- Real-time list of all keys and values
- TTL countdown timers for expiring keys
- Navigate with arrow keys

**Tab 2: Command History**
- See all executed commands
- Color-coded results

**Tab 3: Help**
- Quick reference for all commands

### Keyboard Shortcuts

| Key | Action |
|-----|--------|
| `q` | Quit dashboard |
| `i` | Enter command mode |
| `Esc` | Exit command mode |
| `Enter` | Execute command |
| arrows | Navigate keys |
| `Tab` | Next tab |

### Example Dashboard Session

**Main Dashboard (Keys View):**

![Dashboard Screenshot](./examples/screen_1.png)

**History Tab:**

![History Screenshot](./examples/screen_2.png)

**Key Features:**
- ✅ Real-time key updates (refresh every 250ms)
- ✅ Color-coded command results (cyan=input, green=success, red=error)
- ✅ TTL countdown with visual indicator for expiring keys
- ✅ Responsive keyboard navigation
- ✅ Works alongside TCP server on port 6379

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

### Key Existence Check

```
SET key1 value1
OK
SET key2 value2
OK
EXISTS key1 key2 key3
key1 -> true
key2 -> true
key3 -> false
EXISTS key1
key1 -> true
DEL key1
OK
EXISTS key1 key2
key1 -> false
key2 -> true
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

- **Arc<Db> with interior mutability** - `Db` wraps `Arc<Mutex<DbInner>>` for efficient cloning
- **Command trait pattern** - Each command implements `execute(&self, db: &Arc<Db>, client_id: u64)`
- **Tokio tasks** - Each client spawns an async task
- **Client IDs** - `AtomicU64` counter for unique IDs
- **Broadcast channels** - `tokio::sync::broadcast` for pub/sub (capacity: 16)

### Command Pattern

Commands are implemented using the **Command trait pattern**:

```rust
// Each command is a separate struct
pub struct Get { pub key: String }
pub struct Set { pub key: String, pub value: String, pub expiration: Option<u64> }

// Command trait defines execution
pub trait Command {
    fn execute(&self, db: &Arc<Db>, client_id: u64) -> Return;
}

// Request parses and converts to Command
impl Request {
    pub fn into_command(self) -> Box<dyn Command> {
        match self {
            Request::GET(key) => Box::new(Get { key }),
            // ...
        }
    }
}
```

**Benefits**:
- Easy to add new commands (just implement Command trait)
- Self-contained command logic
- Clean separation of concerns
- Follows idiomatic Rust patterns

### Expiration Strategy

- **Lazy expiration** - Keys are checked and removed on `GET`, `TTL`, or `EXISTS`
- **No background cleanup** - Expired keys remain in memory until accessed

### Message Flow

```
Publisher                  Db (Arc<Mutex<DbInner>>)        Subscriber
    │                              │                           │
    │  PUB channel msg             │                           │
    │ ────────────────────────────►│                           │
    │  Request::parse()            │                           │
    │  .into_command()             │                           │
    │  command.execute(&db)        │                           │
    │                              │  broadcast::send()        │
    │                              │ ─────────────────────────►│
    │                              │                           │  MESSAGE ...
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
├── main.rs                 # Server entry point, connection handling
├── db.rs                   # Database layer (Arc<Mutex<DbInner>>)
├── command.rs              # Command trait definition
├── commands/               # Individual command implementations
│   ├── mod.rs              # Module exports
│   ├── get.rs              # GET command
│   ├── set.rs              # SET command
│   ├── del.rs              # DEL command
│   ├── incr.rs             # INCR command
│   ├── decr.rs             # DECR command
│   ├── save.rs             # SAVE command
│   ├── load.rs             # LOAD command
│   ├── drop.rs             # DROP command
│   ├── publish.rs          # PUB command
│   ├── subscribe.rs        # SUB command
│   ├── unsubscribe.rs       # UNSUB command
│   ├── ttl.rs              # TTL command
│   └── exists.rs           # EXISTS command
├── request.rs              # Request parsing and conversion to Command
├── stock.rs                # Key-value storage with expiration
├── channel_manager.rs      # Pub/sub channel management
├── returns.rs              # Return types
├── ui/                     # Terminal UI dashboard
│   ├── mod.rs              # UI module exports
│   ├── app.rs              # Application state
│   └── ui.rs               # TUI rendering and event handling
└── lib.rs                  # Library exports
```

## Architecture Evolution

**Current Architecture (Command Trait Pattern)**:
- `Db` with interior mutability (`Arc<Mutex<DbInner>>`)
- Separate command files in `commands/` directory
- Each command implements `Command` trait
- `Arc<Db>` passed to each command's `execute()` method
- Extensible: add new command by creating struct + implementing trait

## Dependencies

| Crate | Usage |
|-------|-------|
| `tokio` | Async runtime, TCP, sync primitives |
| `serde` | Serialization |
| `serde_json` | JSON persistence |
| `ratatui` | Terminal UI dashboard |
| `crossterm` | Terminal manipulation |

## Roadmap

See [FEATURES.md](FEATURES.md) for planned and completed features.

## License

Licensed under [MIT](LICENSE-MIT)
