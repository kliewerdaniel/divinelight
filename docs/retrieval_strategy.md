Retrieval Strategy

- Vector-based retrieval
  - Purpose: semantic similarity using embeddings.
  - Use cases: long-tail associations, semantic matching to queries.
  - Merge: top-N by cosine similarity; normalize scores.
  - Ranking: combine similarity with temporal recency and relevance signals.

- Keyword / symbolic retrieval
  - Purpose: deterministic retrieval via tags, topics, and metadata.
  - Use cases: exact phrase matches, topic filters, known entities.
  - Merge: blend with vector results; prioritize exact matches.
  - Ranking: metadata-driven priorities; optional recency weighting.

- Graph traversal
  - Purpose: expand context through connected memories.
  - Use cases: follow semantic, temporal, causal edges to retrieve neighborhoods.
  - Merge: expand neighborhoods up to a depth; score edges by type/weight.
  - Ranking: path-based scoring using edge weights and node scores.

- Temporal slicing
  - Purpose: restrict recall to time windows for sequence reconstruction.
  - Use cases: dialogue sequences, event timelines.
  - Merge: combine with other modalities as needed; enforce time bounds.
  - Ranking: recency and continuity signals.

- Merging strategy
  - Stage 1: collect candidate sets from modalities.
  - Stage 2: deduplicate by memory_id.
  - Stage 3: compute unified score via fusion function (linear or learned).
 - Stage 4: top-K selection for reasoning.

- Explainability
  - Preserve provenance for each memory item to support auditing.
