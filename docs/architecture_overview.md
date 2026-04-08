Architecture Overview

ASCII Diagram
```
+------------------+        +-----------------+        +-----------------+
|  Ingestion Layer | ---->  | Cold Storage    | ---->  | Retrieval Layer |
|  (Capture &       |        | (MemPalace base) |        | (Vector + Graph) |
|   Normalization)   |        +-----------------+ |        +-----------------+
+------------------+                          |            |
                                               v            v
                                      +---------------------------+
                                      | Reasoning Layer (Synt)     |
                                      | - Hybrid retrieval           |
                                      | - Agent-based evaluation     |
                                      | - Hallucination mitigation   |
                                      +---------------------------+
                                               |
                                               v
                                   +------------------------+
                                   | Agent Orchestration    |
                                   | Layer (Workflow,       |
                                   | Arbitration, Belief State)|
                                   +------------------------+
                                               |
                                               v
                               +---------------------------------+
                               | Interface Layer (API / CLI / UI)|
                               +---------------------------------+
```

- Major layers and responsibilities
  - Ingestion Layer: capture events, lightweight normalization, tagging for indexing.
  - Cold Storage Layer (MemPalace base): verbatim storage, append-only, minimal processing.
  - Retrieval Layer: hybrid retrieval (vector embeddings, symbolic/keyword indexing, temporal indexing).
  - Reasoning Layer (Synt-inspired): orchestrates retrieval strategies, multi-agent evaluation, cross-checking, and candidate interpretation generation.
  - Agent Orchestration Layer: coordinates Retriever, Verifier, Synthesizer, Contradiction Detector; maintains a belief-state and arbitration rules.
  - Interface Layer: API, CLI, and UI surfaces for external tooling and human-in-the-loop oversight.

- Data flow summary
  - Ingestion writes verbatim memory to MemPalace.
  - Retrieval Layer reads from MemPalace indices to produce candidate slices.
  - Reasoning Layer consumes retrieval outputs, executes agent evaluations, and emits interpretations with confidence signals.
  - Agent Orchestration Layer merges results, maintains belief-state, resolves conflicts.
  - Interface Layer exposes memory and reasoning results to clients and tooling.
