# Prediction Market Order Matching Engine

A real-time order matching engine built in Rust, featuring an HTTP API, WebSocket feed, and multi-instance support.

## Architecture

```
Client A ──POST /order──▶ ┌──────────────┐
Client B ──POST /order──▶ │  Port 3000   │──┐
Client C ──GET /orderbook▶│  Port 3001   │  │
Client D ──WS /ws───────▶ └──────────────┘  │
                                             ▼
                                   Arc<Mutex<OrderBook>>
                                   (single matching engine)
```

## How does the system handle multiple API server instances without double-matching?

All server instances (ports 3000, 3001) run inside a single process sharing the same `Arc<Mutex<OrderBook>>`. The `Arc` (atomic reference counting) allows multiple threads to own the same state, while `Mutex` ensures only one thread can access the order book at a time. This guarantees that every order goes through a single matching engine — no double-matching is possible.

For true horizontal scaling across separate processes, this would evolve into a dedicated matching engine process with API servers forwarding orders via a message queue (e.g., Redis, gRPC, or TCP).

## What data structure is used for the order book and why?

```rust
bids: BTreeMap<u64, VecDeque<Order>>  // buy side
asks: BTreeMap<u64, VecDeque<Order>>  // sell side
```

- **BTreeMap** — sorted map by price. Gives O(log n) insert/lookup and O(1) access to best price via `first_key_value()` (lowest ask) and `last_key_value()` (highest bid). A HashMap would require scanning all keys to find the best price.
- **VecDeque** — double-ended queue for FIFO ordering within a price level. `push_back()` for new orders, `pop_front()` for the oldest order — both O(1). A Vec would be O(n) for front removal.

This implements **price-time priority**: best price matches first, and at the same price, the earliest order matches first.

## What breaks first under real production load?

- **Mutex contention** — all order submissions are serialized through a single lock. Under high throughput, this becomes the bottleneck.
- **Single-process architecture** — the matching engine cannot scale horizontally.
- **Broadcast channel backpressure** — slow WebSocket clients could lag behind and miss fills (buffer size is 100).
- **No persistence** — a crash loses all state (order book, pending orders, everything).

## What would I build next with 4 more hours?

- **Persistence** — write-ahead log or SQLite for crash recovery
- **Order cancellation** — `DELETE /orders/:id` endpoint
- **Separate matching engine process** — gRPC/TCP interface for true multi-instance deployment
- **Metrics & logging** — using the `tracing` crate
- **Better error handling** — proper HTTP status codes and error responses

## API Endpoints

| Method | Path | Description |
|--------|------|-------------|
| GET | `/health` | Health check |
| POST | `/order` | Submit a new order |
| GET | `/orderbook` | Get aggregated order book |
| GET | `/ws` | WebSocket feed for real-time fill events |

### POST /order

```json
// Request
{ "side": "Buy", "price": 100, "qty": 10 }

// Response
{ "order_id": 1, "fills": [{ "maker_order_id": 2, "taker_order_id": 1, "price": 100, "qty": 10 }] }
```

### GET /orderbook

```json
// Response — aggregated qty per price level
{ "bids": [[95, 20], [90, 15]], "asks": [[100, 10], [105, 5]] }
```

## Running

```bash
cargo build
cargo run
# Server starts on ports 3000 and 3001
```

## Testing

```bash
# Health check
curl http://localhost:3000/health

# Place a sell order
curl -X POST http://localhost:3000/order -H "Content-Type: application/json" -d '{"side":"Sell","price":100,"qty":10}'

# Place a matching buy order
curl -X POST http://localhost:3001/order -H "Content-Type: application/json" -d '{"side":"Buy","price":100,"qty":10}'

# View order book
curl http://localhost:3000/orderbook

# WebSocket feed (requires wscat: npm install -g wscat)
npx wscat -c ws://localhost:3000/ws
```

## Tech Stack

| Crate | Purpose |
|-------|---------|
| `axum` | HTTP server + WebSocket |
| `tokio` | Async runtime |
| `serde` | Serialization/Deserialization |
| `serde_json` | JSON support |
| `futures` | Stream utilities for WebSocket |
