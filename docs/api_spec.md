API Specification (High Level)

- Versioning: v1, with future v2+ branches.
- Endpoints (examples):
  - POST /ingest: ingest raw memory input; returns memory_id.
  - GET /memory/{memory_id}: fetch verbatim memory and metadata.
  - POST /retrieve: submit query; returns candidate memories with provenance and scores.
  - POST /reason/interpret: run reasoning pipeline; returns belief-state and interpretations.
  - GET /beliefs/{belief_id}: fetch current belief-state entry.
- Data formats: JSON payloads with defined schemas; include provenance metadata.
- Security: local-only, optional mutual TLS for multi-process setups; no external network access by default.
- Validation: server should validate payloads against JSON schemas; versioned responses included in headers.
