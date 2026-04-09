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
