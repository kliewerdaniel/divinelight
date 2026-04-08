Frontend Design

- Goals
  - Provide an intuitive, responsive UI for ingestion, recall, graph exploration, and belief-state inspection.
- Core UI components
  - IngestionPanel: dropzone/file picker to ingest memories; metadata tagging.
  - RecallPanel: query input with live results; shows candidate memories with scores and provenance.
  - GraphViewer: interactive memory graph with nodes/edges; supports filtering by type and time.
  - BeliefStatePanel: visualizes current interpretations, confidence, and historical changes.
  - AgentDiagnostics: shows outputs from Retriever, Verifier, Synthesizer, and Contradiction Detector.
  - Settings: local storage path, embedding model selection, archiving policies.
- Data flow
  - UI calls backend API endpoints; results render in panels with provenance and confidence.
- Accessibility & UX
  - Keyboard navigation, scalable visuals for large graphs, responsive layout for multiple window sizes.
- Local-first considerations
  - All data fetched locally; offline-first operation; minimal network footprint.
