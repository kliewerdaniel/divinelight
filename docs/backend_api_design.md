Backend API Design (Local-first)

- High-level principles
  - RESTful endpoints with clear versioning.
  - All operations are stateful on the local machine; backend stores state in MemPalace (raw memory) and Graphify (knowledge graph) with indices.
  - Responses carry explicit provenance and confidence metadata.
  - Endpoints are organized by layer: Memory API, Graph API, Reasoning API.

- Endpoints (illustrative)

  Memory Layer (MemPalace):
  - POST /api/v1/memory/ingest
    - Body: MemoryObject schema
    - Response: { memory_id: string, status: string }
  - GET /api/v1/memory/{memory_id}
    - Response: MemoryObject + EmbeddingIndex + Memory-Graph Linkage references
  - POST /api/v1/memory/query
    - Body: { query: string, limit?: int, filters?: object }
    - Response: { candidates: [ { memory_id, score, provenance } ] }

  Graph Layer (Graphify):
  - POST /api/v1/graph/nodes
    - Body: GraphNode schema
    - Response: { node_id: string, status: string }
  - GET /api/v1/graph/nodes/{node_id}
    - Response: GraphNode + linked memory IDs
  - POST /api/v1/graph/edges
    - Body: GraphEdge schema
    - Response: { edge_id: string, status: string }
  - GET /api/v1/graph/edges/{edge_id}
    - Response: GraphEdge + provenance
  - POST /api/v1/graph/traverse
    - Body: { start_node_id?: string, query?: string, depth?: int, types?: string[] }
    - Response: { subgraphs: [ { nodes, edges } ], paths }
  - GET /api/v1/graph/subgraph/{node_id}
    - Response: neighborhood subgraph
  - GET /api/v1/graph/metadata
    - Response: GraphMetadata (node_count, edge_count, schema_version)

  Hybrid Retrieval:
  - POST /api/v1/retrieve
    - Body: { query: string, modes?: string[], limit?: int, filters?: object }
    - Response: { results: [ { memory?, graph?, score, source, provenance, confidence } ] }

  Reasoning Layer:
  - POST /api/v1/reason/interpret
    - Body: { query: string, context_ids?: [string] }
    - Response: { belief_state, interpretations, provenance }
  - GET /api/v1/reason/beliefs/{belief_id}
    - Response: BeliefState with conflict flags
  - GET /api/v1/reason/conflicts
    - Response: active conflicts across memory/graph

  - GET /health

- Data formats and schemas
  - Use the JSON schemas defined in docs/data_models.md as the canonical source.
- Error handling
  - HTTP status codes: 200 OK, 400 Bad Request, 401/403 for auth if enabled, 500 for internal errors.
- Security
  - Local-only; optional API key or token-based access within localhost.