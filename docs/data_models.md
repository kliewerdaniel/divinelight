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

- Graph Node
  - Purpose: memory unit as graph vertex.
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

- Graph Edge
  - Purpose: relationships between nodes.
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
      "score": 0.87,
      "metadata": {
        "source": "vector_search",
        "explanation": "high cosine similarity to query",
        "time_taken_ms": 12
      }
    }
  ],
  "created_at": "2026-04-01T12:35:00Z"
}
```

- Belief State Representation
  - Purpose: track evolving interpretations and confidence.
  - Example:
```
{
  "belief_id": "belief_001",
  "timestamp": "2026-04-01T13:00:00Z",
  "interpretations": [
    {
      "interpretation_id": "interp_01",
      "summary": "Memory about topic X suggests Y.",
      "confidence": 0.78,
      "supporting_memory_ids": ["mp_...","mp_..."],
      "contradictions": []
    }
  ],
  "state": "open"
}
```

- JSON Schemas (reference)
Provide the schemas from the previous design as formal references for versioning and validation.
