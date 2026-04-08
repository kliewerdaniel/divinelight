Data Models

- Memory Object (MemPalace unit)
  - Purpose: verbatim capture with metadata.
  - JSON schema example:
```
{
  "memory_id": "mp_20260401_123456",
  "created_at": "2026-04-01T12:34:56Z",
  "source": "user_session",
  "format": "plaintext",
  "content": "Full verbatim memory content as captured.",
  "tags": ["session", "discussion", "topic:AI_memory"],
  "checksum": "sha256:abc123...",
  "version": 1,
  "notes": "unmodified verbatim capture; no summarization."
}
```

- Embedding Index
  - Purpose: vector embedding for similarity search.
  - Example:
```
{
  "memory_id": "mp_20260401_123456",
  "embedding": [0.012, -0.003, ..., 0.021],
  "model": "local-embed-v1",
  "timestamp": "2026-04-01T12:34:56Z",
  "norm": 0.97
}
```

- Graph Node (Graphify)
  - Purpose: entity, concept, event, or abstraction extracted from memory.
  - Example:
```
{
  "id": "node-1234",
  "type": "Event",
  "label": "Team Standup",
  "properties": {
    "date": "2026-04-06",
    "duration_min": 15,
    "outcome": "Action items documented"
  },
  "provenance": ["mp_20260401_123456", "graph-extract-202604"],
  "version": "v1.2",
  "created_at": "2026-04-06T09:00:00Z",
  "updated_at": "2026-04-06T09:15:00Z"
}
```

- Graph Edge (Graphify)
  - Purpose: relationships between graph nodes with provenance and confidence.
  - Example:
```
{
  "id": "edge-5678",
  "source": "node-1234",
  "target": "node-9012",
  "relation": "related_to",
  "properties": {
    "context": "discussed in standup",
    "temporal_bounds": ["2026-04-06T09:00:00Z", "2026-04-06T09:15:00Z"]
  },
  "provenance": ["mp_20260401_123456", "mp_20260401_123457"],
  "confidence": 0.82,
  "version": "v1.2",
  "created_at": "2026-04-06T09:15:00Z",
  "updated_at": "2026-04-06T09:15:00Z"
}
```

- Graph Metadata / Versioning
  - Purpose: capture graph schema, provenance, evolution state.
  - Example:
```
{
  "graph_id": "graph-GL-202604",
  "schema_version": "s2.1",
  "created_at": "2026-04-01T00:00:00Z",
  "updated_at": "2026-04-06T10:00:00Z",
  "node_count": 1250,
  "edge_count": 3420,
  "retention_policy": "retain_forever"
}
```

- Memory-Graph Linkage
  - Purpose: establish traceable connections between memory chunks and graph elements.
  - Example:
```
{
  "memory_id": "mp_20260401_123456",
  "linked_nodes": ["node-1234"],
  "linked_edges": ["edge-5678"],
  "link_type": "extracted_as_entity",
  "confidence": 0.88,
  "justification": "Entity extraction with high lexical overlap"
}
```

- Graph Node (Legacy memory-as-node)
  - Purpose: memory unit as graph vertex (pre-Graphify compatibility).
  - Example:
```
{
  "node_id": "node_mp_20260401_123456",
  "memory_id": "mp_20260401_123456",
  "type": "memory_unit",
  "created_at": "2026-04-01T12:34:56Z",
  "attributes": {
    "topic": "AI_memory",
    "source": "user_session",
    "tags": ["session","topic:AI_memory"]
  }
}
```

- Graph Edge (Legacy)
  - Purpose: relationships between nodes (pre-Graphify compatibility).
  - Example:
```
{
  "edge_id": "edge_sem_1",
  "from_node": "node_mp_20260401_123456",
  "to_node": "node_mp_20260402_101112",
  "type": "semantic",
  "weight": 0.85,
  "timestamp": "2026-04-02T10:11:12Z",
  "attributes": {
    "relation": "discussed_in_context_of",
    "confidence": 0.92
  }
}
```

- Agent Output Schema
  - Purpose: standardized outputs from agents including provenance and confidence.
  - Example:
```
{
  "agent_id": "retriever_v1",
  "task_id": "task_01",
  "outputs": [
    {
      "memory_id": "mp_20260401_123456",
      "graph_subgraph": {
        "nodes": ["node-1234"],
        "edges": ["edge-5678"]
      },
      "score": 0.87,
      "metadata": {
        "source": "hybrid_retrieval",
        "explanation": "high cosine similarity to query + graph path corroboration",
        "time_taken_ms": 12
      }
    }
  ],
  "created_at": "2026-04-01T12:35:00Z"
}
```

- Belief State Representation
  - Purpose: track evolving interpretations and confidence across memory and graph sources.
  - Example:
```
{
  "belief_id": "belief_001",
  "timestamp": "2026-04-01T13:00:00Z",
  "interpretations": [
    {
      "interpretation_id": "interp_01",
      "summary": "Memory about topic X suggests Y with graph corroboration.",
      "confidence": 0.78,
      "supporting_memory_ids": ["mp_20260401_123456", "mp_20260401_123457"],
      "supporting_graph_edges": ["edge-5678"],
      "contradictions": []
    }
  ],
  "conflict_flags": [
    {
      "type": "graph_vs_memory",
      "description": "Edge confidence 0.82 vs memory provenance insufficient",
      "affected_edges": ["edge-5678"]
    }
  ],
  "state": "open"
}
```

- JSON Schemas (reference)
Provide the schemas from the previous design as formal references for versioning and validation.