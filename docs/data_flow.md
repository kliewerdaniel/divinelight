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
