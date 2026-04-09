Here is the full coding agent prompt. It is deeply detailed, ordered by severity, and covers every bug, architectural flaw, and UX deficiency found in the codebase.

---

## DivineLight — Comprehensive Fix & Improvement Agent Prompt

You are an autonomous coding agent working on **DivineLight**, a local-first unified AI memory system written in Rust with an Axum HTTP backend and a plain HTML/JS frontend. You have full read/write access to the repository. Execute every step below in order. Do not skip steps. For each file edit, re-read the file immediately before making the change.

---

## PART 1 — CRITICAL BUG FIXES (compilation-safe, logic-breaking)

---

### FIX 1 — `query_nodes` SQL Parameter Binding Bug

**File: `src/storage/graph.rs`**

The `query_nodes` method constructs SQL conditionally but always binds `limit` to `?2` even when the SQL only has one placeholder (`?1`) because the `node_type` branch is absent. This causes a rusqlite `InvalidParameterCount` error on every call without `node_type`.

Replace the entire `query_nodes` method with a version that builds parameters correctly:

```rust
pub fn query_nodes(
    &self,
    node_type: Option<&str>,
    _label_contains: Option<&str>,
    limit: usize,
) -> Result<Vec<GraphNode>> {
    let nodes: Vec<GraphNode> = if let Some(nt) = node_type {
        let mut stmt = self.db.prepare(
            "SELECT id, node_type, label, properties, provenance, version, created_at, updated_at
             FROM nodes WHERE node_type = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;
        stmt.query_map(params![nt, limit as i64], Self::map_node)?
            .filter_map(|r| r.ok())
            .collect()
    } else {
        let mut stmt = self.db.prepare(
            "SELECT id, node_type, label, properties, provenance, version, created_at, updated_at
             FROM nodes ORDER BY created_at DESC LIMIT ?1",
        )?;
        stmt.query_map(params![limit as i64], Self::map_node)?
            .filter_map(|r| r.ok())
            .collect()
    };
    Ok(nodes)
}
```

---

### FIX 2 — BFS Traversal Uses Stack (DFS) — Fix to True BFS

**File: `src/storage/graph.rs`**

Both `traverse_bfs` and `find_path` use `Vec::pop()` which is LIFO (depth-first). Change both to use `std::collections::VecDeque` for proper FIFO breadth-first traversal.

Add `use std::collections::VecDeque;` at the top of the file.

Replace `traverse_bfs`:

```rust
pub fn traverse_bfs(&self, start_node_id: &str, max_depth: usize) -> Result<Vec<GraphNode>> {
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: VecDeque<(String, usize)> = VecDeque::new();
    queue.push_back((start_node_id.to_string(), 0));
    let mut result = Vec::new();

    while let Some((node_id, depth)) = queue.pop_front() {
        if visited.contains(&node_id) || depth > max_depth {
            continue;
        }
        visited.insert(node_id.clone());

        if let Ok(node) = self.get_node(&node_id) {
            result.push(node);
        }

        if depth < max_depth {
            let neighbors = self.get_node_neighbors(&node_id, 100)?;
            for edge in neighbors {
                let next_node = if edge.source == node_id {
                    edge.target.clone()
                } else {
                    edge.source.clone()
                };
                if !visited.contains(&next_node) {
                    queue.push_back((next_node, depth + 1));
                }
            }
        }
    }
    Ok(result)
}
```

Replace `find_path`:

```rust
pub fn find_path(
    &self,
    start_id: &str,
    end_id: &str,
    max_depth: usize,
) -> Result<Option<Vec<String>>> {
    let mut visited: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut queue: VecDeque<(String, Vec<String>)> = VecDeque::new();
    queue.push_back((start_id.to_string(), vec![start_id.to_string()]));

    while let Some((node_id, path)) = queue.pop_front() {
        if visited.contains(&node_id) {
            continue;
        }
        visited.insert(node_id.clone());

        if node_id == end_id {
            return Ok(Some(path));
        }

        if path.len() >= max_depth {
            continue;
        }

        let neighbors = self.get_node_neighbors(&node_id, 100)?;
        for edge in neighbors {
            // Only follow directed edges outward from current node
            if edge.source != node_id {
                continue;
            }
            let next_node = edge.target.clone();
            if !visited.contains(&next_node) {
                let mut new_path = path.clone();
                new_path.push(next_node.clone());
                queue.push_back((next_node, new_path));
            }
        }
    }
    Ok(None)
}
```

---

### FIX 3 — Graph Node Deduplication (Duplicate Nodes Created on Every Ingest)

**File: `src/storage/graph.rs`**

There is no UNIQUE constraint on `(node_type, label)`, and `extract_and_create_graph_nodes` in `src/api/mod.rs` calls `create_node` multiple times for the same concept — once in the `top_words` loop and again inside the edge-creation double loop. Each ingest of similar content therefore creates hundreds of duplicate nodes.

**Step A** — Add a UNIQUE index in `GraphStore::new`:

In the `GraphStore::new` function, after the existing index creation calls, add:

```rust
db.execute(
    "CREATE UNIQUE INDEX IF NOT EXISTS idx_node_label_type ON nodes(node_type, label)",
    [],
)?;
```

**Step B** — Add a `get_or_create_node` method to `GraphStore`:

```rust
pub fn get_or_create_node(
    &self,
    node_type: String,
    label: String,
    properties: serde_json::Value,
    provenance: Vec<String>,
) -> Result<GraphNode> {
    // Check if a node with this type+label already exists
    let existing = self.db.query_row(
        "SELECT id, node_type, label, properties, provenance, version, created_at, updated_at
         FROM nodes WHERE node_type = ?1 AND label = ?2",
        params![node_type, label],
        Self::map_node,
    );

    match existing {
        Ok(node) => Ok(node),
        Err(_) => self.create_node(node_type, label, properties, provenance),
    }
}
```

**Step C** — Rewrite `extract_and_create_graph_nodes` in `src/api/mod.rs` to use `get_or_create_node` and avoid double-creation:

Replace the entire `extract_and_create_graph_nodes` function:

```rust
fn extract_and_create_graph_nodes(graph: &mut GraphStore, memory: &MemoryObject) -> Result<(), AppError> {
    let content = &memory.content;

    // Count significant word frequencies
    let mut word_freq: std::collections::HashMap<String, usize> = std::collections::HashMap::new();
    for word in content.split(|c: char| !c.is_alphanumeric()) {
        let w = word.to_lowercase();
        if w.len() > 4 && !STOP_WORDS.contains(&w.as_str()) {
            *word_freq.entry(w).or_insert(0) += 1;
        }
    }

    // Only extract concepts appearing at least twice; cap at 8
    let mut top_words: Vec<String> = word_freq
        .into_iter()
        .filter(|(_, count)| *count >= 2)
        .map(|(w, _)| w)
        .collect();
    top_words.truncate(8);

    if top_words.is_empty() {
        return Ok(());
    }

    // Create (or retrieve existing) nodes — never create duplicates
    let mut node_ids: Vec<(String, String)> = Vec::new(); // (label, node_id)
    for concept in &top_words {
        let properties = serde_json::json!({
            "source": "auto_extracted",
            "context": format!("Extracted from {}", memory.source)
        });
        match graph.get_or_create_node(
            "concept".to_string(),
            concept.clone(),
            properties,
            vec![memory.memory_id.clone()],
        ) {
            Ok(node) => node_ids.push((concept.clone(), node.id)),
            Err(e) => tracing::warn!("Failed to get/create node for '{}': {}", concept, e),
        }
    }

    // Create edges between co-occurring concepts (with dedup via INSERT OR IGNORE)
    for i in 0..node_ids.len() {
        for j in (i + 1)..node_ids.len() {
            let _ = graph.create_edge_if_absent(
                node_ids[i].1.clone(),
                node_ids[j].1.clone(),
                "related_to".to_string(),
                serde_json::json!({ "source": "auto_extracted", "memory_id": memory.memory_id }),
                vec![memory.memory_id.clone()],
                0.5,
            );
        }
    }

    Ok(())
}
```

**Step D** — Add `create_edge_if_absent` to `GraphStore` in `src/storage/graph.rs`:

```rust
/// Creates an edge only if one with the same source+target+relation does not exist.
pub fn create_edge_if_absent(
    &self,
    source: String,
    target: String,
    relation: String,
    properties: serde_json::Value,
    provenance: Vec<String>,
    confidence: f64,
) -> Result<GraphEdge> {
    // Check for existing edge
    let existing: rusqlite::Result<String> = self.db.query_row(
        "SELECT id FROM edges WHERE source = ?1 AND target = ?2 AND relation = ?3",
        params![source, target, relation],
        |row| row.get(0),
    );

    if let Ok(id) = existing {
        return self.get_edge(&id);
    }

    self.create_edge(source, target, relation, properties, provenance, confidence)
}
```

---

### FIX 4 — `traverse_graph` Handler Returns Duplicate Edges

**File: `src/api/mod.rs`**

The `traverse_graph` handler collects edges via `flat_map` over all nodes, so every edge incident to multiple traversed nodes is returned multiple times.

Replace the handler body:

```rust
async fn traverse_graph(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TraverseRequest>,
) -> Result<AxumJson<TraverseResponse>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let depth = req.depth.unwrap_or(3);
    let start_node_id = req.start_node_id.unwrap_or_default();

    let nodes = if !start_node_id.is_empty() {
        graph.traverse_bfs(&start_node_id, depth)?
    } else {
        graph.query_nodes(None, None, 100)?
    };

    // Collect edges and deduplicate by id
    let mut seen_edges: std::collections::HashSet<String> = std::collections::HashSet::new();
    let mut edges: Vec<GraphEdge> = Vec::new();
    for node in &nodes {
        for edge in graph.get_node_neighbors(&node.id, depth).unwrap_or_default() {
            if seen_edges.insert(edge.id.clone()) {
                edges.push(edge);
            }
        }
    }

    Ok(AxumJson(TraverseResponse { nodes, edges }))
}
```

---

### FIX 5 — `import_data` Does Not Re-Index into Retrieval DB

**File: `src/backup/mod.rs`**

After importing, memories exist as `.json` files but are not indexed in `retrieval.db`, making them unsearchable. The `BackupManager` needs a reference to the retrieval engine path, or the import must write directly to the retrieval index.

Replace `import_data` with a version that opens the retrieval DB and inserts index rows:

```rust
pub fn import_data(&self, import_path: &Path) -> Result<u64> {
    let mut count = 0u64;
    let content = fs::read_to_string(import_path)?;

    fs::create_dir_all(self.data_dir.join("memories"))?;

    // Open retrieval DB for re-indexing
    let retrieval_db_path = self.data_dir.join("retrieval.db");
    let retrieval_conn = rusqlite::Connection::open(&retrieval_db_path)?;
    retrieval_conn.execute(
        "CREATE TABLE IF NOT EXISTS search_index (
            memory_id TEXT PRIMARY KEY,
            content TEXT NOT NULL,
            tags TEXT NOT NULL,
            source TEXT NOT NULL,
            created_at TEXT NOT NULL
        )",
        [],
    )?;

    for line in content.lines() {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(memory) = serde_json::from_str::<MemoryObject>(line) {
            // Write JSON file
            let file_path = self
                .data_dir
                .join("memories")
                .join(format!("{}.json", memory.memory_id));
            let mut file = fs::File::create(&file_path)?;
            file.write_all(line.as_bytes())?;

            // Re-index in retrieval DB
            let tags_json = serde_json::to_string(&memory.tags).unwrap_or_else(|_| "[]".to_string());
            let _ = retrieval_conn.execute(
                "INSERT OR REPLACE INTO search_index (memory_id, content, tags, source, created_at)
                 VALUES (?1, ?2, ?3, ?4, ?5)",
                rusqlite::params![
                    memory.memory_id,
                    memory.content,
                    tags_json,
                    memory.source,
                    memory.created_at.to_rfc3339(),
                ],
            );

            count += 1;
        }
    }

    Ok(count)
}
```

---

### FIX 6 — `count_lines` Parameter Type Should Use `&Path` Not `&PathBuf`

**File: `src/backup/mod.rs`**

Change the signature of `count_lines` for idiomatic Rust:

```rust
fn count_lines(&self, db_path: &Path, query: &str) -> Result<u64> {
```

Update the two call sites from `&backup_db` (which is already a `PathBuf`) — since `&PathBuf` coerces to `&Path` this requires no other changes, but clippy will now accept it.

---

### FIX 7 — `ingest_memory` Holds Two Mutex Locks Simultaneously (Deadlock Risk)

**File: `src/api/mod.rs`**

The `ingest_memory` handler acquires `state.memory` lock, then while still holding it acquires `state.retrieval` and `state.graph` locks. If any of those locks are poisoned or there is any ordering inconsistency across handlers, this can deadlock. Drop each lock before acquiring the next by collecting the values you need.

Replace the `ingest_memory` handler:

```rust
async fn ingest_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<AxumJson<IngestResponse>, AppError> {
    // Validate inputs
    if req.content.trim().is_empty() {
        return Err(AppError(anyhow::anyhow!("Content cannot be empty")));
    }
    if req.source.trim().is_empty() {
        return Err(AppError(anyhow::anyhow!("Source cannot be empty")));
    }

    // Step 1: ingest into memory store, then release lock
    let memory_obj = {
        let mut memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        memory.ingest(req.source.clone(), req.format, req.content.clone(), req.tags.clone())?
    };

    // Step 2: index in retrieval engine, then release lock
    {
        let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        retrieval.index_memory(&memory_obj)?;
    }

    // Step 3: extract concepts into graph, then release lock
    {
        let mut graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        extract_and_create_graph_nodes(&mut graph, &memory_obj)?;
    }

    Ok(AxumJson(IngestResponse {
        memory_id: memory_obj.memory_id,
        status: "created".to_string(),
    }))
}
```

Apply the same sequential lock-drop pattern to `run_verifier`, `run_contradiction_detector`, and `detect_conflicts` — do not hold `state.memory` lock while calling agent `execute`.

---

## PART 2 — ARCHITECTURE & DATA INTEGRITY IMPROVEMENTS

---

### IMPROVEMENT 1 — Add Unique Constraint on Retrieval Index

**File: `src/retrieval/mod.rs`**

The `search_index` table has `memory_id TEXT PRIMARY KEY` which is good, but `index_memory` uses `INSERT OR REPLACE` which silently drops and re-inserts, resetting any future extended fields. Change to `INSERT OR IGNORE` so re-indexing an existing memory is a no-op unless content changes:

```rust
pub fn index_memory(&self, memory: &MemoryObject) -> Result<()> {
    self.db.execute(
        "INSERT OR IGNORE INTO search_index (memory_id, content, tags, source, created_at)
         VALUES (?1, ?2, ?3, ?4, ?5)",
        params![
            memory.memory_id,
            memory.content.clone(),
            serde_json::to_string(&memory.tags)?,
            memory.source.clone(),
            memory.created_at.to_rfc3339(),
        ],
    )?;
    Ok(())
}
```

---

### IMPROVEMENT 2 — Add `GraphStore::count_nodes` and `count_edges` Methods

**File: `src/storage/graph.rs`**

The metadata table is updated via `update_metadata_count` on every write, but this can drift if a write fails mid-transaction. Add live-count methods:

```rust
pub fn count_nodes(&self) -> Result<u64> {
    let count: i64 = self.db.query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
    Ok(count as u64)
}

pub fn count_edges(&self) -> Result<u64> {
    let count: i64 = self.db.query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
    Ok(count as u64)
}
```

Update `get_metadata` to return live counts rather than relying on the cached counter in the metadata table:

```rust
pub fn get_metadata(&self) -> Result<GraphMetadata> {
    let mut meta = self.db.query_row(
        "SELECT graph_id, schema_version, created_at, updated_at, node_count, edge_count, retention_policy
         FROM metadata WHERE graph_id = 'main'",
        [],
        |row: &rusqlite::Row| {
            Ok(GraphMetadata {
                graph_id: row.get(0)?,
                schema_version: row.get(1)?,
                created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?)
                    .unwrap().with_timezone(&chrono::Utc),
                updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?)
                    .unwrap().with_timezone(&chrono::Utc),
                node_count: row.get(4)?,
                edge_count: row.get(5)?,
                retention_policy: row.get(6)?,
            })
        },
    ).map_err(|e| anyhow::anyhow!("Metadata not found: {}", e))?;

    // Always return live counts to avoid drift
    meta.node_count = self.count_nodes()?;
    meta.edge_count = self.count_edges()?;
    Ok(meta)
}
```

---

### IMPROVEMENT 3 — Add `GET /api/v1/graph/nodes` Endpoint (List Nodes)

**File: `src/api/mod.rs`**

There is no way to list graph nodes from the frontend. Add:

In `create_router`, add:
```rust
.route("/api/v1/graph/nodes/list", get(list_nodes))
```

Add handler:
```rust
#[derive(Debug, Deserialize)]
struct ListNodesQuery {
    node_type: Option<String>,
    limit: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ListNodesResponse {
    nodes: Vec<GraphNode>,
    total: u64,
}

async fn list_nodes(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<ListNodesQuery>,
) -> Result<AxumJson<ListNodesResponse>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let limit = params.limit.unwrap_or(100);
    let nodes = graph.query_nodes(params.node_type.as_deref(), None, limit)?;
    let total = graph.count_nodes()?;
    Ok(AxumJson(ListNodesResponse { nodes, total }))
}
```

---

### IMPROVEMENT 4 — Add `GET /api/v1/memory/search` GET Endpoint for URL-based search

**File: `src/api/mod.rs`**

Add to the router:
```rust
.route("/api/v1/memory/search", get(search_memories_get))
```

Add handler so the frontend can use simple GET requests for search:
```rust
#[derive(Debug, Deserialize)]
struct SearchQuery {
    q: Option<String>,
    limit: Option<usize>,
}

async fn search_memories_get(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<SearchQuery>,
) -> Result<AxumJson<RetrieveResponse>, AppError> {
    let query = params.q.unwrap_or_else(|| "*".to_string());
    let limit = params.limit.unwrap_or(20);
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let results = retrieval.search(&query, limit)?;
    Ok(AxumJson(RetrieveResponse { results }))
}
```

---

### IMPROVEMENT 5 — Expose `MemoryStore::delete` and `GraphStore::delete_node`

**File: `src/storage/memory.rs`**

Add a soft-delete (or hard delete) to allow the frontend to remove memories. Hard delete is appropriate for a local-first system:

```rust
pub fn delete(&mut self, memory_id: &str) -> Result<()> {
    // Remove the JSON file
    let file_path = self
        .data_dir
        .join("memories")
        .join(format!("{}.json", memory_id));
    if file_path.exists() {
        std::fs::remove_file(&file_path)?;
    }
    // Remove from index
    self.db.execute("DELETE FROM memories WHERE memory_id = ?1", params![memory_id])?;
    self.db.execute("DELETE FROM tags WHERE memory_id = ?1", params![memory_id])?;
    Ok(())
}
```

**File: `src/storage/graph.rs`**

```rust
pub fn delete_node(&self, node_id: &str) -> Result<()> {
    self.db.execute("DELETE FROM edges WHERE source = ?1 OR target = ?1", params![node_id])?;
    self.db.execute("DELETE FROM nodes WHERE id = ?1", params![node_id])?;
    self.update_metadata_count()?;
    Ok(())
}
```

**File: `src/api/mod.rs`**

Add to router:
```rust
.route("/api/v1/memory/:memory_id", axum::routing::delete(delete_memory))
.route("/api/v1/graph/nodes/:node_id", axum::routing::delete(delete_node))
```

Add `use axum::routing::delete as axum_delete;` (or inline the delete routes). Add handlers:

```rust
async fn delete_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<String>,
) -> Result<AxumJson<serde_json::Value>, AppError> {
    // Also remove from retrieval index
    {
        let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        retrieval.delete_from_index(&memory_id)?;
    }
    let mut memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    memory.delete(&memory_id)?;
    Ok(AxumJson(serde_json::json!({ "status": "deleted", "memory_id": memory_id })))
}

async fn delete_node_handler(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<AxumJson<serde_json::Value>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    graph.delete_node(&node_id)?;
    Ok(AxumJson(serde_json::json!({ "status": "deleted", "node_id": node_id })))
}
```

**File: `src/retrieval/mod.rs`**

Add:
```rust
pub fn delete_from_index(&self, memory_id: &str) -> Result<()> {
    self.db.execute("DELETE FROM search_index WHERE memory_id = ?1", params![memory_id])?;
    Ok(())
}
```

---

### IMPROVEMENT 6 — Add Request Size and Content Length Guard

**File: `src/api/mod.rs`**

Very large content strings can lock the server for extended periods during SHA256 computation and graph extraction. Add a content size limit in `ingest_memory` after the empty-check:

```rust
const MAX_CONTENT_BYTES: usize = 1_000_000; // 1MB
if req.content.len() > MAX_CONTENT_BYTES {
    return Err(AppError(anyhow::anyhow!(
        "Content too large: {} bytes (max {})",
        req.content.len(),
        MAX_CONTENT_BYTES
    )));
}
```

---

## PART 3 — COMPLETE FRONTEND REWRITE

**File: `frontend/index.html`**

The current frontend has several critical issues:
- The `call()` function routes all API errors to `systemResult`, polluting the health panel with unrelated errors.
- `showGraphTab` and `showRetrieveTab` use the deprecated `event` global object — they fail silently in strict contexts.
- Tab switching does not remove `active` class from siblings correctly across panels.
- There is no way to browse all memories, view memory details, or delete memories.
- The agent panel for verifier/contradiction provides no way to select which memory IDs to use.
- The stats display shows "—" for beliefs with no explanation.
- There are no loading states — the user sees no feedback while API calls are in flight.
- The graph traverse panel has no meaningful output for the default "show all" case.

Replace the **entire** `frontend/index.html` with the following. This is a complete, self-contained rewrite with all functionality preserved and improved:

```html
<!DOCTYPE html>
<html lang="en">
<head>
  <meta charset="UTF-8">
  <meta name="viewport" content="width=device-width, initial-scale=1.0">
  <title>DivineLight — Memory System</title>
  <style>
    *, *::before, *::after { box-sizing: border-box; margin: 0; padding: 0; }

    :root {
      --bg:       #0d1117;
      --surface:  #161b22;
      --border:   #30363d;
      --border2:  #21262d;
      --text:     #c9d1d9;
      --muted:    #8b949e;
      --accent:   #58a6ff;
      --green:    #3fb950;
      --red:      #f85149;
      --orange:   #d29922;
      --radius:   6px;
    }

    body {
      font-family: -apple-system, BlinkMacSystemFont, 'Segoe UI', Roboto, sans-serif;
      background: var(--bg);
      color: var(--text);
      min-height: 100vh;
      font-size: 14px;
      line-height: 1.5;
    }

    /* ── Layout ── */
    .app { display: grid; grid-template-rows: 56px 1fr; height: 100vh; }

    .topbar {
      background: var(--surface);
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: center;
      padding: 0 20px;
      gap: 16px;
    }
    .topbar h1 { font-size: 1.1rem; color: var(--accent); font-weight: 700; letter-spacing: -0.3px; }
    .topbar .status { display: flex; align-items: center; gap: 6px; font-size: 0.8rem; color: var(--muted); margin-left: auto; }
    .dot { width: 7px; height: 7px; border-radius: 50%; background: var(--red); transition: background .3s; }
    .dot.ok { background: var(--green); }

    .main { display: grid; grid-template-columns: 320px 1fr; overflow: hidden; }

    .sidebar {
      border-right: 1px solid var(--border);
      display: flex;
      flex-direction: column;
      overflow: hidden;
    }

    .nav {
      display: flex;
      border-bottom: 1px solid var(--border);
      overflow-x: auto;
    }
    .nav-btn {
      flex: 1;
      padding: 10px 6px;
      background: none;
      border: none;
      border-bottom: 2px solid transparent;
      color: var(--muted);
      cursor: pointer;
      font-size: 0.78rem;
      font-weight: 500;
      white-space: nowrap;
      transition: color .2s, border-color .2s;
    }
    .nav-btn.active { color: var(--accent); border-bottom-color: var(--accent); }
    .nav-btn:hover:not(.active) { color: var(--text); }

    .panel { display: none; flex-direction: column; gap: 12px; padding: 16px; overflow-y: auto; flex: 1; }
    .panel.active { display: flex; }

    .content-area { display: flex; flex-direction: column; overflow: hidden; }
    .content-pane { display: none; flex-direction: column; overflow: hidden; flex: 1; }
    .content-pane.active { display: flex; }

    /* ── Form controls ── */
    input[type=text], input[type=number], textarea, select {
      width: 100%;
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      color: var(--text);
      padding: 8px 10px;
      font-size: 13px;
      font-family: inherit;
      transition: border-color .15s;
    }
    input:focus, textarea:focus, select:focus { outline: none; border-color: var(--accent); }
    textarea { resize: vertical; min-height: 90px; }

    label { display: block; font-size: 0.75rem; color: var(--muted); margin-bottom: 4px; text-transform: uppercase; letter-spacing: 0.4px; }

    .field { display: flex; flex-direction: column; gap: 4px; }

    /* ── Buttons ── */
    button {
      background: #238636;
      color: #fff;
      border: none;
      border-radius: var(--radius);
      padding: 8px 14px;
      cursor: pointer;
      font-size: 13px;
      font-weight: 500;
      font-family: inherit;
      transition: background .15s, opacity .15s;
    }
    button:hover { background: #2ea043; }
    button:disabled { opacity: 0.45; cursor: not-allowed; }
    button.secondary { background: var(--border2); color: var(--text); }
    button.secondary:hover { background: var(--border); }
    button.danger { background: transparent; color: var(--red); border: 1px solid var(--border); }
    button.danger:hover { background: rgba(248,81,73,.1); border-color: var(--red); }
    button.small { padding: 5px 10px; font-size: 12px; }
    .btn-row { display: flex; gap: 8px; flex-wrap: wrap; align-items: center; }

    /* ── Stats ── */
    .stats-grid { display: grid; grid-template-columns: 1fr 1fr; gap: 8px; }
    .stat-card {
      background: var(--border2);
      border-radius: var(--radius);
      padding: 12px;
      text-align: center;
    }
    .stat-value { font-size: 1.4rem; font-weight: 700; color: var(--accent); }
    .stat-label { font-size: 0.72rem; color: var(--muted); margin-top: 2px; text-transform: uppercase; letter-spacing: 0.4px; }

    /* ── Result list ── */
    .result-list { display: flex; flex-direction: column; gap: 1px; overflow-y: auto; flex: 1; }
    .result-item {
      padding: 10px 12px;
      border-bottom: 1px solid var(--border2);
      cursor: pointer;
      transition: background .1s;
    }
    .result-item:hover { background: var(--border2); }
    .result-item.selected { background: rgba(88,166,255,.08); border-left: 2px solid var(--accent); }
    .result-title { font-weight: 500; color: var(--text); font-size: 13px; white-space: nowrap; overflow: hidden; text-overflow: ellipsis; }
    .result-meta { font-size: 0.75rem; color: var(--muted); margin-top: 3px; }
    .score-badge {
      display: inline-block;
      background: rgba(88,166,255,.15);
      color: var(--accent);
      border-radius: 3px;
      padding: 1px 5px;
      font-size: 0.72rem;
      font-weight: 600;
    }

    /* ── Detail view ── */
    .detail-header {
      padding: 16px 20px 12px;
      border-bottom: 1px solid var(--border);
      display: flex;
      align-items: flex-start;
      gap: 12px;
    }
    .detail-header-info { flex: 1; min-width: 0; }
    .detail-header h2 { font-size: 0.95rem; color: var(--text); word-break: break-all; }
    .detail-header .detail-id { font-size: 0.72rem; color: var(--muted); font-family: monospace; margin-top: 3px; }
    .detail-body { padding: 16px 20px; overflow-y: auto; flex: 1; display: flex; flex-direction: column; gap: 16px; }
    .detail-field { display: flex; flex-direction: column; gap: 6px; }
    .detail-field-label { font-size: 0.72rem; color: var(--muted); text-transform: uppercase; letter-spacing: 0.4px; font-weight: 600; }
    .detail-field-value { font-size: 13px; color: var(--text); white-space: pre-wrap; word-break: break-word; }
    .detail-content-box {
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 12px;
      font-size: 13px;
      white-space: pre-wrap;
      word-break: break-word;
      max-height: 320px;
      overflow-y: auto;
      line-height: 1.6;
    }
    .tags-row { display: flex; flex-wrap: wrap; gap: 6px; }
    .tag {
      display: inline-block;
      background: var(--border2);
      border: 1px solid var(--border);
      padding: 2px 8px;
      border-radius: 12px;
      font-size: 0.72rem;
      color: var(--muted);
    }
    .checksum { font-family: monospace; font-size: 0.72rem; color: var(--muted); word-break: break-all; }

    /* ── Empty / loading / error states ── */
    .empty-state {
      display: flex;
      flex-direction: column;
      align-items: center;
      justify-content: center;
      gap: 8px;
      padding: 40px 20px;
      color: var(--muted);
      font-size: 0.85rem;
      text-align: center;
      flex: 1;
    }
    .empty-icon { font-size: 2rem; opacity: 0.4; }
    .error-banner {
      background: rgba(248,81,73,.1);
      border: 1px solid rgba(248,81,73,.3);
      border-radius: var(--radius);
      color: var(--red);
      padding: 8px 12px;
      font-size: 12px;
    }
    .success-banner {
      background: rgba(63,185,80,.1);
      border: 1px solid rgba(63,185,80,.3);
      border-radius: var(--radius);
      color: var(--green);
      padding: 8px 12px;
      font-size: 12px;
    }
    .loading { color: var(--muted); font-size: 12px; padding: 12px 0; text-align: center; }
    .spinner { display: inline-block; width: 14px; height: 14px; border: 2px solid var(--border); border-top-color: var(--accent); border-radius: 50%; animation: spin .6s linear infinite; vertical-align: middle; margin-right: 6px; }
    @keyframes spin { to { transform: rotate(360deg); } }

    /* ── Graph ── */
    .graph-node-row {
      padding: 8px 12px;
      border-bottom: 1px solid var(--border2);
      display: flex;
      align-items: center;
      gap: 8px;
    }
    .node-type-badge {
      background: rgba(88,166,255,.12);
      color: var(--accent);
      border-radius: 3px;
      padding: 1px 6px;
      font-size: 0.7rem;
      font-weight: 600;
      white-space: nowrap;
    }
    .concept-badge {
      background: rgba(63,185,80,.12);
      color: var(--green);
      border-radius: 3px;
      padding: 1px 6px;
      font-size: 0.7rem;
      font-weight: 600;
    }

    /* ── Search bar ── */
    .search-bar { display: flex; gap: 6px; }
    .search-bar input { flex: 1; }

    /* ── Separator ── */
    hr { border: none; border-top: 1px solid var(--border2); }

    /* ── Code / monospace ── */
    pre {
      background: var(--bg);
      border: 1px solid var(--border);
      border-radius: var(--radius);
      padding: 10px;
      font-size: 11px;
      overflow-x: auto;
      white-space: pre-wrap;
      word-break: break-word;
      color: var(--muted);
      max-height: 200px;
      overflow-y: auto;
    }

    /* ── Responsive ── */
    @media (max-width: 800px) {
      .main { grid-template-columns: 1fr; }
      .sidebar { max-height: 55vh; border-right: none; border-bottom: 1px solid var(--border); }
    }
  </style>
</head>
<body>
<div class="app">

  <!-- Top bar -->
  <header class="topbar">
    <h1>⚡ DivineLight</h1>
    <span style="color:var(--muted);font-size:0.8rem;">Unified AI Memory System</span>
    <div class="status">
      <div class="dot" id="statusDot"></div>
      <span id="statusText">Connecting…</span>
      <button class="secondary small" onclick="refreshAll()">↻ Refresh</button>
    </div>
  </header>

  <div class="main">

    <!-- ══ SIDEBAR ══ -->
    <aside class="sidebar">
      <nav class="nav">
        <button class="nav-btn active" onclick="switchSidebar('memories', this)">Memories</button>
        <button class="nav-btn" onclick="switchSidebar('ingest', this)">+ Ingest</button>
        <button class="nav-btn" onclick="switchSidebar('graph', this)">Graph</button>
        <button class="nav-btn" onclick="switchSidebar('agents', this)">Agents</button>
        <button class="nav-btn" onclick="switchSidebar('backup', this)">Backup</button>
      </nav>

      <!-- ── MEMORIES PANEL ── -->
      <div class="panel active" id="panel-memories">
        <div class="stats-grid">
          <div class="stat-card"><div class="stat-value" id="s-memories">—</div><div class="stat-label">Memories</div></div>
          <div class="stat-card"><div class="stat-value" id="s-nodes">—</div><div class="stat-label">Nodes</div></div>
          <div class="stat-card"><div class="stat-value" id="s-edges">—</div><div class="stat-label">Edges</div></div>
          <div class="stat-card"><div class="stat-value" id="s-indexed">—</div><div class="stat-label">Indexed</div></div>
        </div>
        <div class="search-bar">
          <input type="text" id="memSearchInput" placeholder="Search memories…" oninput="debounceSearch(this.value)">
          <button class="secondary small" onclick="loadMemoryList()">All</button>
        </div>
        <div id="memListMsg"></div>
        <div class="result-list" id="memList">
          <div class="empty-state"><div class="empty-icon">🧠</div>No memories yet</div>
        </div>
      </div>

      <!-- ── INGEST PANEL ── -->
      <div class="panel" id="panel-ingest">
        <div class="field">
          <label>Source</label>
          <input type="text" id="inSource" placeholder="e.g. user_session, file.md">
        </div>
        <div class="field">
          <label>Format</label>
          <select id="inFormat">
            <option value="plaintext">Plaintext</option>
            <option value="markdown">Markdown</option>
            <option value="html">HTML</option>
            <option value="json">JSON</option>
          </select>
        </div>
        <div class="field">
          <label>Tags (comma-separated)</label>
          <input type="text" id="inTags" placeholder="e.g. ai, research, notes">
        </div>
        <div class="field">
          <label>Content</label>
          <textarea id="inContent" rows="8" placeholder="Paste or type memory content here…"></textarea>
        </div>
        <div id="ingestMsg"></div>
        <button onclick="ingest()">Ingest Memory</button>
      </div>

      <!-- ── GRAPH PANEL ── -->
      <div class="panel" id="panel-graph">
        <div>
          <label>Create Node</label>
          <div style="display:grid;gap:6px;margin-top:4px;">
            <input type="text" id="gNodeType" placeholder="Type (e.g. Person, Concept)">
            <input type="text" id="gNodeLabel" placeholder="Label">
            <button onclick="createGraphNode()" class="secondary">Create Node</button>
          </div>
        </div>
        <hr>
        <div>
          <label>Create Edge</label>
          <div style="display:grid;gap:6px;margin-top:4px;">
            <input type="text" id="gEdgeSrc" placeholder="Source Node ID">
            <input type="text" id="gEdgeTgt" placeholder="Target Node ID">
            <input type="text" id="gEdgeRel" placeholder="Relation (e.g. knows)">
            <input type="number" id="gEdgeConf" placeholder="Confidence" value="0.9" step="0.1" min="0" max="1">
            <button onclick="createGraphEdge()" class="secondary">Create Edge</button>
          </div>
        </div>
        <hr>
        <div>
          <label>Find Path</label>
          <div style="display:grid;gap:6px;margin-top:4px;">
            <input type="text" id="gPathSrc" placeholder="Start Node ID">
            <input type="text" id="gPathTgt" placeholder="End Node ID">
            <input type="number" id="gPathDepth" placeholder="Max depth" value="5">
            <button onclick="findGraphPath()" class="secondary">Find Path</button>
          </div>
        </div>
        <hr>
        <div>
          <label>Traverse from Node</label>
          <div style="display:grid;gap:6px;margin-top:4px;">
            <input type="text" id="gTraverseNode" placeholder="Start Node ID (blank = all)">
            <input type="number" id="gTraverseDepth" placeholder="Depth" value="3">
            <button onclick="traverseGraph()" class="secondary">Traverse</button>
          </div>
        </div>
        <div id="graphMsg"></div>
      </div>

      <!-- ── AGENTS PANEL ── -->
      <div class="panel" id="panel-agents">
        <div class="field">
          <label>Query (for Retriever / Synthesizer)</label>
          <input type="text" id="agQuery" placeholder="e.g. neural networks">
        </div>
        <div class="field">
          <label>Memory IDs (for Verifier / Contradiction — comma-separated)</label>
          <input type="text" id="agMemIds" placeholder="mp_..., mp_...">
        </div>
        <div class="btn-row">
          <button class="secondary" onclick="runAgent('retriever')">Retriever</button>
          <button class="secondary" onclick="runAgent('synthesizer')">Synthesizer</button>
          <button class="secondary" onclick="runAgent('verifier')">Verifier</button>
          <button class="secondary" onclick="runAgent('contradiction')">Contradiction</button>
        </div>
        <div style="font-size:0.72rem;color:var(--muted);">
          Tip: open a memory, copy its ID into Memory IDs above, then run Verifier or Contradiction.
        </div>
        <div id="agentsMsg"></div>
      </div>

      <!-- ── BACKUP PANEL ── -->
      <div class="panel" id="panel-backup">
        <p style="font-size:0.8rem;color:var(--muted);">All paths are relative to the server working directory or absolute.</p>
        <div class="field">
          <label>Backup / Restore Path</label>
          <input type="text" id="bkPath" placeholder="./backups/my-backup">
        </div>
        <div class="btn-row">
          <button onclick="doBackup('create')">Create Backup</button>
          <button class="secondary" onclick="doBackup('restore')">Restore</button>
        </div>
        <hr>
        <div class="field">
          <label>Export / Import Path (JSONL)</label>
          <input type="text" id="bkExportPath" placeholder="./export.jsonl">
        </div>
        <div class="btn-row">
          <button class="secondary" onclick="doBackup('export')">Export JSONL</button>
          <button class="secondary" onclick="doBackup('import')">Import JSONL</button>
        </div>
        <div id="backupMsg"></div>
      </div>

    </aside>

    <!-- ══ CONTENT AREA ══ -->
    <section class="content-area">

      <!-- Memory detail -->
      <div class="content-pane active" id="pane-memory-detail">
        <div class="empty-state" id="detailEmpty">
          <div class="empty-icon">🗄️</div>
          <div>Select a memory to view its details</div>
          <div style="font-size:0.75rem;">Use the search bar or browse the list on the left</div>
        </div>
        <div id="detailView" style="display:none;flex-direction:column;flex:1;overflow:hidden;">
          <div class="detail-header">
            <div class="detail-header-info">
              <h2 id="dSource">—</h2>
              <div class="detail-id" id="dId">—</div>
            </div>
            <div class="btn-row">
              <button class="secondary small" onclick="copyId()">Copy ID</button>
              <button class="secondary small" onclick="useInAgents()">Use in Agents</button>
              <button class="danger small" onclick="deleteCurrentMemory()">Delete</button>
            </div>
          </div>
          <div class="detail-body">
            <div class="detail-field">
              <div class="detail-field-label">Content</div>
              <div class="detail-content-box" id="dContent">—</div>
            </div>
            <div style="display:grid;grid-template-columns:1fr 1fr;gap:16px;">
              <div class="detail-field">
                <div class="detail-field-label">Format</div>
                <div class="detail-field-value" id="dFormat">—</div>
              </div>
              <div class="detail-field">
                <div class="detail-field-label">Created</div>
                <div class="detail-field-value" id="dCreated">—</div>
              </div>
            </div>
            <div class="detail-field">
              <div class="detail-field-label">Tags</div>
              <div class="tags-row" id="dTags"></div>
            </div>
            <div class="detail-field">
              <div class="detail-field-label">Checksum</div>
              <div class="checksum" id="dChecksum">—</div>
            </div>
            <div class="detail-field">
              <div class="detail-field-label">Interpretation (Reasoning Engine)</div>
              <button class="secondary small" onclick="interpretCurrent()" style="width:fit-content;">Run Interpretation</button>
              <div id="dInterpret"></div>
            </div>
          </div>
        </div>
      </div>

      <!-- Graph detail pane -->
      <div class="content-pane" id="pane-graph-detail">
        <div class="detail-header">
          <div class="detail-header-info">
            <h2>Knowledge Graph</h2>
            <div class="detail-id" id="graphDetailSubtitle">Browse nodes and relationships</div>
          </div>
          <button class="secondary small" onclick="loadGraphNodes()">Load Nodes</button>
        </div>
        <div class="detail-body" style="padding:0;">
          <div id="graphDetailMsg" style="padding:12px 16px;"></div>
          <div class="result-list" id="graphNodeList">
            <div class="empty-state"><div class="empty-icon">🕸️</div>Click "Load Nodes" to browse the graph</div>
          </div>
        </div>
      </div>

      <!-- Agent output pane -->
      <div class="content-pane" id="pane-agent-output">
        <div class="detail-header">
          <div class="detail-header-info">
            <h2 id="agentOutputTitle">Agent Output</h2>
            <div class="detail-id" id="agentOutputSubtitle">—</div>
          </div>
        </div>
        <div class="detail-body">
          <div id="agentOutputBody"></div>
        </div>
      </div>

      <!-- Search results pane -->
      <div class="content-pane" id="pane-search-results">
        <div class="detail-header">
          <div class="detail-header-info">
            <h2>Search Results</h2>
            <div class="detail-id" id="searchResultsSubtitle">—</div>
          </div>
          <div class="search-bar" style="width:280px;">
            <input type="text" id="searchMain" placeholder="Search…" onkeydown="if(event.key==='Enter')doSearch()">
            <button onclick="doSearch()">Go</button>
          </div>
        </div>
        <div class="result-list" id="searchResultList">
          <div class="empty-state"><div class="empty-icon">🔍</div>Enter a query above</div>
        </div>
      </div>

    </section>
  </div>
</div>

<script>
'use strict';

const API = 'http://127.0.0.1:8080';
let currentMemoryId = null;
let searchDebounceTimer = null;
const selectedMemoryIds = new Set();

// ── API ──────────────────────────────────────────────────────────────────────

async function apiFetch(endpoint, options = {}) {
  const res = await fetch(`${API}${endpoint}`, {
    ...options,
    headers: { 'Content-Type': 'application/json', ...(options.headers || {}) },
  });
  if (!res.ok) {
    const text = await res.text().catch(() => `HTTP ${res.status}`);
    throw new Error(text || `HTTP ${res.status}`);
  }
  return res.json();
}

// ── Health ───────────────────────────────────────────────────────────────────

async function checkHealth() {
  try {
    await apiFetch('/health');
    document.getElementById('statusDot').className = 'dot ok';
    document.getElementById('statusText').textContent = 'Connected';
  } catch {
    document.getElementById('statusDot').className = 'dot';
    document.getElementById('statusText').textContent = 'Disconnected';
  }
}

// ── Stats ────────────────────────────────────────────────────────────────────

async function refreshStats() {
  try {
    const [listRes, metaRes] = await Promise.all([
      apiFetch('/api/v1/memory/list?limit=1'),
      apiFetch('/api/v1/graph/metadata'),
    ]);
    document.getElementById('s-memories').textContent = listRes.total ?? 0;
    document.getElementById('s-nodes').textContent = metaRes.node_count ?? 0;
    document.getElementById('s-edges').textContent = metaRes.edge_count ?? 0;
    document.getElementById('s-indexed').textContent = listRes.total ?? 0;
  } catch { /* stats are non-critical */ }
}

async function refreshAll() {
  await checkHealth();
  await refreshStats();
  await loadMemoryList();
}

// ── Navigation ───────────────────────────────────────────────────────────────

function switchSidebar(name, btn) {
  document.querySelectorAll('.nav-btn').forEach(b => b.classList.remove('active'));
  btn.classList.add('active');
  document.querySelectorAll('.panel').forEach(p => p.classList.remove('active'));
  document.getElementById(`panel-${name}`).classList.add('active');

  // Auto-switch content pane
  if (name === 'graph') showContentPane('pane-graph-detail');
  else if (name === 'agents') showContentPane('pane-agent-output');
  else showContentPane('pane-memory-detail');
}

function showContentPane(id) {
  document.querySelectorAll('.content-pane').forEach(p => p.classList.remove('active'));
  const pane = document.getElementById(id);
  if (pane) pane.classList.add('active');
}

// ── Memory List ──────────────────────────────────────────────────────────────

async function loadMemoryList(query) {
  const listEl = document.getElementById('memList');
  const msgEl = document.getElementById('memListMsg');
  listEl.innerHTML = '<div class="loading"><span class="spinner"></span>Loading…</div>';
  msgEl.innerHTML = '';

  try {
    let data;
    if (query && query.trim() && query.trim() !== '*') {
      data = await apiFetch('/api/v1/retrieve', {
        method: 'POST',
        body: JSON.stringify({ query: query.trim(), limit: 50 }),
      });
      // data.results[] with .memory inside
      const items = (data.results || []).filter(r => r.memory);
      renderMemoryList(listEl, items.map(r => ({ ...r.memory, _score: r.score })));
      msgEl.innerHTML = items.length
        ? `<div style="font-size:0.75rem;color:var(--muted);padding:4px 0;">${items.length} result(s) for "${query}"</div>`
        : `<div class="error-banner">No results for "${query}"</div>`;
    } else {
      data = await apiFetch('/api/v1/memory/list?limit=100');
      renderMemoryList(listEl, data.memories || []);
    }
  } catch (e) {
    listEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

function renderMemoryList(container, memories) {
  if (!memories.length) {
    container.innerHTML = '<div class="empty-state"><div class="empty-icon">🧠</div>No memories found</div>';
    return;
  }
  container.innerHTML = memories.map(m => {
    const preview = (m.content || '').slice(0, 80).replace(/</g, '&lt;').replace(/>/g, '&gt;');
    const date = m.created_at ? new Date(m.created_at).toLocaleDateString() : '';
    const score = m._score !== undefined ? `<span class="score-badge">${m._score.toFixed(2)}</span> ` : '';
    return `<div class="result-item" onclick="openMemory('${m.memory_id}')" data-id="${m.memory_id}">
      <div class="result-title">${score}${esc(m.source || m.memory_id)}</div>
      <div class="result-meta">${esc(preview)}${preview.length < (m.content || '').length ? '…' : ''}</div>
      <div class="result-meta">${date}${m.tags && m.tags.length ? ' · ' + m.tags.slice(0,3).join(', ') : ''}</div>
    </div>`;
  }).join('');
}

function debounceSearch(value) {
  clearTimeout(searchDebounceTimer);
  searchDebounceTimer = setTimeout(() => loadMemoryList(value), 350);
}

// ── Memory Detail ────────────────────────────────────────────────────────────

async function openMemory(id) {
  currentMemoryId = id;
  showContentPane('pane-memory-detail');

  // Highlight in list
  document.querySelectorAll('.result-item').forEach(el => {
    el.classList.toggle('selected', el.dataset.id === id);
  });

  document.getElementById('detailEmpty').style.display = 'none';
  const dv = document.getElementById('detailView');
  dv.style.display = 'flex';

  // Show loading state
  document.getElementById('dSource').textContent = 'Loading…';
  document.getElementById('dId').textContent = id;
  document.getElementById('dContent').textContent = '';
  document.getElementById('dTags').innerHTML = '';
  document.getElementById('dInterpret').innerHTML = '';

  try {
    const mem = await apiFetch(`/api/v1/memory/${id}`);
    document.getElementById('dSource').textContent = mem.source || '—';
    document.getElementById('dId').textContent = mem.memory_id;
    document.getElementById('dContent').textContent = mem.content || '';
    document.getElementById('dFormat').textContent = mem.format || '—';
    document.getElementById('dCreated').textContent = mem.created_at
      ? new Date(mem.created_at).toLocaleString() : '—';
    document.getElementById('dChecksum').textContent = mem.checksum || '—';
    const tagsEl = document.getElementById('dTags');
    tagsEl.innerHTML = (mem.tags || []).length
      ? mem.tags.map(t => `<span class="tag">${esc(t)}</span>`).join('')
      : '<span style="color:var(--muted);font-size:0.75rem;">No tags</span>';
  } catch (e) {
    document.getElementById('dContent').textContent = `Error: ${e.message}`;
  }
}

function copyId() {
  if (currentMemoryId) {
    navigator.clipboard.writeText(currentMemoryId).catch(() => {});
    document.getElementById('dId').textContent = `${currentMemoryId} (copied!)`;
    setTimeout(() => document.getElementById('dId').textContent = currentMemoryId, 1500);
  }
}

function useInAgents() {
  if (currentMemoryId) {
    // Switch to agents panel and pre-fill the memory ID field
    const agMemIds = document.getElementById('agMemIds');
    const existing = agMemIds.value.trim();
    agMemIds.value = existing ? `${existing}, ${currentMemoryId}` : currentMemoryId;
    // Switch sidebar
    document.querySelectorAll('.nav-btn').forEach((b, i) => { if (i === 3) b.click(); });
  }
}

async function deleteCurrentMemory() {
  if (!currentMemoryId) return;
  if (!confirm(`Delete memory ${currentMemoryId}? This cannot be undone.`)) return;
  try {
    await apiFetch(`/api/v1/memory/${currentMemoryId}`, { method: 'DELETE' });
    currentMemoryId = null;
    document.getElementById('detailEmpty').style.display = '';
    document.getElementById('detailView').style.display = 'none';
    await refreshAll();
  } catch (e) {
    alert(`Delete failed: ${e.message}`);
  }
}

async function interpretCurrent() {
  if (!currentMemoryId) return;
  const el = document.getElementById('dInterpret');
  el.innerHTML = '<div class="loading"><span class="spinner"></span>Running interpretation…</div>';
  try {
    const mem = await apiFetch(`/api/v1/memory/${currentMemoryId}`);
    const res = await apiFetch('/api/v1/reason/interpret', {
      method: 'POST',
      body: JSON.stringify({ query: mem.content.slice(0, 200) }),
    });
    const interps = res.interpretations || [];
    if (!interps.length) {
      el.innerHTML = '<div style="color:var(--muted);font-size:0.8rem;">No interpretations generated.</div>';
      return;
    }
    el.innerHTML = interps.map(i => `
      <div style="background:var(--border2);border-radius:var(--radius);padding:10px;margin-top:8px;">
        <div style="font-size:0.75rem;color:var(--accent);margin-bottom:4px;">Confidence: ${(i.confidence * 100).toFixed(0)}%</div>
        <div style="font-size:13px;">${esc(i.summary)}</div>
        ${i.contradictions && i.contradictions.length ? `<div style="color:var(--red);font-size:0.75rem;margin-top:4px;">⚠ ${i.contradictions.length} contradiction(s)</div>` : ''}
      </div>
    `).join('');
  } catch (e) {
    el.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

// ── Ingest ───────────────────────────────────────────────────────────────────

async function ingest() {
  const content = document.getElementById('inContent').value.trim();
  const source = document.getElementById('inSource').value.trim() || 'user';
  const format = document.getElementById('inFormat').value;
  const tagsRaw = document.getElementById('inTags').value;
  const tags = tagsRaw.split(',').map(t => t.trim()).filter(Boolean);
  const msgEl = document.getElementById('ingestMsg');

  if (!content) { msgEl.innerHTML = '<div class="error-banner">Content is required.</div>'; return; }

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span>Ingesting…</div>';

  try {
    const res = await apiFetch('/api/v1/memory/ingest', {
      method: 'POST',
      body: JSON.stringify({ source, format, content, tags }),
    });
    msgEl.innerHTML = `<div class="success-banner">✓ Created: <code>${res.memory_id}</code></div>`;
    document.getElementById('inContent').value = '';
    await refreshAll();
    // Auto-open the new memory
    setTimeout(() => openMemory(res.memory_id), 300);
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

// ── Graph ────────────────────────────────────────────────────────────────────

async function createGraphNode() {
  const node_type = document.getElementById('gNodeType').value.trim();
  const label = document.getElementById('gNodeLabel').value.trim();
  const msgEl = document.getElementById('graphMsg');
  if (!node_type || !label) { msgEl.innerHTML = '<div class="error-banner">Type and label required.</div>'; return; }

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span></div>';
  try {
    const res = await apiFetch('/api/v1/graph/nodes', {
      method: 'POST',
      body: JSON.stringify({ node_type, label, properties: {}, provenance: [] }),
    });
    msgEl.innerHTML = `<div class="success-banner">✓ Node created: <code>${res.id}</code></div>`;
    await refreshStats();
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

async function createGraphEdge() {
  const source = document.getElementById('gEdgeSrc').value.trim();
  const target = document.getElementById('gEdgeTgt').value.trim();
  const relation = document.getElementById('gEdgeRel').value.trim();
  const confidence = parseFloat(document.getElementById('gEdgeConf').value) || 0.9;
  const msgEl = document.getElementById('graphMsg');
  if (!source || !target || !relation) { msgEl.innerHTML = '<div class="error-banner">Source, target, and relation required.</div>'; return; }

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span></div>';
  try {
    const res = await apiFetch('/api/v1/graph/edges', {
      method: 'POST',
      body: JSON.stringify({ source, target, relation, properties: {}, provenance: [], confidence }),
    });
    msgEl.innerHTML = `<div class="success-banner">✓ Edge created: <code>${res.id}</code></div>`;
    await refreshStats();
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

async function findGraphPath() {
  const start_id = document.getElementById('gPathSrc').value.trim();
  const end_id = document.getElementById('gPathTgt').value.trim();
  const max_depth = parseInt(document.getElementById('gPathDepth').value) || 5;
  const msgEl = document.getElementById('graphMsg');
  if (!start_id || !end_id) { msgEl.innerHTML = '<div class="error-banner">Start and end node IDs required.</div>'; return; }

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span>Searching…</div>';
  try {
    const res = await apiFetch('/api/v1/graph/path', {
      method: 'POST',
      body: JSON.stringify({ start_id, end_id, max_depth }),
    });
    if (res.path) {
      msgEl.innerHTML = `<div class="success-banner">✓ Path (${res.path.length} hops):<br><code style="word-break:break-all;font-size:11px;">${res.path.join(' → ')}</code></div>`;
    } else {
      msgEl.innerHTML = `<div class="error-banner">No path found between these nodes within depth ${max_depth}.</div>`;
    }
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

async function traverseGraph() {
  const start_node_id = document.getElementById('gTraverseNode').value.trim() || null;
  const depth = parseInt(document.getElementById('gTraverseDepth').value) || 3;
  const msgEl = document.getElementById('graphMsg');
  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span>Traversing…</div>';
  try {
    const res = await apiFetch('/api/v1/graph/traverse', {
      method: 'POST',
      body: JSON.stringify({ start_node_id, depth }),
    });
    msgEl.innerHTML = `<div class="success-banner">✓ ${res.nodes.length} node(s), ${res.edges.length} edge(s)</div>`;
    renderGraphNodes(res.nodes, `Traversal result (depth ${depth})`);
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

async function loadGraphNodes() {
  document.getElementById('graphDetailMsg').innerHTML = '<div class="loading"><span class="spinner"></span>Loading nodes…</div>';
  try {
    const res = await apiFetch('/api/v1/graph/nodes/list?limit=200');
    renderGraphNodes(res.nodes, `${res.total} total nodes`);
    document.getElementById('graphDetailMsg').innerHTML = '';
    document.getElementById('graphDetailSubtitle').textContent = `${res.total} node(s) in graph`;
  } catch (e) {
    document.getElementById('graphDetailMsg').innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

function renderGraphNodes(nodes, subtitle) {
  showContentPane('pane-graph-detail');
  document.getElementById('graphDetailSubtitle').textContent = subtitle;
  const listEl = document.getElementById('graphNodeList');
  if (!nodes.length) {
    listEl.innerHTML = '<div class="empty-state"><div class="empty-icon">🕸️</div>No nodes found</div>';
    return;
  }
  listEl.innerHTML = nodes.map(n => {
    const badge = n.node_type === 'concept'
      ? `<span class="concept-badge">${esc(n.node_type)}</span>`
      : `<span class="node-type-badge">${esc(n.node_type)}</span>`;
    return `<div class="graph-node-row">
      ${badge}
      <span style="flex:1;font-size:13px;">${esc(n.label)}</span>
      <span style="font-family:monospace;font-size:0.68rem;color:var(--muted);">${n.id.slice(0,16)}…</span>
      <button class="secondary small" onclick="copyToClipboard('${n.id}', this)">Copy ID</button>
    </div>`;
  }).join('');
}

// ── Search ───────────────────────────────────────────────────────────────────

async function doSearch() {
  const query = document.getElementById('searchMain').value.trim();
  showContentPane('pane-search-results');
  const listEl = document.getElementById('searchResultList');
  const subtitle = document.getElementById('searchResultsSubtitle');
  listEl.innerHTML = '<div class="loading"><span class="spinner"></span>Searching…</div>';
  subtitle.textContent = `Searching for "${query}"…`;

  try {
    const res = await apiFetch('/api/v1/retrieve', {
      method: 'POST',
      body: JSON.stringify({ query: query || '*', limit: 20 }),
    });
    const results = res.results || [];
    subtitle.textContent = `${results.length} result(s) for "${query}"`;
    if (!results.length) {
      listEl.innerHTML = '<div class="empty-state"><div class="empty-icon">🔍</div>No results found</div>';
      return;
    }
    listEl.innerHTML = results.map(r => {
      const m = r.memory;
      if (!m) return '';
      const preview = (m.content || '').slice(0, 100);
      return `<div class="result-item" onclick="openMemory('${m.memory_id}')">
        <div class="result-title"><span class="score-badge">${r.score.toFixed(2)}</span> ${esc(m.source || m.memory_id)}</div>
        <div class="result-meta">${esc(preview)}${preview.length < m.content.length ? '…' : ''}</div>
      </div>`;
    }).join('');
  } catch (e) {
    listEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
    subtitle.textContent = 'Search failed';
  }
}

// ── Agents ───────────────────────────────────────────────────────────────────

async function runAgent(type) {
  const query = document.getElementById('agQuery').value.trim() || '*';
  const memIdsRaw = document.getElementById('agMemIds').value;
  const memIds = memIdsRaw.split(',').map(s => s.trim()).filter(Boolean);
  const msgEl = document.getElementById('agentsMsg');

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span>Running agent…</div>';

  showContentPane('pane-agent-output');
  document.getElementById('agentOutputTitle').textContent = `Agent: ${type}`;
  document.getElementById('agentOutputSubtitle').textContent = 'Running…';
  document.getElementById('agentOutputBody').innerHTML = '<div class="loading"><span class="spinner"></span></div>';

  try {
    let body = {};
    if (type === 'retriever' || type === 'synthesizer') {
      body = { query, limit: 10 };
    } else {
      if (!memIds.length) {
        // Auto-fetch some memory IDs
        const list = await apiFetch('/api/v1/memory/list?limit=10');
        body = { memory_ids: (list.memories || []).map(m => m.memory_id) };
      } else {
        body = { memory_ids: memIds };
      }
    }

    const res = await apiFetch(`/api/v1/agents/${type}`, { method: 'POST', body: JSON.stringify(body) });
    msgEl.innerHTML = `<div class="success-banner">✓ Agent completed: ${res.outputs ? res.outputs.length : 0} output(s)</div>`;
    renderAgentOutput(res, type);
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
    document.getElementById('agentOutputBody').innerHTML = `<div class="error-banner">${e.message}</div>`;
    document.getElementById('agentOutputSubtitle').textContent = 'Error';
  }
}

function renderAgentOutput(res, type) {
  document.getElementById('agentOutputSubtitle').textContent = `${res.agent_id} · Task: ${res.task_id}`;
  const outputs = res.outputs || [];
  if (!outputs.length) {
    document.getElementById('agentOutputBody').innerHTML = '<div class="empty-state"><div class="empty-icon">🤖</div>No outputs returned</div>';
    return;
  }
  document.getElementById('agentOutputBody').innerHTML = outputs.map((o, i) => `
    <div style="background:var(--border2);border-radius:var(--radius);padding:12px;margin-bottom:8px;">
      <div style="display:flex;align-items:center;gap:8px;margin-bottom:6px;">
        <span class="score-badge">Score: ${o.score.toFixed(3)}</span>
        ${o.memory_id ? `<span style="font-family:monospace;font-size:0.7rem;color:var(--muted);">${o.memory_id}</span>` : ''}
      </div>
      <div style="font-size:13px;color:var(--text);margin-bottom:4px;">${esc(o.metadata.explanation)}</div>
      <div style="font-size:0.72rem;color:var(--muted);">Source: ${o.metadata.source} · ${o.metadata.time_taken_ms}ms</div>
      ${o.graph_subgraph ? `<div style="font-size:0.72rem;color:var(--muted);margin-top:4px;">Graph: ${o.graph_subgraph.nodes.length} node(s), ${o.graph_subgraph.edges.length} edge(s)</div>` : ''}
    </div>
  `).join('');
}

// ── Backup ───────────────────────────────────────────────────────────────────

async function doBackup(action) {
  const pathInput = action === 'export' || action === 'import'
    ? document.getElementById('bkExportPath')
    : document.getElementById('bkPath');
  const path = pathInput.value.trim();
  const msgEl = document.getElementById('backupMsg');
  if (!path) { msgEl.innerHTML = '<div class="error-banner">Path is required.</div>'; return; }

  msgEl.innerHTML = '<div class="loading"><span class="spinner"></span>Processing…</div>';
  try {
    const res = await apiFetch(`/api/v1/backup/${action}`, {
      method: 'POST',
      body: JSON.stringify({ path }),
    });
    msgEl.innerHTML = `<div class="success-banner">✓ ${res.status}${res.manifest ? ` · Memories: ${res.manifest.memory_count}` : ''}</div>`;
    if (action === 'restore' || action === 'import') await refreshAll();
  } catch (e) {
    msgEl.innerHTML = `<div class="error-banner">${e.message}</div>`;
  }
}

// ── Utilities ────────────────────────────────────────────────────────────────

function esc(str) {
  return String(str || '')
    .replace(/&/g, '&amp;')
    .replace(/</g, '&lt;')
    .replace(/>/g, '&gt;')
    .replace(/"/g, '&quot;');
}

function copyToClipboard(text, btn) {
  navigator.clipboard.writeText(text).then(() => {
    const orig = btn.textContent;
    btn.textContent = 'Copied!';
    setTimeout(() => { btn.textContent = orig; }, 1200);
  }).catch(() => {});
}

// ── Init ─────────────────────────────────────────────────────────────────────

(async function init() {
  await checkHealth();
  await refreshStats();
  await loadMemoryList();
  setInterval(checkHealth, 8000);
})();
</script>
</body>
</html>
```

---

## PART 4 — ADDITIONAL RUST FIXES

---

### FIX 8 — Routing Order: Static Routes Must Precede Dynamic Routes

**File: `src/api/mod.rs`**

In Axum 0.7, static segments take priority over dynamic segments only when registered correctly. Ensure `list` and `search` routes come before `/:memory_id` in `create_router`. Verify the route order is:

```rust
.route("/api/v1/memory/list", get(list_memories))
.route("/api/v1/memory/search", get(search_memories_get))
.route("/api/v1/graph/nodes/list", get(list_nodes))
.route("/api/v1/memory/:memory_id", get(get_memory).delete(delete_memory))
.route("/api/v1/graph/nodes/:node_id", get(get_node).delete(delete_node_handler))
```

Combine the GET and DELETE methods on the same route using the method chaining syntax shown above.

---

### FIX 9 — Remove `use axum::extract::Json` Ambiguity

**File: `src/api/mod.rs`**

The file imports both `Json` (for extraction) and `Json as AxumJson` (for response). Consolidate to avoid confusion:

```rust
use axum::{
    Router,
    routing::{get, post},
    extract::{Path, State, Json as ExtractJson},
    response::Json as AxumJson,
    http::Method,
    http::StatusCode,
    response::{IntoResponse, Response},
};
```

Update all handler function signatures to use `ExtractJson` for request body extraction and `AxumJson` for response. For example:

```rust
async fn ingest_memory(
    State(state): State<Arc<AppState>>,
    ExtractJson(req): ExtractJson<IngestRequest>,
) -> Result<AxumJson<IngestResponse>, AppError> {
```

Apply this rename consistently to all POST handlers with request bodies.

---

### FIX 10 — `AppError` Should Implement `std::error::Error`

**File: `src/api/mod.rs`**

The `From<E: Into<anyhow::Error>>` impl creates a conflict with the blanket `From<anyhow::Error>` when used with `?` in certain contexts. Tighten the impl:

```rust
impl From<anyhow::Error> for AppError {
    fn from(err: anyhow::Error) -> Self {
        Self(err)
    }
}
```

Remove the generic `impl<E: Into<anyhow::Error>> From<E> for AppError`. Instead, use `.map_err(anyhow::Error::from)?` in any handler where the underlying error is not already `anyhow::Error`. This avoids the orphan rule conflict and gives cleaner error chains.

---

## PART 5 — TESTS

---

### IMPROVEMENT 7 — Add Integration Tests for New Endpoints

**File: `src/tests.rs`**

Add these test blocks inside the existing `mod tests { ... }` block:

```rust
#[test]
fn test_graph_store_get_or_create_node_dedup() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();

    let n1 = store.get_or_create_node(
        "concept".to_string(), "rust".to_string(), serde_json::json!({}), vec![]
    ).unwrap();
    let n2 = store.get_or_create_node(
        "concept".to_string(), "rust".to_string(), serde_json::json!({}), vec![]
    ).unwrap();

    // Must return the same ID — no duplicate created
    assert_eq!(n1.id, n2.id);

    let meta = store.get_metadata().unwrap();
    assert_eq!(meta.node_count, 1);
}

#[test]
fn test_graph_store_create_edge_if_absent_dedup() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();

    let a = store.create_node("T".to_string(), "A".to_string(), serde_json::json!({}), vec![]).unwrap();
    let b = store.create_node("T".to_string(), "B".to_string(), serde_json::json!({}), vec![]).unwrap();

    let e1 = store.create_edge_if_absent(a.id.clone(), b.id.clone(), "r".to_string(), serde_json::json!({}), vec![], 1.0).unwrap();
    let e2 = store.create_edge_if_absent(a.id.clone(), b.id.clone(), "r".to_string(), serde_json::json!({}), vec![], 1.0).unwrap();

    assert_eq!(e1.id, e2.id);

    let meta = store.get_metadata().unwrap();
    assert_eq!(meta.edge_count, 1);
}

#[test]
fn test_memory_store_delete() {
    let dir = TempDir::new().unwrap();
    let mut store = crate::storage::MemoryStore::new(dir.path().to_path_buf()).unwrap();

    let m = store.ingest("s".to_string(), "p".to_string(), "hello".to_string(), vec![]).unwrap();
    assert_eq!(store.count().unwrap(), 1);
    store.delete(&m.memory_id).unwrap();
    assert_eq!(store.count().unwrap(), 0);
    assert!(store.get(&m.memory_id).is_err());
}

#[test]
fn test_retrieval_delete_from_index() {
    let dir = TempDir::new().unwrap();
    let engine = crate::retrieval::RetrievalEngine::new(dir.path().to_path_buf()).unwrap();

    let m = crate::models::MemoryObject::new("s".to_string(), "p".to_string(), "content".to_string(), vec![]);
    engine.index_memory(&m).unwrap();

    std::fs::create_dir_all(dir.path().join("memories")).unwrap();
    let path = dir.path().join("memories").join(format!("{}.json", m.memory_id));
    std::fs::write(&path, serde_json::to_string(&m).unwrap()).unwrap();

    let before = engine.search("content", 10).unwrap();
    assert!(!before.is_empty());

    engine.delete_from_index(&m.memory_id).unwrap();
    std::fs::remove_file(&path).unwrap();

    let after = engine.search("content", 10).unwrap();
    assert!(after.is_empty());
}

#[test]
fn test_query_nodes_without_node_type_no_panic() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();
    store.create_node("X".to_string(), "foo".to_string(), serde_json::json!({}), vec![]).unwrap();
    // This was broken before Fix 1 — must not panic or return an error
    let nodes = store.query_nodes(None, None, 10).unwrap();
    assert_eq!(nodes.len(), 1);
}

#[test]
fn test_graph_metadata_live_count() {
    let dir = TempDir::new().unwrap();
    let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();
    store.create_node("T".to_string(), "A".to_string(), serde_json::json!({}), vec![]).unwrap();
    store.create_node("T".to_string(), "B".to_string(), serde_json::json!({}), vec![]).unwrap();
    let meta = store.get_metadata().unwrap();
    assert_eq!(meta.node_count, 2);
    assert_eq!(meta.edge_count, 0);
}
```

---

## PART 6 — VERIFICATION

After all changes are applied, run:

```bash
cargo build 2>&1
```

Fix any compilation errors. Then:

```bash
cargo test 2>&1
```

All tests must pass (minimum 22 tests). If a test fails, fix the root cause — never delete a test.

```bash
cargo clippy -- -D warnings 2>&1
```

Fix all warnings. Common expected issues after these changes:
- `VecDeque` needs `use std::collections::VecDeque` in `graph.rs`
- `rusqlite::params!` needs explicit import in `backup/mod.rs` — add `use rusqlite::params;`
- `delete` routing requires `use axum::routing::delete` — add the import

Final smoke test:

```bash
cargo run &
sleep 2
curl -s http://127.0.0.1:8080/health | python3 -m json.tool
curl -s -X POST http://127.0.0.1:8080/api/v1/memory/ingest \
  -H "Content-Type: application/json" \
  -d '{"source":"smoke-test","format":"plaintext","content":"The quick brown fox jumps over the lazy dog","tags":["test"]}' \
  | python3 -m json.tool
curl -s 'http://127.0.0.1:8080/api/v1/memory/list?limit=5' | python3 -m json.tool
curl -s 'http://127.0.0.1:8080/api/v1/graph/nodes/list' | python3 -m json.tool
```

The implementation is complete when:
1. `cargo build` is error-free
2. `cargo test` shows ≥ 22 tests passing
3. `cargo clippy -- -D warnings` is warning-free
4. `/health` returns `{"status":"ok","version":"0.1.0"}`
5. Ingesting a memory creates graph nodes visible in `/api/v1/graph/nodes/list` without duplicates
6. The frontend opens, shows the memory list, and supports selecting a memory to view full detail, running interpretation inline, and deleting it
7. `query_nodes(None, None, 10)` does not panic or error

---

## Summary of All Changes

| # | File | Change |
|---|------|--------|
| Fix 1 | `src/storage/graph.rs` | `query_nodes` SQL parameter binding bug |
| Fix 2 | `src/storage/graph.rs` | `traverse_bfs` and `find_path` use VecDeque (real BFS) |
| Fix 3 | `src/storage/graph.rs` + `src/api/mod.rs` | Node dedup via UNIQUE index + `get_or_create_node` + `create_edge_if_absent` |
| Fix 4 | `src/api/mod.rs` | `traverse_graph` deduplicates returned edges |
| Fix 5 | `src/backup/mod.rs` | `import_data` re-indexes into retrieval.db |
| Fix 6 | `src/backup/mod.rs` | `count_lines` uses `&Path` not `&PathBuf` |
| Fix 7 | `src/api/mod.rs` | `ingest_memory` drops locks sequentially to prevent deadlock |
| Fix 8 | `src/api/mod.rs` | Route order: static before dynamic |
| Fix 9 | `src/api/mod.rs` | `Json` import ambiguity resolved |
| Fix 10 | `src/api/mod.rs` | `AppError` From impl tightened |
| Imp 1 | `src/retrieval/mod.rs` | `INSERT OR IGNORE` in `index_memory` |
| Imp 2 | `src/storage/graph.rs` | Live-count `count_nodes`/`count_edges`; `get_metadata` returns live values |
| Imp 3 | `src/api/mod.rs` | `GET /api/v1/graph/nodes/list` endpoint |
| Imp 4 | `src/api/mod.rs` | `GET /api/v1/memory/search` endpoint |
| Imp 5 | `src/storage/memory.rs` + `src/storage/graph.rs` + `src/retrieval/mod.rs` + `src/api/mod.rs` | Delete endpoints for memories and nodes |
| Imp 6 | `src/api/mod.rs` | Content size guard on ingest |
| Imp 7 | `src/tests.rs` | 6 new targeted regression tests |
| UX | `frontend/index.html` | Complete rewrite: sidebar navigation, memory detail view, inline interpretation, delete button, agent ID pre-fill, per-panel error display, loading spinners, graph node browser, search results pane |