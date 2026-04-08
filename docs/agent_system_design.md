Agent System Design

- Minimum viable agents
  - Retriever Agent
    - Purpose: perform initial retrieval using vector, symbolic, and graph signals; produce candidate memories and graph subgraphs with provenance.
    - Inputs: user query, memory store, graph store, embeddings index, tags/metadata, graph indices.
    - Outputs: candidate memories with scores and sources; graph subgraphs with confidence.
    - Failure Modes: embedding drift, index staleness, cache issues, missing graph context.

  - Verifier Agent
    - Purpose: validate recalled items against question context; assess consistency across memory and graph.
    - Inputs: candidate memories, graph subgraphs, belief-state hints.
    - Outputs: validated memories and graph elements with confidence, notes on potential contradictions.
    - Failure Modes: false positives/negatives due to validation rules; graph-memory inconsistencies.

  - Graph Query Agent
    - Purpose: perform graph-centric reasoning, path finding, subgraph interpretation, and missing-relation discovery.
    - Inputs: graph store, query patterns, confidence thresholds, candidate memory context.
    - Outputs: graph query results (paths, subgraphs), proposed candidate edges/nodes, missing-relation proposals.
    - Failure Modes: traversal cycles, query pattern mismatches, missing confidence thresholds.

  - Synthesizer Agent
    - Purpose: fuse validated memories and graph subgraphs into coherent interpretations; produce structured outputs.
    - Inputs: validated memories, validated graph subgraphs, belief-state, desired output format.
    - Outputs: synthesized interpretations with provenance and confidence.
    - Failure Modes: over-generalization; loss of verbatim fidelity; graph-memory fusion errors.

  - Contradiction Detector Agent
    - Purpose: detect contradictions among competing interpretations across memory and graph representations.
    - Inputs: multiple interpretations, confidences, supporting memories, supporting graph edges.
    - Outputs: contradiction flags (memory vs. memory, memory vs. graph, graph vs. graph), arbitration notes, updated belief-state annotations.
    - Failure Modes: ambiguous contradictions; insufficient context for resolution.

- Interaction model
  - Agents operate in a staged workflow with central arbitration.
  - Disagreements resolved by: composite confidence ranking, cross-check with validators, graph path corroboration, optional human-in-the-loop.
  - Provenance and audit trails are preserved for all agent decisions.
  - Graph Query Agent operates in parallel with Retriever to augment context before Synthesis.

- Arbitration protocol (high level)
  - Step 1: Retriever provides top-K candidates from memory and graph.
  - Step 2: Graph Query Agent extracts relevant subgraphs and proposes missing relations.
  - Step 3: Verifier scores and flags contradictions across sources.
  - Step 4: Contradiction Detector analyzes and emits a conflict graph.
  - Step 5: Synthesizer produces provisional interpretations with confidence.
  - Step 6: Belief-state manager finalizes or revises based on arbitration.