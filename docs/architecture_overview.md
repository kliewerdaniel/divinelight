Architecture Overview

ASCII Diagram
```
+------------------+        +-----------------+        +-----------------+
|  Ingestion Layer | ---->  | Cold Storage    | ---->  | Retrieval Layer |
|  (Capture &       |        | (MemPalace base) |        | (Vector + Graph)|
|   Normalization)   |        +-----------------+ |        +-----------------+
+------------------+                          |            |
                                                v            v
                                       +---------------------------+
                                       | Graph Construction Layer  |
                                       | (Graphify: Entity/Relation |
                                       |  Extraction, Graph Build)|
                                       +---------------------------+
                                                |
                                                v
                                       +---------------------------+
                                       | Reasoning Layer (Synt)     |
                                       | - Hybrid retrieval           |
                                       | - Graph Query Agent        |
                                       | - Agent-based evaluation  |
                                       | - Hallucination mitigation|
                                       +---------------------------+
                                                |
                                                v
                                    +------------------------+
                                    | Agent Orchestration    |
                                    | Layer (Workflow,   |
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
  - Retrieval Layer: hybrid retrieval (vector embeddings, graph traversal, symbolic/keyword indexing, temporal indexing).
  - Graph Construction Layer (Graphify): entity extraction, relationship discovery, event detection, graph building and versioning, memory-graph linkage.
  - Reasoning Layer (Synt-inspired): orchestrates retrieval strategies, multi-agent evaluation, cross-checking, graph-aware reasoning, Graph Query Agent, and candidate interpretation generation.
  - Agent Orchestration Layer: coordinates Retriever, Verifier, Synthesizer, Contradiction Detector, Graph Query Agent; maintains a belief-state and arbitration rules.
  - Interface Layer: API, CLI, and UI surfaces for external tooling and human-in-the-loop oversight.

- Data flow summary
  - Ingestion writes verbatim memory to MemPalace.
  - Graph Construction Layer extracts entities/relationships from memory and builds/maintains the knowledge graph.
  - Retrieval Layer reads from MemPalace and graph indices to produce candidate slices.
  - Reasoning Layer consumes retrieval outputs (memory chunks and graph subgraphs), executes agent evaluations, and emits interpretations with confidence signals.
  - Agent Orchestration Layer merges results, maintains belief-state, resolves conflicts across memory and graph representations.
  - Interface Layer exposes memory, graph, and reasoning results to clients and tooling.