Performance Considerations

- Local hardware constraints
  - Multi-core CPU, optional GPU for embeddings/graph processing, configurable RAM, and local SSD storage.

- Storage growth projections
  - Verbatim memory grows with capture rate; indices and graph edges scale with relationship complexity.
  - Tiered storage: hot, warm, cold archival to manage cost and latency.

- Retrieval latency challenges
  - Vector search latency depends on embedding size and index size.
  - Graph traversal latency grows with graph diameter; cap depth and cache hot paths.

- Accuracy vs speed tradeoffs
  - Early phases favor fidelity and auditability; later phases incorporate caching and approximate indexing for responsiveness while preserving traces.
