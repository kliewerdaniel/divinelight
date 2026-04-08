# DivineLight

A unified AI memory system combining MemPalace verbatim storage, Graphify knowledge graphs, and Synt-inspired reasoning agents.

## Overview

DivineLight is a local-first, production-grade memory system that integrates:
- **MemPalace**: Verbatim cold storage for raw memory without summarization
- **Graphify**: Knowledge graph construction and structured reasoning
- **Synt Agents**: Graph-aware reasoning, evaluation, and belief-state management

## Features

### Core Capabilities
- **Memory Ingestion**: Store verbatim memories with SHA256 integrity checks
- **Knowledge Graph**: Entity extraction, relationship discovery, BFS/DFS traversal
- **Hybrid Retrieval**: Keyword search with relevance scoring
- **Reasoning Layer**: Belief-state management and interpretation
- **Agent System**: Retriever, Verifier, Synthesizer, Contradiction Detector agents

### API Endpoints
- Memory: `POST /api/v1/memory/ingest`, `GET /api/v1/memory/:id`
- Graph: `POST /api/v1/graph/nodes`, `POST /api/v1/graph/edges`, `POST /api/v1/graph/traverse`, `POST /api/v1/graph/path`
- Retrieval: `POST /api/v1/retrieve`
- Reasoning: `POST /api/v1/reason/interpret`, `GET /api/v1/reason/beliefs/:id`
- Agents: `/api/v1/agents/retriever|verifier|synthesizer|contradiction`
- Backup: `POST /api/v1/backup/create|restore|export|import`

### CLI Commands
```bash
cargo run              # Start server on http://127.0.0.1:8080
cargo run -- backup <path>    # Create backup
cargo run -- restore <path>  # Restore from backup
cargo run -- export <path>   # Export data
cargo run -- import <path>   # Import data
cargo run -- status          # Check status
```

### Testing
```bash
cargo test              # Run all tests
```

## Installation

### Prerequisites
- Rust 1.70+
- SQLite (bundled via rusqlite)

### Build
```bash
cargo build --release
```

### Run
```bash
cargo run
```

The server starts on `http://127.0.0.1:8080`

## Configuration

Data is stored in `~/Library/Application Support/divinelight/`:
- `memories/` - JSON memory files
- `divinelight.db` - Memory metadata
- `graph.db` - Knowledge graph
- `retrieval.db` - Search index

## Architecture

```
┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│  Ingestion   │────▶│  MemPalace  │────▶│  Retrieval   │
│   Layer       │     │  (Storage)  │     │   Layer      │
└──────────────┘     └─────────────┘     └──────────────┘
                                               │
                                               ▼
┌──────────────┐     ┌─────────────┐     ┌──────────────┐
│   Graph      │◀────│   Graphify  │◀────│   Reasoning  │
│   Storage    │     │  (Extract)  │     │    Layer     │
└──────────────┘     └─────────────┘     └──────────────┘
                                               │
                                               ▼
                                    ┌──────────────────┐
                                    │    Agents        │
                                    │ - Retriever      │
                                    │ - Verifier       │
                                    │ - Synthesizer    │
                                    │ - Contradiction  │
                                    └──────────────────┘
```

## Tech Stack

- **Backend**: Rust with Axum web framework
- **Storage**: SQLite (rusqlite)
- **API**: RESTful JSON over HTTP
- **Frontend**: Web UI (frontend/index.html)

## License

MIT
