Retrieval Strategy

- Hybrid Retrieval Sources
  - Vector-based retrieval
    - Purpose: semantic similarity using embeddings on raw memory.
    - Use cases: long-tail associations, semantic matching to queries.
    - Merge: top-N by cosine similarity; normalize scores.
    - Ranking: combine similarity with temporal recency and relevance signals.

  - Keyword / symbolic retrieval
    - Purpose: deterministic retrieval via tags, topics, and metadata.
    - Use cases: exact phrase matches, topic filters, known entities.
    - Merge: blend with vector results; prioritize exact matches.
    - Ranking: metadata-driven priorities; optional recency weighting.

  - Graph traversal
    - Purpose: expand context through connected graph nodes and edges.
    - Use cases: follow semantic, temporal, causal edges to retrieve neighborhoods, subgraph expansion.
    - Merge: expand neighborhoods up to a depth; score edges by type/weight/confidence.
    - Ranking: path-based scoring using edge weights, node scores, and confidence.
    - Supports multi-hop traversal, shortest-path, and motif queries.

  - Temporal slicing
    - Purpose: restrict recall to time windows for sequence reconstruction.
    - Use cases: dialogue sequences, event timelines.
    - Merge: combine with other modalities as needed; enforce time bounds.
    - Ranking: recency and continuity signals.

- Fusion & Ranking
  - Stage 1: collect candidate sets from vector, keyword/graph, and temporal modalities.
  - Stage 2: reconcile overlapping results; deduplicate by memory_id and graph element.
  - Stage 3: compute unified score via fusion function (linear or learned) across sources.
  - Stage 4: conflict-aware ranking; deprioritize items with known conflicts or low confidence.
  - Stage 5: provenance-aware presentation; expose source (memory chunk, graph node/edge) for each result.

- Explainability
  - Preserve provenance for each memory item and graph element to support auditing.
  - Include confidence scores and conflict flags in retrieval outputs.
  - Track source modality (vector, graph, temporal) for each result item.

- Graph-Aware Retrieval Extensions
  - Subgraph extraction by thematic filters or query patterns.
  - Cross-referencing between memory IDs and graph elements.
  - Temporal constraints on graph edges (temporal_bounds in edge properties).
  - Confidence-weighted path scoring.