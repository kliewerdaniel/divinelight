Testing Plan

- Unit tests
  - MemPalace storage read/write fidelity.
  - Index build and query correctness for embeddings, keywords, and temporal signals.
- Integration tests
  - Ingestion -> indexing -> retrieval -> reasoning flow with deterministic outputs.
- End-to-end tests
  - Simulated sessions with a predefined memory corpus; verify belief-state evolution across multiple queries.
- Performance tests
  - Latency benchmarks for ingestion and recall; memory footprint under load.
- Regression tests
  - Ensure phase transitions do not regress existing capabilities.
