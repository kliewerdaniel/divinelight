# DivineLight

<div align="center">

**A local-first, unified AI memory system combining verbatim cold storage, structured knowledge graphs, and graph-aware reasoning agents.**

[![Rust](https://img.shields.io/badge/Rust-1.70%2B-orange.svg)](https://www.rust-lang.org/)
[![License: MIT](https://img.shields.io/badge/License-MIT-blue.svg)](LICENSE)
[![Tests](https://img.shields.io/badge/Tests-22%20passing-green.svg)](https://github.com/divinelight/divinelight)

</div>

## Why DivineLight?

DivineLight is your **personal AI memory companion** that:

- 🔒 **Keeps everything local** — No data ever leaves your machine
- 🧠 **Builds knowledge graphs automatically** — Concepts are extracted and linked from your content
- 🔍 **Searches intelligently** — Hybrid keyword + semantic search across all memories
- ⚖️ **Detects contradictions** — Finds conflicting ideas in your knowledge base
- 🤖 **Provides AI agents** — Specialized agents for retrieval, verification, synthesis, and contradiction detection

## Quick Start

### Prerequisites

- **Rust 1.70+** — [Install Rust](https://rustup.rs/)
- No external database required (SQLite bundled)

### Run the Server

```bash
cargo run
```

The server starts at **`http://127.0.0.1:8080`**

### Web Interface

Open `frontend/index.html` in your browser for a full-featured UI with:
- Memory list and search
- Memory detail view with interpretation
- Knowledge graph browser
- Agent panel
- Backup/restore controls

### Try It Out

```bash
# Check health
curl http://127.0.0.1:8080/health

# Ingest a memory (auto-creates graph nodes from content)
curl -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{"source": "notes/ai-research.md", "format": "markdown", "content": "Machine learning uses neural networks with backpropagation to train models on large datasets. Deep learning has revolutionized computer vision and natural language processing.", "tags": ["ai", "research"]}'

# Search memories
curl -X POST http://127.0.0.1:8080/api/v1/retrieve \
  -H "Content-Type: application/json" \
  -d '{"query": "neural networks", "limit": 5}'

# View graph stats
curl http://127.0.0.1:8080/api/v1/graph/metadata
```

## Features

| Feature | Description |
|---------|-------------|
| **Memory Storage** | Append-only storage with SHA-256 integrity verification |
| **Auto-Graph Construction** | Automatically extracts concepts → creates graph nodes & edges |
| **Hybrid Retrieval** | Keyword + semantic similarity search |
| **Reasoning Engine** | Query interpretation with conflict detection |
| **Specialized Agents** | Retriever, Verifier, Synthesizer, ContradictionDetector |
| **Knowledge Graph** | BFS traversal, path finding, node/edge CRUD |
| **Backup/Export** | Full backup/restore, JSONL import/export |
| **Content Deduplication** | Smart deduplication prevents duplicate nodes/edges |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                      Ingestion Layer                            │
│         (files, notes, web content, any text input)            │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                     MemoryStore                                 │
│   • Append-only JSON files with SHA-256 checksums              │
│   • SQLite metadata index                                       │
│   • Tag-based filtering                                         │
└──────────────────────────┬──────────────────────────────────────┘
                           │
         ┌─────────────────┼─────────────────┐
         ▼                 ▼                 ▼
┌──────────────┐   ┌──────────────┐   ┌──────────────┐
│  Retrieval   │   │  GraphStore  │   │  Reasoning  │
│   Engine     │   │ (auto-extract│   │   Engine    │
│              │   │  concepts)   │   │             │
│ • Keyword    │   │              │   │ • Interpret │
│ • Semantic   │   │ • Nodes     │   │ • Conflict  │
│ • Ranking    │   │ • Edges     │   │   detection │
└──────┬───────┘   └──────┬───────┘   └──────┬───────┘
       │                 │                 │
       └─────────────────┼─────────────────┘
                         ▼
              ┌───────────────────────┐
              │       Agents          │
              │  • Retriever          │
              │  • Verifier           │
              │  • Synthesizer        │
              │  • ContradictionDetector
              └───────────┬───────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │   REST API (Axum)    │
              │   + Web UI           │
              └───────────────────────┘
```

## Configuration

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DIVINELIGHT_DATA_DIR` | Platform data dir | Data storage location |
| `DIVINELIGHT_HOST` | `127.0.0.1` | Server bind address |
| `DIVINELIGHT_PORT` | `8080` | Server port |
| `RUST_LOG` | `divinelight=info` | Log level |

### Data Storage

All data stays in `DIVINELIGHT_DATA_DIR`:

| Path | Description |
|------|-------------|
| `memories/*.json` | Verbatim memory files |
| `divinelight.db` | Memory metadata index |
| `graph.db` | Knowledge graph (nodes + edges) |
| `retrieval.db` | Search index |

## API Reference

### Memory API

```bash
# Ingest memory
POST /api/v1/memory/ingest
{
  "source": "file.md",
  "format": "markdown",
  "content": "Your content here...",
  "tags": ["tag1", "tag2"]
}

# Get memory
GET /api/v1/memory/:id

# List memories
GET /api/v1/memory/list?limit=50&offset=0

# Search (GET endpoint)
GET /api/v1/memory/search?q=query&limit=20

# Delete memory
DELETE /api/v1/memory/:id
```

### Graph API

```bash
# Create node
POST /api/v1/graph/nodes
{ "node_type": "concept", "label": "AI", "properties": {}, "provenance": [] }

# List nodes
GET /api/v1/graph/nodes/list?limit=100

# Create edge
POST /api/v1/graph/edges
{ "source": "node-1", "target": "node-2", "relation": "related_to", "confidence": 0.9 }

# Traverse (BFS)
POST /api/v1/graph/traverse
{ "start_node_id": "node-1", "depth": 3 }

# Find path
POST /api/v1/graph/path
{ "start_id": "node-1", "end_id": "node-2", "max_depth": 5 }

# Get metadata
GET /api/v1/graph/metadata
```

### Agent API

```bash
# Retriever - find relevant memories
POST /api/v1/agents/retriever
{ "query": "neural networks", "limit": 10 }

# Verifier - check memory integrity
POST /api/v1/agents/verifier
{ "memory_ids": ["mp_xxx", "mp_yyy"] }

# Synthesizer - combine memories
POST /api/v1/agents/synthesizer
{ "query": "Summarize the key points", "limit": 5 }

# ContradictionDetector - find conflicts
POST /api/v1/agents/contradiction
{ "memory_ids": ["mp_xxx", "mp_yyy"] }
```

### Backup API

```bash
# Create backup
POST /api/v1/backup/create
{ "path": "./backup" }

# Restore
POST /api/v1/backup/restore
{ "path": "./backup" }

# Export to JSONL
POST /api/v1/backup/export
{ "path": "./export.jsonl" }

# Import from JSONL
POST /api/v1/backup/import
{ "path": "./export.jsonl" }
```

### CLI Commands

```bash
cargo run -- backup  ./backups/my-backup
cargo run -- restore ./backups/my-backup
cargo run -- export  ./data.jsonl
cargo run -- import  ./data.jsonl
cargo run -- status
```

## Use Cases

### 1. Build Your Knowledge Base

```bash
# Ingest a blog post
curl -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{
    "source": "blog/understanding-transformers.md",
    "format": "markdown",
    "content": "Transformers use attention mechanisms to process sequential data...",
    "tags": ["ai", "transformers", "deep-learning"]
  }'
```

The system automatically:
- Stores the memory with integrity checksum
- Indexes for keyword search
- Extracts key concepts → adds to knowledge graph
- Creates edges between related concepts

### 2. Ask Questions

```bash
curl -X POST http://127.0.0.1:8080/api/v1/reason/interpret \
  -H "Content-Type: application/json" \
  -d '{"query": "What do you know about neural networks?"}'
```

Returns relevant memories + AI interpretations with confidence scores.

### 3. Detect Contradictions

```bash
curl -X POST http://127.0.0.1:8080/api/v1/agents/contradiction \
  -H "Content-Type: application/json" \
  -d '{"memory_ids": ["mp_xxx", "mp_yyy"]}'
```

Finds semantic conflicts between memories.

### 4. Manual Graph Operations

```bash
# Create a concept
curl -X POST http://127.0.0.1:8080/api/v1/graph/nodes \
  -H "Content-Type: application/json" \
  -d '{"node_type": "concept", "label": "Rust", "properties": {"description": "Systems language"}}'

# Link concepts
curl -X POST http://127.0.0.1:8080/api/v1/graph/edges \
  -H "Content-Type: application/json" \
  -d '{"source": "node-id-1", "target": "node-id-2", "relation": "related_to", "confidence": 0.9}'
```

## Running Tests

```bash
cargo test
```

All **22 tests** should pass.

## Development

```bash
# Build
cargo build

# Run with logging
RUST_LOG=debug cargo run

# Run clippy
cargo clippy -- -D warnings
```

## License

MIT License — See [LICENSE](LICENSE) for details.