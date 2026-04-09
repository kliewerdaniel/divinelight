# DivineLight

A local-first, unified AI memory system combining verbatim cold storage (MemPalace), structured knowledge graphs (Graphify), and graph-aware reasoning agents (Synt-inspired).

## Quickstart

### Prerequisites
- Rust 1.70+
- No external database required (SQLite is bundled)

### Run the server
```bash
cargo run
```

Server starts at `http://127.0.0.1:8080`. Open `frontend/index.html` in a browser.

### Configuration
Copy `.env.example` to `.env` and adjust values:
```bash
cp .env.example .env
```
Environment variables:
| Variable | Default | Description |
|---|---|---|
| `DIVINELIGHT_DATA_DIR` | Platform data dir | Where memories and databases are stored |
| `DIVINELIGHT_HOST` | `127.0.0.1` | Server bind host |
| `DIVINELIGHT_PORT` | `8080` | Server port |
| `RUST_LOG` | `divinelight=info` | Log level |

### CLI Operations
```bash
cargo run -- backup  <path>   # Create backup
cargo run -- restore <path>   # Restore from backup
cargo run -- export  <path>   # Export memories to JSONL
cargo run -- import  <path>   # Import memories from JSONL
cargo run -- status           # Show data directory status
```

### Tests
```bash
cargo test
```

## Architecture

```
Ingestion ──▶ MemoryStore ──▶ RetrievalEngine
                                    │
              GraphStore ◀──────────┤
                                    │
              ReasoningEngine ◀─────┘
                    │
              Agents (Retriever, Verifier, Synthesizer, ContradictionDetector)
                    │
              REST API (Axum) ──▶ Frontend (HTML)
```

See `docs/architecture_overview.md` for full architecture diagrams.

## API Endpoints

| Method | Path | Description |
|---|---|---|
| GET | `/health` | System health check |
| POST | `/api/v1/memory/ingest` | Store a new memory |
| GET | `/api/v1/memory/:id` | Retrieve a memory by ID |
| GET | `/api/v1/memory/list` | List all memories (paginated) |
| POST | `/api/v1/graph/nodes` | Create a graph node |
| GET | `/api/v1/graph/nodes/:id` | Get a graph node |
| POST | `/api/v1/graph/edges` | Create a graph edge |
| GET | `/api/v1/graph/edges/:id` | Get a graph edge |
| GET | `/api/v1/graph/metadata` | Graph statistics |
| POST | `/api/v1/graph/traverse` | BFS/DFS traversal |
| POST | `/api/v1/graph/path` | Find path between nodes |
| POST | `/api/v1/retrieve` | Hybrid search |
| POST | `/api/v1/reason/interpret` | Run reasoning pipeline |
| GET | `/api/v1/reason/beliefs/:id` | Get belief state |
| POST | `/api/v1/agents/retriever` | Run retriever agent |
| POST | `/api/v1/agents/verifier` | Run verifier agent |
| POST | `/api/v1/agents/synthesizer` | Run synthesizer agent |
| POST | `/api/v1/agents/contradiction` | Run contradiction detector |
| POST | `/api/v1/backup/create` | Create backup |
| POST | `/api/v1/backup/restore` | Restore backup |
| POST | `/api/v1/backup/export` | Export data |
| POST | `/api/v1/backup/import` | Import data |

## Data Storage

All data is stored locally in `DIVINELIGHT_DATA_DIR`:
- `memories/*.json` — verbatim memory files (append-only)
- `divinelight.db` — memory metadata index (SQLite)
- `graph.db` — knowledge graph nodes and edges (SQLite)
- `retrieval.db` — keyword search index (SQLite)

## License
MIT
