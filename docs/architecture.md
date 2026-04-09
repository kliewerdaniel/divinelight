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
     │ - VerifierAgent     │
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
