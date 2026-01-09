# CEX

A minimal centralized exchange backend in Rust.

## Architecture

```
┌─────────┐     ┌───────────┐     ┌────────┐
│   API   │────▶│   Redis   │◀────│ Engine │
└─────────┘     └───────────┘     └────────┘
                     │
        ┌────────────┼────────────┐
        ▼            ▼            ▼
   ┌─────────┐  ┌─────────┐  ┌──────────┐
   │   WS    │  │DB Filler│  │ Postgres │
   └─────────┘  └─────────┘  └──────────┘
```

| Service | Description |
|---------|-------------|
| `api` | HTTP gateway (orders, markets, health) |
| `engine` | In-memory orderbook matching engine |
| `ws` | WebSocket real-time event streaming |
| `db_filler` | Persists events to Postgres |

## Quick Start

```bash
# Start infrastructure
docker compose up -d redis postgres

# Run services (in separate terminals)
cargo run -p api
cargo run -p engine
cargo run -p ws
cargo run -p db_filler
```

Or with Docker:

```bash
docker compose up -d
```

## API

| Endpoint | Method | Description |
|----------|--------|-------------|
| `/order/new` | POST | Create order |
| `/order/cancel` | POST | Cancel order |
| `/markets` | GET | List markets |
| `/health` | GET | Health check |

## Tests

```bash
cargo test --all
```

## Requirements

- Rust 1.77+
- Redis
- PostgreSQL
