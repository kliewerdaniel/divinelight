Backend API Design (Local-first)

- High-level principles
  - RESTful endpoints with clear versioning.
  - All operations are stateful on the local machine; backend stores state in MemPalace with indices.
  - Responses carry explicit provenance and confidence metadata.

- Endpoints (illustrative)
  - POST /api/v1/ingest
    - Body: MemoryObject schema
    - Response: { memory_id: string, status: string }
  - GET /api/v1/memory/{memory_id}
    - Response: MemoryObject + EmbeddingIndex + GraphNode reference if linked
  - POST /api/v1/retrieve
    - Body: { query: string, limit?: int, filters?: object }
    - Response: { candidates: [ { memory_id, score, provenance } ] }
  - POST /api/v1/reason/interpret
    - Body: { query: string, context_ids?: [string] }
    - Response: { belief_state, interpretations, provenance }
  - GET /api/v1/beliefs/{belief_id}
  - POST /api/v1/graph/nodes
    - Body: GraphNode schema
  - GET /health

- Data formats and schemas
  - Use the JSON schemas defined in docs/data_models.md as the canonical source.
- Error handling
  - HTTP status codes: 200 OK, 400 Bad Request, 401/403 for auth if enabled, 500 for internal errors.
- Security
  - Local-only; optional API key or token-based access within localhost.
