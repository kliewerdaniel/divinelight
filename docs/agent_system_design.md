Agent System Design

- Minimum viable agents
  - Retriever Agent
    - Purpose: perform initial retrieval using vector and symbolic signals; produce candidate memories with provenance.
    - Inputs: user query, memory store, embeddings index, tags/metadata.
    - Outputs: candidate memories with scores and sources.
    - Failure Modes: embedding drift, index staleness, cache issues.

  - Verifier Agent
    - Purpose: validate recalled items against question context; assess consistency and cross-checks.
    - Inputs: candidate memories, graph context, belief-state hints.
    - Outputs: validated memories with confidence, notes on potential contradictions.
    - Failure Modes: false positives/negatives due to validation rules.

  - Synthesizer Agent
    - Purpose: fuse validated memories into coherent interpretations; produce structured outputs.
    - Inputs: validated memories, belief-state, desired output format.
    - Outputs: synthesized interpretations with provenance and confidence.
    - Failure Modes: over-generalization; loss of verbatim fidelity.

  - Contradiction Detector Agent
    - Purpose: detect contradictions among competing interpretations.
    - Inputs: multiple interpretations, confidences, supporting memories.
    - Outputs: contradiction flags, arbitration notes, updated belief-state annotations.
    - Failure Modes: ambiguous contradictions; insufficient context for resolution.

- Interaction model
  - Agents operate in a staged workflow with central arbitration.
  - Disagreements resolved by: composite confidence ranking, cross-check with validators, optional human-in-the-loop.
  - Provenance and audit trails are preserved for all agent decisions.

- Arbitration protocol (high level)
  - Step 1: Retriever provides top-K candidates.
  - Step 2: Verifier scores and flags contradictions.
  - Step 3: Contradiction Detector analyzes and emits a conflict graph.
  - Step 4: Synthesizer produces provisional interpretations with confidence.
  - Step 5: Belief-state manager finalizes or revises based on arbitration.
