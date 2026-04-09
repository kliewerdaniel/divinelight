# DivineLight

A local-first, unified AI memory system combining verbatim cold storage, structured knowledge graphs, and graph-aware reasoning agents.

## Overview

DivineLight is a personal AI memory system that:
- **Stores memories** - Captures content with metadata, checksums, and tags
- **Builds knowledge graphs** - Automatically extracts concepts and creates graph relationships from ingested content
- **Reasons over memories** - Uses semantic similarity and contradiction detection
- **Provides agents** - Specialized agents for retrieval, verification, synthesis, and contradiction detection

All data stays local - nothing is sent to external servers.

## Features

| Feature | Description |
|---------|-------------|
| **Memory Storage** | Append-only storage with SHA-256 checksums |
| **Auto-Graph Construction** | Automatically extracts concepts and creates graph nodes/edges on ingestion |
| **Hybrid Retrieval** | Keyword + semantic search across all memories |
| **Reasoning Engine** | Interprets queries and detects conflicts between memories |
| **Specialized Agents** | Retriever, Verifier, Synthesizer, ContradictionDetector |
| **Knowledge Graph** | BFS/DFS traversal, path finding between concepts |
| **Backup/Export** | JSONL export, full backup/restore |

## Quickstart

### Prerequisites
- Rust 1.70+
- No external database required (SQLite is bundled)

### Run the Server

```bash
cargo run
```

Server starts at `http://127.0.0.1:8080`

### Web Interface

Open `frontend/index.html` in a browser for the web UI.

### Test It

```bash
# Check health
curl http://127.0.0.1:8080/health

# Ingest a test memory
curl -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{"source": "test.txt", "format": "text", "content": "Python machine learning AI neural networks", "tags": ["test"]}'

# Check graph (auto-constructed from content)
curl http://127.0.0.1:8080/api/v1/graph/metadata
```

## Configuration

Copy `.env.example` to `.env`:
```bash
cp .env.example .env
```

### Environment Variables

| Variable | Default | Description |
|----------|---------|-------------|
| `DIVINELIGHT_DATA_DIR` | Platform data dir | Where memories and databases are stored |
| `DIVINELIGHT_HOST` | `127.0.0.1` | Server bind host |
| `DIVINELIGHT_PORT` | `8080` | Server port |
| `RUST_LOG` | `divinelight=info` | Log level (debug, info, warn, error) |

## Use Cases

### 1. Ingest Blog Posts or Notes

```bash
# Ingest a single file
curl -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{
    "source": "blog/my-post.md",
    "format": "markdown",
    "content": "Your full content here...",
    "tags": ["blog", "ai"]
  }'

# Batch ingest multiple files
for f in mynotes/*.md; do
  curl -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
    -H "Content-Type: application/json" \
    -d "{\"source\": \"$f\", \"format\": \"markdown\", \"content\": \"$(cat $f)\", \"tags\": [\"notes\"]}"
done
```

**What happens:** The content is stored, indexed for search, AND automatically analyzed to extract key concepts which are added to the knowledge graph.

### 2. Search Your Memories

```bash
# Search for anything about Python
curl -X POST http://127.0.0.1:8080/api/v1/retrieve \
  -H "Content-Type: application/json" \
  -d '{
    "query": "Python machine learning",
    "limit": 10
  }'
```

Returns ranked results with relevance scores.

### 3. Query the Knowledge Graph

```bash
# Get graph statistics
curl http://127.0.0.1:8080/api/v1/graph/metadata

# Traverse graph (get all nodes/edges starting from a node)
curl -X POST http://127.0.0.1:8080/api/v1/graph/traverse \
  -H "Content-Type: application/json" \
  -d '{
    "start_node_id": "node-id-here",
    "depth": 3
  }'

# Find path between two concepts
curl -X POST http://127.0.0.1:8080/api/v1/graph/path \
  -H "Content-Type: application/json" \
  -d '{
    "start_id": "node-id-1",
    "end_id": "node-id-2",
    "max_depth": 5
  }'
```

### 4. Ask Questions (Reasoning)

```bash
# Interpret a query and get relevant memories + conflict detection
curl -X POST http://127.0.0.1:8080/api/v1/reason/interpret \
  -H "Content-Type: application/json" \
  -d '{"query": "What do you know about AI agents?"}'
```

Returns:
- Relevant memories
- Interpretations with confidence scores
- Conflict flags (only for semantically similar content with contradictions)

### 5. Use Specialized Agents

```bash
# Retriever Agent - Find relevant memories
curl -X POST http://127.0.0.1:8080/api/v1/agents/retriever \
  -H "Content-Type: application/json" \
  -d '{"query": "neural networks", "limit": 5}'

# Verifier Agent - Check memory integrity
curl -X POST http://127.0.0.1:8080/api/v1/agents/verifier \
  -H "Content-Type: application/json" \
  -d '{"memory_ids": ["mp_20260409_xxx"]}'

# Synthesizer Agent - Combine multiple memories into summary
curl -X POST http://127.0.0.1:8080/api/v1/agents/synthesizer \
  -H "Content-Type: application/json" \
  -d '{"memory_ids": ["mp_20260409_xxx", "mp_20260409_yyy"], "query": "Summarize what you learned"}'

# Contradiction Detector - Find conflicts between memories
curl -X POST http://127.0.0.1:8080/api/v1/agents/contradiction \
  -H "Content-Type: application/json" \
  -d '{"memory_ids": ["mp_20260409_xxx", "mp_20260409_yyy"]}'
```

### 6. Manual Graph Operations

```bash
# Create a concept node
curl -X POST http://127.0.0.1:8080/api/v1/graph/nodes \
  -H "Content-Type: application/json" \
  -d '{
    "node_type": "concept",
    "label": "Machine Learning",
    "properties": {"description": "Field of AI"},
    "provenance": []
  }'

# Create relationship between nodes
curl -X POST http://127.0.0.1:8080/api/v1/graph/edges \
  -H "Content-Type: application/json" \
  -d '{
    "source": "node-id-1",
    "target": "node-id-2",
    "relation": "related_to",
    "properties": {"weight": 0.8},
    "provenance": [],
    "confidence": 0.8
  }'

# Get specific node
curl http://127.0.0.1:8080/api/v1/graph/nodes/node-id-here
```

### 7. Backup and Export

```bash
# Create backup
curl -X POST http://127.0.0.1:8080/api/v1/backup/create \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/backup.tar.gz"}'

# Restore from backup
curl -X POST http://127.0.0.1:8080/api/v1/backup/restore \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/backup.tar.gz"}'

# Export to JSONL
curl -X POST http://127.0.0.1:8080/api/v1/backup/export \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/export.jsonl"}'

# Import from JSONL
curl -X POST http://127.0.0.1:8080/api/v1/backup/import \
  -H "Content-Type: application/json" \
  -d '{"path": "/path/to/export.jsonl"}'
```

Or use CLI commands:
```bash
cargo run -- backup  <path>
cargo run -- restore <path>
cargo run -- export  <path>
cargo run -- import  <path>
cargo run -- status
```

## API Reference

### Memory Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/memory/ingest` | Store a new memory |
| GET | `/api/v1/memory/:id` | Retrieve a memory by ID |
| GET | `/api/v1/memory/list` | List all memories (paginated) |

**Ingest Request:**
```json
{
  "source": "file path or URL",
  "format": "markdown|text|html",
  "content": "the actual content",
  "tags": ["tag1", "tag2"]
}
```

**List Query Params:** `?limit=50&offset=0`

### Graph Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/graph/nodes` | Create a graph node |
| GET | `/api/v1/graph/nodes/:id` | Get a graph node |
| POST | `/api/v1/graph/edges` | Create a graph edge |
| GET | `/api/v1/graph/edges/:id` | Get a graph edge |
| GET | `/api/v1/graph/metadata` | Graph statistics |
| POST | `/api/v1/graph/traverse` | BFS traversal |
| POST | `/api/v1/graph/path` | Find path between nodes |

**Create Node Request:**
```json
{
  "node_type": "concept|entity|source",
  "label": "Node Label",
  "properties": {"key": "value"},
  "provenance": ["memory_id_1"]
}
```

**Create Edge Request:**
```json
{
  "source": "node-id",
  "target": "node-id", 
  "relation": "related_to|depends_on|mentions",
  "properties": {},
  "provenance": ["memory_id"],
  "confidence": 1.0
}
```

### Retrieval Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/retrieve` | Hybrid keyword + semantic search |

**Retrieve Request:**
```json
{
  "query": "search terms",
  "modes": ["keyword", "semantic"],
  "limit": 10
}
```

### Reasoning Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/reason/interpret` | Run reasoning pipeline |
| GET | `/api/v1/reason/beliefs/:id` | Get belief state |

**Interpret Request:**
```json
{
  "query": "your question",
  "context_ids": ["optional_memory_ids"]
}
```

### Agent Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/agents/retriever` | Run retriever agent |
| POST | `/api/v1/agents/verifier` | Run verifier agent |
| POST | `/api/v1/agents/synthesizer` | Run synthesizer agent |
| POST | `/api/v1/agents/contradiction` | Run contradiction detector |

### Backup Endpoints

| Method | Path | Description |
|--------|------|-------------|
| POST | `/api/v1/backup/create` | Create backup |
| POST | `/api/v1/backup/restore` | Restore backup |
| POST | `/api/v1/backup/export` | Export to JSONL |
| POST | `/api/v1/backup/import` | Import from JSONL |

## Architecture

```
┌─────────────────────────────────────────────────────────────────┐
│                         Ingestion                                │
│    (files, notes, blog posts, any text content)                  │
└──────────────────────────┬──────────────────────────────────────┘
                           │
                           ▼
┌─────────────────────────────────────────────────────────────────┐
│                       MemoryStore                                │
│  - Append-only storage with SHA-256 checksums                   │
│  - SQLite metadata index                                         │
│  - JSON memory files                                             │
└──────────────────────────┬──────────────────────────────────────┘
                           │
            ┌──────────────┼──────────────┐
            ▼              ▼              ▼
    ┌────────────┐  ┌────────────┐  ┌────────────┐
    │ Retrieval  │  │ GraphStore  │  │ Reasoning  │
    │  Engine    │  │ (auto-extract│  │  Engine    │
    │ keyword+   │  │ concepts)   │  │ conflict   │
    │ semantic   │  │             │  │ detection  │
    └─────┬──────┘  └──────┬───────┘  └─────┬──────┘
          │                │                 │
          └────────────────┼─────────────────┘
                           ▼
              ┌───────────────────────┐
              │      Agents           │
              │  Retriever            │
              │  Verifier             │
              │  Synthesizer          │
              │  ContradictionDetector│
              └───────────┬───────────┘
                          │
                          ▼
              ┌───────────────────────┐
              │   REST API (Axum)     │
              │   + Frontend HTML    │
              └───────────────────────┘
```

## Data Storage

All data is stored locally in `DIVINELIGHT_DATA_DIR` (default: `~/Library/Application Support/divinelight/`):

| File/Directory | Description |
|---------------|-------------|
| `memories/*.json` | Verbatim memory files (append-only) |
| `divinelight.db` | Memory metadata index (SQLite) |
| `graph.db` | Knowledge graph nodes and edges (SQLite) |
| `retrieval.db` | Keyword search index (SQLite) |

## Running Tests

```bash
cargo test
```

All 16 unit tests should pass.

## License

MIT
