# DivineLight — Targeted Bug Fix Agent Prompt

You are an autonomous coding agent working on **DivineLight**. The repository has
already gone through two rounds of improvement (IMPROVE.md, IMPROVE2.md) and
compiles cleanly. The bugs below are **still present** in the current source.
Fix each one in order. Re-read every file immediately before editing it.

---

## FIX 1 — `export_data` Has Wrong Return Type

**File: `src/backup/mod.rs`**

`export_data` currently returns `Result<File>`. The `File` handle is unused by
every caller and the signature is semantically wrong. Change it to `Result<()>`:

```rust
pub fn export_data(&self, export_path: &Path) -> Result<()> {
    let mut file = File::create(export_path)?;

    let memories_dir = self.data_dir.join("memories");
    if memories_dir.exists() {
        for entry in fs::read_dir(&memories_dir)? {
            let entry = entry?;
            if entry.path().extension().is_some_and(|e| e == "json") {
                let content = fs::read_to_string(entry.path())?;
                writeln!(file, "{}", content)?;
            }
        }
    }

    Ok(())
}
```

No caller changes are needed — both call sites already ignore the return value
with `?` or `Ok(_)`.

---

## FIX 2 — Backup Handlers Ignore `DIVINELIGHT_DATA_DIR`

**File: `src/api/mod.rs`**

All four backup handlers (`create_backup`, `restore_backup`, `export_data`,
`import_data`) reconstruct `data_dir` by calling `dirs::data_local_dir()`
directly. This silently ignores the `DIVINELIGHT_DATA_DIR` environment variable
that the `Config` struct already reads correctly.

**Step A** — Add `data_dir` to `AppState`:

```rust
pub struct AppState {
    pub data_dir: std::path::PathBuf,   // ADD THIS FIELD
    pub memory: Mutex<MemoryStore>,
    pub graph: Mutex<GraphStore>,
    pub retrieval: Mutex<RetrievalEngine>,
    pub reasoning: Mutex<ReasoningEngine>,
}
```

**Step B** — In `src/main.rs`, populate the new field when constructing `AppState`:

```rust
let state = Arc::new(AppState {
    data_dir: config.data_dir.clone(),   // ADD THIS LINE
    memory: Mutex::new(memory_store),
    graph: Mutex::new(graph_store),
    retrieval: Mutex::new(retrieval_engine),
    reasoning: Mutex::new(reasoning_engine),
});
```

**Step C** — Replace the hardcoded `data_dir` reconstruction in all four backup
handlers with `state.data_dir.clone()`. For example:

```rust
async fn create_backup(
    State(state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<AxumJson<BackupResponse>, AppError> {
    let backup_path = std::path::PathBuf::from(&req.path);
    let manager = BackupManager::new(state.data_dir.clone());   // ← use state
    let manifest = manager.create_backup(&backup_path)?;
    Ok(AxumJson(BackupResponse {
        status: "success".to_string(),
        manifest: Some(manifest),
    }))
}
```

Apply the same pattern (`BackupManager::new(state.data_dir.clone())`) to
`restore_backup`, `export_data`, and `import_data`. Remove the stale
`dirs::data_local_dir()` calls from these four handlers; the `dirs` import
in `src/api/mod.rs` can be removed entirely if it is no longer used elsewhere
in that file.

---

## FIX 3 — `interpret` Handler Holds Two Mutex Locks Simultaneously

**File: `src/api/mod.rs`**

The `interpret` handler acquires the `retrieval` lock and, while still holding
it, acquires the `reasoning` lock. This violates the sequential lock-drop
pattern already applied to `ingest_memory` and risks lock-ordering issues.

Replace the handler body with:

```rust
async fn interpret(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InterpretRequest>,
) -> Result<AxumJson<InterpretResponse>, AppError> {
    // Step 1: search, then drop the retrieval lock
    let results = {
        let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
        retrieval.search(&req.query, 10)?
    };

    // Step 2: interpret with a separate lock acquisition
    let reasoning = state.reasoning.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let response = reasoning.interpret(&req.query, results)?;

    Ok(AxumJson(response))
}
```

---

## FIX 4 — `POST /api/v1/graph/nodes` Returns 500 on Duplicate `(node_type, label)`

**File: `src/api/mod.rs`**

The `create_node` handler calls `graph.create_node(...)` directly. Because a
`UNIQUE INDEX idx_node_label_type ON nodes(node_type, label)` exists, submitting
a node with an already-used `(node_type, label)` pair returns an opaque
`INTERNAL_SERVER_ERROR`. The correct behaviour is to return the existing node
(idempotent upsert), matching what auto-extraction already does via
`get_or_create_node`.

Replace the handler:

```rust
async fn create_node(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<AxumJson<GraphNode>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    // Use get_or_create_node so duplicate (type, label) pairs are idempotent
    let node = graph.get_or_create_node(
        req.node_type,
        req.label,
        req.properties,
        req.provenance,
    )?;
    Ok(AxumJson(node))
}
```

---

## FIX 5 — Unused Imports in `src/tests.rs` Break `cargo clippy -- -D warnings`

**File: `src/tests.rs`**

The top of the `mod tests` block imports six symbols that are never referenced
directly — the tests use full `crate::` paths instead. Remove the unused imports:

```rust
// DELETE these lines:
use crate::agents::{ContradictionDetectorAgent, RetrieverAgent};
use crate::models::graph::{GraphEdge, GraphNode};
use crate::reasoning::ReasoningEngine;
use crate::retrieval::{RetrievalEngine, RetrievalResult};
use std::path::PathBuf;
```

Keep only what is actually used at the top of the test module:

```rust
use crate::models::memory::MemoryObject;
use crate::models::belief::{BeliefState, ConflictFlag};
use crate::retrieval::RetrievalResult;
use tempfile::TempDir;
```

Check every individual `use` statement against actual usage in the test
functions before removing anything; adjust if a symbol is genuinely referenced.

---

## VERIFICATION

After all five fixes, run:

```bash
cargo build 2>&1
```
Zero errors expected.

```bash
cargo test 2>&1
```
All existing tests (≥ 22) must still pass.

```bash
cargo clippy -- -D warnings 2>&1
```
Zero warnings expected.

```bash
DIVINELIGHT_DATA_DIR=/tmp/dl-test cargo run &
sleep 2
curl -s http://127.0.0.1:8080/health
# Confirm backup uses /tmp/dl-test and not ~/Library/Application Support/divinelight
curl -s -X POST http://127.0.0.1:8080/api/v1/backup/create \
  -H "Content-Type: application/json" \
  -d '{"path":"/tmp/dl-backup-test"}'
ls /tmp/dl-backup-test   # manifest.json must exist here
```

The implementation is complete when:
1. `cargo build` is error-free
2. `cargo test` passes ≥ 22 tests
3. `cargo clippy -- -D warnings` is warning-free
4. A backup created while `DIVINELIGHT_DATA_DIR=/tmp/dl-test` writes to that
   directory, not to the platform default
5. `POST /api/v1/graph/nodes` with the same `node_type`+`label` twice returns
   the same `id` both times (idempotent)
6. `POST /api/v1/reason/interpret` succeeds without deadlock under concurrent load