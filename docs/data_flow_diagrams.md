Data Flow Diagrams

- Ingestion to MemPalace
  - User action -> Ingestion Layer -> MemPalace (append-only) -> EmbeddingIndex update.
- Recall flow
  - Player asks a question -> API -> Retrieval Layer (vector + keywords) -> Candidate memories -> Reasoning Layer (Retriever/Verifier/Synthesizer) -> Belief State update -> API results.
- Belief-state evolution
  - Belief-state store updated by arbitration results; agent outputs attached for auditability.
- Graph-based recall
  - From memory nodes to graph traversal to expand contexts; results feed into reasoning.
- Error handling path
  - If any stage fails, error path logs with provenance to support debugging.
