# FEATURES

## Core Logic & Data Operations
- [x] GET/SET/DEL - Basic key-value store operations.
- [x] SET EXP - Key expiration support with TTL parameter.
- [x] DROP - Clear all data from the store.
- [x] ASYNC SERVER - Multi-threaded async server with Tokio runtime.
- [ ] INCR/DECR - Atomic increment and decrement for thread-safe counting.
- [ ] MULTI/EXEC - Support for atomic transactions.
- [ ] EVICTION POLICY - Basic memory management with LRU, LFU, etc.

## Persistence & Disaster Recovery
- [x] SAVE/LOAD - JSON-based persistence to file.
- [x] GRACEFUL SHUTDOWN - Signal handling for clean Ctrl+C termination.
- [ ] AOF - Append Only File for real-time data safety.
- [ ] SCHEDULED BACKUP - Save a backup every x time to prevent data loss.
- [ ] OS SIGNALS - Handle SIGTERM and other signals to save data before exiting.

## Protocol & Networking
- [x] PUB/SUB/UNSUB - Pub/Sub messaging system with broadcast channels.
- [x] CLIENT TRACKING - Client connection tracking and cleanup.
- [ ] RESP - Support the official Redis Serialization Protocol.
- [ ] REPLICATION - Simple replication mechanism for data redundancy.
- [ ] TELL - Server-to-server communication in a cluster setup.

## Security & Configuration
- [ ] AUTH - Password protection for client connections.
- [ ] CONFIG - Runtime configuration changes.
- [ ] CONFIG FILES - Support config files or ENV vars for port, passwords, and limits.

## Observability & Performance
- [ ] INFO - Server stats and memory usage.
- [ ] STRUCTURED LOGGING - Using `tracing` or `log` crate.
- [ ] METRICS - Monitoring endpoint for Prometheus integration.
- [ ] ACTIVE EXPIRATION - Background thread that periodically scans and removes expired keys.
- [ ] COMPRESSION - Compress values above threshold to reduce memory usage.

## Data Structures
- [ ] LISTS - Ordered lists with LPUSH, RPUSH, LPOP, RPOP, LRANGE.
- [ ] HASHES - Field-value pairs with HSET, HGET, HDEL, HGETALL.
- [ ] SETS - Unique unordered collections with SADD, SREM, SMEMBERS, SISMEMBER.
- [ ] SORTED SETS - Score-ordered elements with ZADD, ZREM, ZRANGE.

## Advanced Operations
- [ ] MGET/MSET - Batch get/set multiple keys in a single command.
- [ ] KEYS - Find all keys matching a pattern (e.g., `KEYS user:*`).
- [ ] TTL - Query remaining time-to-live on a key.
- [ ] EXISTS - Check if one or more keys exist.
- [ ] RENAME - Atomically rename a key.

## Developer Experience
- [ ] CLI CLIENT - Interactive command-line client for testing and debugging.
- [ ] CONNECTION POOLING - Reuse connections for better performance.
- [ ] BATCH - Execute multiple commands in a single request.
