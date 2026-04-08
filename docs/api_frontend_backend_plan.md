API & Backend Plan

- Overall architecture
  - Local-only backend services exposing REST/GraphQL endpoints; a frontend SPA or CLI can interact with them locally.
- Core services
  - ingestion-service: capture and normalize input; writes to MemPalace.
  - memory-store-service: MemPalace-based verbatim storage with indices.
  - retrieval-service: hybrid retrieval (vector + symbolic + temporal) on top of the memory store.
  - reasoning-service: coordinates Agent workflow, belief-state, arbitration.
  - agent-registry-service: registers available agents, their capabilities, and lifecycle.
  - interface-service: API gateway, authentication (local), and UI endpoints.
- API design goals
  - Clear versioning, backwards-compatible schema evolution, and explicit provenance in responses.
  - Stateless external API surfaces; state is maintained in memory-fast stores or memory backend.
- Frontend considerations
  - Local-first UI/CLI for memory ingestion, recall sessions, belief-state exploration, and audit trails.
  - Visualizations: memory graph, recall pathways, agent outputs, confidence heatmaps.
