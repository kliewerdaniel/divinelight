# DivineLight Coding Agent Prompt
## System Context

You are an autonomous coding agent working on **DivineLight** — a local-first, unified AI memory system written in Rust. The system integrates three subsystems: MemPalace (verbatim cold storage), Graphify (knowledge graph), and Synt-inspired agents (reasoning layer). The backend is an Axum HTTP server with SQLite persistence, exposing a REST API consumed by a frontend HTML interface.

Your task is to systematically refactor, complete, and harden this repository into a clean, production-grade, modular system. Execute the steps below in order. Do not skip steps. Each step specifies exact files to create, modify, or delete.

---

## STEP 1 — Fix Compilation Errors and Remove Dead Code

**File: `src/models/mod.rs`**
Remove the unused wildcard import:
```rust
// DELETE this line:
use serde::{Deserialize, Serialize};
```

**File: `src/storage/graph.rs`**
Remove unused function parameters from `query_edges` (the `source`, `target`, `relation` params are accepted but never used in the SQL):
```rust
// REPLACE the function signature with:
pub fn query_edges(&self, limit: usize) -> Result<Vec<GraphEdge>> {
```
Update all internal callers of `query_edges` to match the new signature.

**File: `src/cli.rs`**
This file defines a `main()` function that conflicts with `src/main.rs`. It is also non-functional (no async runtime, no actual logic). Delete this file entirely:
```
DELETE: src/cli.rs
```

**File: `Cargo.toml`**
Remove the `[[bin]]` entry for any CLI binary that referenced `src/cli.rs` if present, and ensure the only `[[bin]]` is:
```toml
[[bin]]
name = "divinelight"
path = "src/main.rs"
```

---

## STEP 2 — Add CORS Middleware to the Axum Router

**File: `src/api/mod.rs`**

The frontend at `frontend/index.html` calls `http://127.0.0.1:8080` from the browser. Without CORS headers, all requests fail. Add the CORS layer.

At the top of `create_router`, import and configure CORS:
```rust
use tower_http::cors::{CorsLayer, Any};
use http::Method;
```

Update `create_router` to attach a CORS layer:
```rust
pub fn create_router(state: Arc<AppState>) -> Router {
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        // ... all existing routes unchanged ...
        .with_state(state)
        .layer(cors)
}
```

**File: `Cargo.toml`**
Ensure `tower-http` has the `cors` feature enabled:
```toml
tower-http = { version = "0.5", features = ["trace", "cors"] }
```

---

## STEP 3 — Fix the `health` Endpoint to Return JSON

**File: `src/api/mod.rs`**

The health endpoint returns a plain string but the frontend and spec expect JSON. Replace:
```rust
async fn health() -> &'static str {
    "OK"
}
```
With:
```rust
#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> AxumJson<HealthResponse> {
    AxumJson(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}
```

---

## STEP 4 — Fix the Retrieval Engine Scoring Bug

**File: `src/retrieval/mod.rs`**

The current `search` method returns 0 results when the query is `"*"` because `"*"` never matches any content term. The frontend calls `retrieve` with `query: "*"` to count all memories. Fix `calculate_score` to treat `"*"` as a wildcard:

```rust
fn calculate_score(&self, query: &str, content: &str, tags: &str) -> f64 {
    // Wildcard: return all memories with a base score
    if query == "*" || query.trim().is_empty() {
        return 0.5;
    }

    let query_terms: Vec<&str> = query.split_whitespace().collect();
    let mut score = 0.0;

    for term in &query_terms {
        if content.contains(term) {
            score += 1.0;
        }
        if tags.contains(term) {
            score += 0.5;
        }
    }

    if score > 0.0 {
        let content_len = content.split_whitespace().count() as f64;
        score + (1.0 / (1.0 + (content_len / 100.0 + 1.0).ln()))
    } else {
        0.0
    }
}
```

---

## STEP 5 — Fix the Backup Count Logic

**File: `src/backup/mod.rs`**

The `count_lines` method opens a SQLite database as raw bytes and returns 0 or 1 based on whether the file is non-empty. This is incorrect. Replace the entire `count_lines` method with a real SQLite query:

```rust
fn count_lines(&self, db_path: &PathBuf, query: &str) -> Result<u64> {
    if !db_path.exists() {
        return Ok(0);
    }
    let conn = rusqlite::Connection::open(db_path)?;
    // The query string is like "SELECT COUNT(*) FROM nodes"
    // Safely execute and return 0 on table-not-found errors
    match conn.query_row(query, [], |row| row.get::<_, i64>(0)) {
        Ok(count) => Ok(count as u64),
        Err(_) => Ok(0),
    }
}
```

Add `use rusqlite;` at the top of the file if not already present.

---

## STEP 6 — Add Proper Error Handling with `AppError`

**File: `src/api/mod.rs`**

All handler functions currently return `Result<AxumJson<T>, String>`. Axum maps `String` errors to a 200 OK with an error body, not a proper HTTP error status. Replace with a proper error type.

Add this type at the top of `src/api/mod.rs` (after imports):

```rust
use axum::http::StatusCode;
use axum::response::{IntoResponse, Response};

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (
            StatusCode::INTERNAL_SERVER_ERROR,
            format!("Error: {}", self.0),
        )
            .into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}
```

Replace every handler return type from `Result<AxumJson<T>, String>` to `Result<AxumJson<T>, AppError>`.

Replace every `.map_err(|e| e.to_string())?` with just `?`.

Example — replace:
```rust
async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<String>,
) -> Result<AxumJson<MemoryObject>, String> {
    let memory = state.memory.lock().map_err(|e| e.to_string())?;
    let result = memory.get(&memory_id).map_err(|e| e.to_string())?;
    Ok(AxumJson(result))
}
```
With:
```rust
async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<String>,
) -> Result<AxumJson<MemoryObject>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let result = memory.get(&memory_id)?;
    Ok(AxumJson(result))
}
```

Apply this pattern to all handlers in `src/api/mod.rs`.

---

## STEP 7 — Add Missing `GET /api/v1/memory/list` Endpoint

**File: `src/api/mod.rs`**

The frontend counts memories by querying `retrieve` with `*`. Add a proper list endpoint.

Add to the router in `create_router`:
```rust
.route("/api/v1/memory/list", get(list_memories))
```

Add the handler:
```rust
#[derive(Debug, Deserialize)]
struct ListMemoriesQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ListMemoriesResponse {
    memories: Vec<MemoryObject>,
    total: u64,
}

async fn list_memories(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<ListMemoriesQuery>,
) -> Result<AxumJson<ListMemoriesResponse>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let memories = memory.list_all(limit, offset)?;
    let total = memory.count()?;
    Ok(AxumJson(ListMemoriesResponse { memories, total }))
}
```

---

## STEP 8 — Fix the Frontend Stats Display

**File: `frontend/index.html`**

The `refreshStats` function currently tries to count memories via a retrieve call with `query: "*"`. Replace it to use the new list endpoint and fix belief count (currently always 0 with no source):

Find and replace the `refreshStats` function:
```javascript
async function refreshStats() {
    try {
        const meta = await call('/api/v1/graph/metadata');
        document.getElementById('nodeCount').textContent = meta.node_count ?? 0;
        document.getElementById('edgeCount').textContent = meta.edge_count ?? 0;

        const list = await call('/api/v1/memory/list?limit=1');
        document.getElementById('memCount').textContent = list.total ?? 0;

        document.getElementById('beliefCount').textContent = '—';

        document.getElementById('systemResult').innerHTML =
            '<div class="result-item">System healthy ✓</div>';
    } catch (e) {
        showError('systemResult', e.message);
    }
}
```

---

## STEP 9 — Create `src/config.rs` for Centralized Configuration

Create a new file `src/config.rs`:

```rust
use std::path::PathBuf;

#[derive(Debug, Clone)]
pub struct Config {
    pub data_dir: PathBuf,
    pub host: String,
    pub port: u16,
    pub log_level: String,
}

impl Config {
    pub fn from_env() -> Self {
        let data_dir = std::env::var("DIVINELIGHT_DATA_DIR")
            .map(PathBuf::from)
            .unwrap_or_else(|_| {
                dirs::data_local_dir()
                    .unwrap_or_else(|| PathBuf::from("."))
                    .join("divinelight")
            });

        Self {
            data_dir,
            host: std::env::var("DIVINELIGHT_HOST").unwrap_or_else(|_| "127.0.0.1".to_string()),
            port: std::env::var("DIVINELIGHT_PORT")
                .ok()
                .and_then(|p| p.parse().ok())
                .unwrap_or(8080),
            log_level: std::env::var("RUST_LOG")
                .unwrap_or_else(|_| "divinelight=info,tower_http=info".to_string()),
        }
    }

    pub fn socket_addr(&self) -> std::net::SocketAddr {
        format!("{}:{}", self.host, self.port)
            .parse()
            .expect("Invalid address")
    }
}
```

**File: `src/lib.rs`**
Add the config module:
```rust
pub mod config;
```

**File: `src/main.rs`**
Replace the hardcoded `data_dir` and address construction with `Config`:
```rust
use divinelight::config::Config; // or use crate::config::Config if in same crate

// In main():
let config = Config::from_env();
// Replace all uses of data_dir with config.data_dir.clone()
// Replace the SocketAddr::from(([127, 0, 0, 1], 8080)) with config.socket_addr()
```

---

## STEP 10 — Add `.env.example` File

Create `.env.example` in the repository root:

```
# DivineLight Environment Configuration

# Directory where all data is stored (memories, graph DB, retrieval DB)
# Default: platform data dir (e.g., ~/Library/Application Support/divinelight on macOS)
DIVINELIGHT_DATA_DIR=./data/divinelight

# Server bind address
DIVINELIGHT_HOST=127.0.0.1

# Server port
DIVINELIGHT_PORT=8080

# Log level (trace, debug, info, warn, error)
RUST_LOG=divinelight=info,tower_http=info
```

---

## STEP 11 — Expand the Test Suite

**File: `src/tests.rs`**

Add the following test blocks after the existing tests:

```rust
#[test]
fn test_memory_store_ingest_and_get() {
    let dir = TempDir::new().unwrap();
    let mut store = crate::storage::MemoryStore::new(dir.path().to_path_buf()).unwrap();
    
    let memory = store.ingest(
        "test_source".to_string(),
        "plaintext".to_string(),
        "Hello world".to_string(),
        vec!["test".to_string()],
    ).unwrap();
    
    let retrieved = store.get(&memory.memory_id).unwrap();
    assert_eq!(retrieved.content, "Hello world");
    assert_eq!(retrieved.source, "test_source");
    assert!(retrieved.verify());
}

#[test]
fn test_memory_store_count() {
    let dir = TempDir::new().unwrap();
    let mut store = crate::storage::MemoryStore::new(dir.path().to_path_buf()).unwrap();
    assert_eq!(store.count().unwrap(), 0);
    
    store.ingest("s".to_string(), "p".to_string(), "c1".to_string(), vec![]).unwrap();
    store.ingest("s".to_string(), "p".to_string(), "c2".to_string(), vec![]).unwrap();
    
    assert_eq!(store.count().unwrap(), 2);
}

#[test]
fn test_graph_store_create_node_and_edge() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();
    
    let node_a = store.create_node(
        "Person".to_string(),
        "Alice".to_string(),
        serde_json::json!({}),
        vec![],
    ).unwrap();
    
    let node_b = store.create_node(
        "Person".to_string(),
        "Bob".to_string(),
        serde_json::json!({}),
        vec![],
    ).unwrap();
    
    let edge = store.create_edge(
        node_a.id.clone(),
        node_b.id.clone(),
        "knows".to_string(),
        serde_json::json!({}),
        vec![],
        0.9,
    ).unwrap();
    
    assert_eq!(edge.source, node_a.id);
    assert_eq!(edge.target, node_b.id);
    
    let meta = store.get_metadata().unwrap();
    assert_eq!(meta.node_count, 2);
    assert_eq!(meta.edge_count, 1);
}

#[test]
fn test_graph_store_bfs_traversal() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();
    
    let a = store.create_node("T".to_string(), "A".to_string(), serde_json::json!({}), vec![]).unwrap();
    let b = store.create_node("T".to_string(), "B".to_string(), serde_json::json!({}), vec![]).unwrap();
    let c = store.create_node("T".to_string(), "C".to_string(), serde_json::json!({}), vec![]).unwrap();
    
    store.create_edge(a.id.clone(), b.id.clone(), "r".to_string(), serde_json::json!({}), vec![], 1.0).unwrap();
    store.create_edge(b.id.clone(), c.id.clone(), "r".to_string(), serde_json::json!({}), vec![], 1.0).unwrap();
    
    let traversed = store.traverse_bfs(&a.id, 2).unwrap();
    assert!(traversed.len() >= 3);
}

#[test]
fn test_graph_store_find_path() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();
    
    let a = store.create_node("T".to_string(), "A".to_string(), serde_json::json!({}), vec![]).unwrap();
    let b = store.create_node("T".to_string(), "B".to_string(), serde_json::json!({}), vec![]).unwrap();
    
    store.create_edge(a.id.clone(), b.id.clone(), "r".to_string(), serde_json::json!({}), vec![], 1.0).unwrap();
    
    let path = store.find_path(&a.id, &b.id, 5).unwrap();
    assert!(path.is_some());
    
    let no_path = store.find_path(&b.id, &a.id, 5).unwrap();
    // Directed graph — no reverse path
    assert!(no_path.is_none());
}

#[test]
fn test_retrieval_engine_indexing_and_search() {
    let dir = TempDir::new().unwrap();
    let engine = crate::retrieval::RetrievalEngine::new(dir.path().to_path_buf()).unwrap();
    
    let memory = crate::models::MemoryObject::new(
        "test".to_string(),
        "plaintext".to_string(),
        "The quick brown fox".to_string(),
        vec!["animals".to_string()],
    );
    
    engine.index_memory(&memory).unwrap();
    
    // Save memory file so load_memory can find it
    std::fs::create_dir_all(dir.path().join("memories")).unwrap();
    let path = dir.path().join("memories").join(format!("{}.json", memory.memory_id));
    std::fs::write(&path, serde_json::to_string(&memory).unwrap()).unwrap();
    
    let results = engine.search("fox", 10).unwrap();
    assert!(!results.is_empty());
    assert!(results[0].score > 0.0);
}

#[test]
fn test_retrieval_wildcard_returns_all() {
    let dir = TempDir::new().unwrap();
    let engine = crate::retrieval::RetrievalEngine::new(dir.path().to_path_buf()).unwrap();
    
    let m1 = crate::models::MemoryObject::new("s".to_string(), "p".to_string(), "content one".to_string(), vec![]);
    let m2 = crate::models::MemoryObject::new("s".to_string(), "p".to_string(), "content two".to_string(), vec![]);
    
    for m in [&m1, &m2] {
        engine.index_memory(m).unwrap();
        std::fs::create_dir_all(dir.path().join("memories")).unwrap();
        let path = dir.path().join("memories").join(format!("{}.json", m.memory_id));
        std::fs::write(&path, serde_json::to_string(m).unwrap()).unwrap();
    }
    
    let results = engine.search("*", 100).unwrap();
    assert_eq!(results.len(), 2);
}

#[test]
fn test_synthesizer_agent_empty_input() {
    use crate::agents::SynthesizerAgent;
    let agent = SynthesizerAgent::new();
    let output = agent.execute(vec![]).unwrap();
    assert_eq!(output.outputs[0].metadata.explanation, "No content to synthesize");
}

#[test]
fn test_verifier_agent_detects_tampered_memory() {
    use crate::agents::VerifierAgent;
    let mut memory = crate::models::MemoryObject::new(
        "test".to_string(),
        "plaintext".to_string(),
        "Original content".to_string(),
        vec![],
    );
    // Tamper with content without updating checksum
    memory.content = "Tampered content".to_string();
    
    let agent = VerifierAgent::new();
    let output = agent.execute(vec![&memory]).unwrap();
    assert_eq!(output.outputs[0].score, 0.0);
    assert!(output.outputs[0].metadata.explanation.contains("failed"));
}
```

---

## STEP 12 — Rewrite `README.md`

Replace the entire `README.md` with the following:

```markdown
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
```

---

## STEP 13 — Create `docs/architecture.md`

Create `docs/architecture.md`:

```markdown
# Architecture

## Component Graph

```
┌─────────────────────────────────────────────────────────────┐
│                         REST API Layer                       │
│                    (Axum, src/api/mod.rs)                   │
└──────────────┬───────────────────┬──────────────────────────┘
               │                   │
    ┌──────────▼──────────┐  ┌────▼──────────────────┐
    │   MemoryStore        │  │     GraphStore         │
    │  (src/storage/       │  │  (src/storage/         │
    │   memory.rs)         │  │   graph.rs)            │
    │                      │  │                        │
    │ Inputs:  source,     │  │ Inputs: node/edge      │
    │          content,    │  │         schemas        │
    │          tags        │  │ Outputs: GraphNode,    │
    │ Outputs: MemoryObject│  │          GraphEdge     │
    │ Side FX: writes JSON │  │ Side FX: writes SQLite │
    │          + SQLite    │  └────────────────────────┘
    └──────────┬───────────┘
               │
    ┌──────────▼───────────┐
    │   RetrievalEngine    │
    │ (src/retrieval/      │
    │  mod.rs)             │
    │                      │
    │ Inputs:  query,      │
    │          limit       │
    │ Outputs: Vec<        │
    │   RetrievalResult>   │
    │ Side FX: writes to   │
    │   retrieval.db       │
    └──────────┬───────────┘
               │
    ┌──────────▼───────────┐
    │  ReasoningEngine     │
    │ (src/reasoning/      │
    │  mod.rs)             │
    │                      │
    │ Inputs: query +      │
    │         RetrievalResults│
    │ Outputs: BeliefState,│
    │   Interpretations    │
    │ Side FX: none        │
    └──────────┬───────────┘
               │
    ┌──────────▼───────────┐
    │      Agents          │
    │  (src/agents/        │
    │   mod.rs)            │
    │                      │
    │ - RetrieverAgent     │
    │ - VerifierAgent      │
    │ - SynthesizerAgent   │
    │ - ContradictionDetector│
    │                      │
    │ Inputs: memories,    │
    │         results      │
    │ Outputs: AgentOutput │
    │ Side FX: none        │
    └──────────────────────┘
```

## Module Responsibilities

| Module | File | Inputs | Outputs | Side Effects |
|---|---|---|---|---|
| API Layer | `src/api/mod.rs` | HTTP requests | HTTP responses | Delegates to stores |
| Config | `src/config.rs` | Env vars | Config struct | None |
| MemoryStore | `src/storage/memory.rs` | source, content, tags | MemoryObject | Writes JSON + SQLite |
| GraphStore | `src/storage/graph.rs` | node/edge data | GraphNode, GraphEdge | Writes SQLite |
| RetrievalEngine | `src/retrieval/mod.rs` | query string | RetrievalResult[] | Writes retrieval.db |
| ReasoningEngine | `src/reasoning/mod.rs` | query + results | BeliefState | None |
| Agents | `src/agents/mod.rs` | memories/results | AgentOutput | None |
| BackupManager | `src/backup/mod.rs` | paths | manifests | Reads/writes files |

## Data Flow: Memory Ingestion

```
POST /api/v1/memory/ingest
        │
        ▼
  MemoryStore.ingest()
   - Generate memory_id (mp_YYYYMMDD_HHmmss_XXXXXXXX)
   - Compute SHA256 checksum
   - Write JSON to memories/{id}.json
   - Insert metadata row into divinelight.db
        │
        ▼
  RetrievalEngine.index_memory()
   - Insert into retrieval.db search_index table
        │
        ▼
  Return { memory_id, status: "created" }
```

## Data Flow: Retrieval

```
POST /api/v1/retrieve { query, limit }
        │
        ▼
  RetrievalEngine.search()
   - Scan search_index for keyword matches
   - Score each result
   - Load MemoryObject from JSON file
   - Sort by score descending
   - Truncate to limit
        │
        ▼
  Return { results: [{ memory, score, source, provenance, confidence }] }
```
```

---

## STEP 14 — Create `docs/data_flow.md`

Create `docs/data_flow.md`:

```markdown
# Data Flow

## Ingestion Pipeline

```
User Input
    │  source, format, content, tags
    ▼
POST /api/v1/memory/ingest
    │
    ├──▶ MemoryStore.ingest()
    │       ├── Generate: memory_id = mp_{timestamp}_{uuid8}
    │       ├── Compute:  checksum = sha256:{hex}
    │       ├── Write:    memories/{memory_id}.json (verbatim)
    │       └── Insert:   divinelight.db/memories row
    │
    └──▶ RetrievalEngine.index_memory()
            └── Insert: retrieval.db/search_index row
                        (memory_id, content, tags, source, created_at)
```

## Recall Pipeline

```
User Query
    │  query, limit
    ▼
POST /api/v1/retrieve
    │
    ▼
RetrievalEngine.search()
    ├── Scan:  search_index WHERE content/tags match query terms
    ├── Score: term frequency + tag bonus + length normalization
    ├── Load:  MemoryObject from memories/{id}.json
    └── Sort:  descending by score, truncate to limit

    ▼
Return: Vec<RetrievalResult>
    { memory, graph_node, graph_edge, score, source, provenance, confidence }
```

## Reasoning Pipeline

```
POST /api/v1/reason/interpret { query }
    │
    ├──▶ RetrievalEngine.search(query, 10)
    │        └── Vec<RetrievalResult>
    │
    └──▶ ReasoningEngine.interpret(query, results)
             ├── generate_interpretations() → Vec<Interpretation>
             ├── detect_conflicts()          → Vec<ConflictFlag>
             └── Build BeliefState { belief_id, interpretations, conflict_flags }

    ▼
Return: { belief_state, interpretations, provenance }
```

## Graph Pipeline

```
POST /api/v1/graph/nodes  →  GraphStore.create_node()  →  SQLite nodes table
POST /api/v1/graph/edges  →  GraphStore.create_edge()  →  SQLite edges table
POST /api/v1/graph/traverse  →  GraphStore.traverse_bfs()  →  Vec<GraphNode>
POST /api/v1/graph/path      →  GraphStore.find_path()     →  Option<Vec<String>>
```

## Backup Pipeline

```
POST /api/v1/backup/create { path }
    │
    ├── Copy memories/*.json  →  {path}/memories/
    ├── Copy graph.db         →  {path}/graph.db
    ├── Copy retrieval.db     →  {path}/retrieval.db
    └── Write manifest.json   →  {path}/manifest.json
        { version, created_at, memory_count, node_count, edge_count }
```
```

---

## STEP 15 — Create `docs/api_spec.md` (Formal Spec)

Create `docs/api_spec.md`:

```markdown
# API Specification v1

Base URL: `http://127.0.0.1:8080`

All request and response bodies are `application/json`.

---

## Health

### GET /health
**Response 200:**
```json
{ "status": "ok", "version": "0.1.0" }
```

---

## Memory

### POST /api/v1/memory/ingest
**Request:**
```json
{
  "source": "user_session",
  "format": "plaintext",
  "content": "Full verbatim memory content.",
  "tags": ["tag1", "tag2"]
}
```
**Response 200:**
```json
{ "memory_id": "mp_20260409_123456_abcd1234", "status": "created" }
```

### GET /api/v1/memory/:memory_id
**Response 200:** Full `MemoryObject`
```json
{
  "memory_id": "mp_20260409_123456_abcd1234",
  "created_at": "2026-04-09T12:34:56Z",
  "source": "user_session",
  "format": "plaintext",
  "content": "Full verbatim memory content.",
  "tags": ["tag1"],
  "checksum": "sha256:abc123...",
  "version": 1,
  "notes": ""
}
```
**Response 500:** Memory not found or checksum failure.

### GET /api/v1/memory/list?limit=50&offset=0
**Response 200:**
```json
{
  "memories": [ /* MemoryObject[] */ ],
  "total": 42
}
```

---

## Graph

### POST /api/v1/graph/nodes
**Request:**
```json
{
  "node_type": "Person",
  "label": "Alice",
  "properties": { "age": 30 },
  "provenance": ["mp_20260409_123456_abcd1234"]
}
```
**Response 200:** Full `GraphNode`

### GET /api/v1/graph/nodes/:node_id
**Response 200:** Full `GraphNode`

### POST /api/v1/graph/edges
**Request:**
```json
{
  "source": "node-uuid-a",
  "target": "node-uuid-b",
  "relation": "knows",
  "properties": {},
  "provenance": [],
  "confidence": 0.9
}
```
**Response 200:** Full `GraphEdge`

### GET /api/v1/graph/edges/:edge_id
**Response 200:** Full `GraphEdge`

### GET /api/v1/graph/metadata
**Response 200:**
```json
{
  "graph_id": "main",
  "schema_version": "1.0",
  "created_at": "...",
  "updated_at": "...",
  "node_count": 12,
  "edge_count": 8,
  "retention_policy": "retain_forever"
}
```

### POST /api/v1/graph/traverse
**Request:**
```json
{ "start_node_id": "node-uuid", "depth": 3 }
```
**Response 200:**
```json
{ "nodes": [ /* GraphNode[] */ ], "edges": [ /* GraphEdge[] */ ] }
```

### POST /api/v1/graph/path
**Request:**
```json
{ "start_id": "node-uuid-a", "end_id": "node-uuid-b", "max_depth": 5 }
```
**Response 200:**
```json
{ "path": ["node-uuid-a", "node-uuid-c", "node-uuid-b"] }
```
Or `{ "path": null }` if no path exists.

---

## Retrieval

### POST /api/v1/retrieve
**Request:**
```json
{ "query": "search terms or *", "limit": 10 }
```
**Response 200:**
```json
{
  "results": [
    {
      "memory": { /* MemoryObject or null */ },
      "graph_node": null,
      "graph_edge": null,
      "score": 0.87,
      "source": "memory",
      "provenance": ["mp_..."],
      "confidence": 0.87
    }
  ]
}
```

---

## Reasoning

### POST /api/v1/reason/interpret
**Request:**
```json
{ "query": "project status", "context_ids": [] }
```
**Response 200:**
```json
{
  "belief_state": {
    "belief_id": "belief_abcd1234",
    "timestamp": "...",
    "interpretations": [ /* Interpretation[] */ ],
    "conflict_flags": [ /* ConflictFlag[] */ ],
    "state": "open"
  },
  "interpretations": [ /* Interpretation[] */ ],
  "provenance": ["mp_..."]
}
```

### GET /api/v1/reason/beliefs/:belief_id
**Response 200:** Full `BeliefState`

---

## Agents

### POST /api/v1/agents/retriever
**Request:** `{ "query": "...", "limit": 10 }`
**Response 200:** `AgentOutput`

### POST /api/v1/agents/verifier
**Request:** `{ "memory_ids": ["mp_...", "mp_..."] }`
**Response 200:** `AgentOutput`

### POST /api/v1/agents/synthesizer
**Request:** `{ "query": "...", "limit": 10 }`
**Response 200:** `AgentOutput`

### POST /api/v1/agents/contradiction
**Request:** `{ "memory_ids": ["mp_...", "mp_..."] }`
**Response 200:** `AgentOutput`

---

## Backup

### POST /api/v1/backup/create
**Request:** `{ "path": "/absolute/or/relative/path" }`
**Response 200:** `{ "status": "success", "manifest": { ... } }`

### POST /api/v1/backup/restore
**Request:** `{ "path": "/path/to/backup" }`
**Response 200:** `{ "status": "success", "manifest": null }`

### POST /api/v1/backup/export
**Request:** `{ "path": "/path/to/export.jsonl" }`
**Response 200:** `{ "status": "success", "manifest": null }`

### POST /api/v1/backup/import
**Request:** `{ "path": "/path/to/import.jsonl" }`
**Response 200:** `{ "status": "Imported N memories", "manifest": null }`
```

---

## STEP 16 — Verify All Modules Are Exported in `src/lib.rs`

**File: `src/lib.rs`**

Ensure it reads exactly:
```rust
pub mod storage;
pub mod api;
pub mod models;
pub mod retrieval;
pub mod reasoning;
pub mod agents;
pub mod backup;
pub mod config;

#[cfg(test)]
mod tests;
```

---

## STEP 17 — Run Verification

After all changes, execute:

```bash
cargo build 2>&1
```

Fix any compilation errors before proceeding.

```bash
cargo test 2>&1
```

All tests must pass. If any test fails, fix the root cause — do not delete the test.

```bash
cargo clippy -- -D warnings 2>&1
```

Fix all clippy warnings. Common fixes:
- Replace `format!("{}", x)` with `x.to_string()` where `x: String`
- Remove unused imports
- Add `#[allow(dead_code)]` only where genuinely needed for future phases

---

## STEP 18 — Final File Checklist

Confirm the following files exist and are correct:

| File | Status |
|---|---|
| `src/main.rs` | Uses `Config`, no hardcoded addresses |
| `src/lib.rs` | Exports all 8 modules |
| `src/config.rs` | NEW — config from env vars |
| `src/api/mod.rs` | CORS, AppError, list endpoint, JSON health |
| `src/storage/memory.rs` | Unchanged except any clippy fixes |
| `src/storage/graph.rs` | `query_edges` signature fixed |
| `src/retrieval/mod.rs` | Wildcard `*` scoring fixed |
| `src/reasoning/mod.rs` | Unchanged |
| `src/agents/mod.rs` | Unchanged |
| `src/backup/mod.rs` | `count_lines` uses real SQLite query |
| `src/models/mod.rs` | Unused import removed |
| `src/tests.rs` | Expanded with 9 new tests |
| `src/cli.rs` | DELETED |
| `README.md` | Fully rewritten |
| `.env.example` | NEW |
| `docs/architecture.md` | NEW |
| `docs/data_flow.md` | NEW |
| `docs/api_spec.md` | NEW (replaces old api_spec.md) |
| `frontend/index.html` | `refreshStats` fixed |

---

## Completion Criteria

The implementation is complete when:
1. `cargo build` produces zero errors
2. `cargo test` shows all tests passing (minimum 15 tests)
3. `cargo clippy -- -D warnings` produces zero warnings
4. Starting the server with `cargo run` and opening `frontend/index.html` in a browser shows the status dot as green (Connected)
5. Ingesting a memory via the UI returns a `memory_id` and increments the Memories counter
6. The `/health` endpoint returns JSON `{"status":"ok","version":"0.1.0"}`