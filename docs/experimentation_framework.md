Experimentation Framework

- Metrics
  - Retrieval accuracy (precision, recall, F1).
  - Graph accuracy (precision/recall of extracted entities/relationships; correctness of event sequencing).
  - Consistency between graph and memory (proportion of graph edges with supporting memory provenance).
  - Hallucination rate (graph-supported vs. unsupported conclusions).
  - Latency (vector, graph traversal, hybrid fusion).
  - Coherence over time.
  - Provenance completeness.
  - Conflict detection rate and resolution success.
  - Belief-state evolution stability.

- Testing strategies
  - Controlled memory sets with known entity/relationship ground truth.
  - Adversarial queries designed to probe graph traversal and fusion.
  - Long-context simulations with memory growth and graph evolution.
  - Conflict injection (memory vs. memory, memory vs. graph, graph vs. graph).

- Experimental workflow
  - Baseline measurements (phase by phase).
  - Graph integration experiments (entity extraction accuracy, graph completeness).
  - Hybrid retrieval experiments (precision/recall with fused ranking).
  - Graph-aware reasoning experiments (Graph Query Agent correctness).
  - Conflict resolution experiments (conflict detection and belief evolution).
  - Regression checks across all metrics.

- Graph-specific experiments
  - Extraction precision/recall on known entity sets.
  - Graph traversal correctness (path finding, shortest path).
  - Memory-graph linkage accuracy (confidence calibration).
  - Graph vs. memory provenance consistency.
  - Impact of graph on hallucination rate reduction.