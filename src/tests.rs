#[cfg(test)]
mod tests {
    use crate::agents::{ContradictionDetectorAgent, RetrieverAgent};
    use crate::models::graph::{GraphEdge, GraphNode};
    use crate::models::memory::MemoryObject;
    use crate::reasoning::ReasoningEngine;
    use crate::retrieval::{RetrievalEngine, RetrievalResult};
    use std::path::PathBuf;
    use tempfile::TempDir;

    #[test]
    fn test_memory_object_creation() {
        let memory = MemoryObject::new(
            "test_source".to_string(),
            "plaintext".to_string(),
            "Test content".to_string(),
            vec!["tag1".to_string(), "tag2".to_string()],
        );

        assert!(memory.memory_id.starts_with("mp_"));
        assert_eq!(memory.source, "test_source");
        assert_eq!(memory.content, "Test content");
        assert_eq!(memory.tags.len(), 2);
        assert!(memory.checksum.starts_with("sha256:"));
    }

    #[test]
    fn test_memory_checksum_verification() {
        let memory = MemoryObject::new(
            "test".to_string(),
            "plaintext".to_string(),
            "Test content".to_string(),
            vec![],
        );

        assert!(memory.verify());
    }

    #[test]
    fn test_reasoning_engine() {
        let engine = ReasoningEngine::new();

        let results = vec![RetrievalResult {
            memory: Some(MemoryObject::new(
                "test".to_string(),
                "plaintext".to_string(),
                "The project is going well".to_string(),
                vec!["status".to_string()],
            )),
            graph_node: None,
            graph_edge: None,
            score: 0.9,
            source: "memory".to_string(),
            provenance: vec!["test_id".to_string()],
            confidence: 0.9,
        }];

        let response = engine.interpret("project status", results).unwrap();

        assert!(response.belief_state.belief_id.starts_with("belief_"));
        assert!(!response.interpretations.is_empty());
        assert_eq!(response.interpretations[0].confidence, 0.9);
    }

    #[test]
    fn test_contradiction_detection() {
        let memories = vec![
            MemoryObject::new(
                "test".to_string(),
                "plaintext".to_string(),
                "Yes, I agree with this decision".to_string(),
                vec![],
            ),
            MemoryObject::new(
                "test".to_string(),
                "plaintext".to_string(),
                "No, I disagree with this decision".to_string(),
                vec![],
            ),
        ];

        let agent = ContradictionDetectorAgent::new();
        let output = agent.execute(memories.iter().collect()).unwrap();

        assert!(output.outputs.iter().any(|o| o.score > 0.0));
    }

    #[test]
    fn test_graph_node_creation() {
        use chrono::Utc;

        let node = GraphNode {
            id: "test-node-1".to_string(),
            node_type: "Person".to_string(),
            label: "John Doe".to_string(),
            properties: serde_json::json!({"age": 30}),
            provenance: vec!["mp_001".to_string()],
            version: "v1.0".to_string(),
            created_at: Utc::now(),
            updated_at: Utc::now(),
        };

        assert_eq!(node.node_type, "Person");
        assert_eq!(node.label, "John Doe");
    }

    #[test]
    fn test_retrieval_result_ordering() {
        let results = vec![
            RetrievalResult {
                memory: None,
                graph_node: None,
                graph_edge: None,
                score: 0.5,
                source: "memory".to_string(),
                provenance: vec![],
                confidence: 0.5,
            },
            RetrievalResult {
                memory: None,
                graph_node: None,
                graph_edge: None,
                score: 0.9,
                source: "memory".to_string(),
                provenance: vec![],
                confidence: 0.9,
            },
            RetrievalResult {
                memory: None,
                graph_node: None,
                graph_edge: None,
                score: 0.3,
                source: "memory".to_string(),
                provenance: vec![],
                confidence: 0.3,
            },
        ];

        let mut sorted = results.clone();
        sorted.sort_by(|a, b| b.score.partial_cmp(&a.score).unwrap());

        assert_eq!(sorted[0].score, 0.9);
        assert_eq!(sorted[1].score, 0.5);
        assert_eq!(sorted[2].score, 0.3);
    }

    #[test]
    fn test_belief_state_conflicts() {
        use crate::models::belief::{BeliefState, ConflictFlag};

        let belief = BeliefState {
            belief_id: "belief_001".to_string(),
            timestamp: chrono::Utc::now(),
            interpretations: vec![],
            conflict_flags: vec![ConflictFlag {
                conflict_type: "memory_vs_memory".to_string(),
                description: "Test conflict".to_string(),
                affected_edges: vec![],
            }],
            state: "open".to_string(),
        };

        assert_eq!(belief.conflict_flags.len(), 1);
        assert_eq!(belief.state, "open");
    }

    #[test]
    fn test_memory_store_ingest_and_get() {
        let dir = TempDir::new().unwrap();
        let mut store = crate::storage::MemoryStore::new(dir.path().to_path_buf()).unwrap();

        let memory = store
            .ingest(
                "test_source".to_string(),
                "plaintext".to_string(),
                "Hello world".to_string(),
                vec!["test".to_string()],
            )
            .unwrap();

        let retrieved = store.get(&memory.memory_id).unwrap();
        assert_eq!(retrieved.content, "Hello world");
        assert_eq!(retrieved.source, "test_source");
        assert!(retrieved.verify());
    }

    #[test]
    fn test_memory_store_count() {
        let dir = TempDir::new().unwrap();
        let mut store = crate::storage::MemoryStore::new(dir.path().to_path_buf()).unwrap();
        assert_eq!(store.count().unwrap(), 0);

        store
            .ingest("s".to_string(), "p".to_string(), "c1".to_string(), vec![])
            .unwrap();
        store
            .ingest("s".to_string(), "p".to_string(), "c2".to_string(), vec![])
            .unwrap();

        assert_eq!(store.count().unwrap(), 2);
    }

    #[test]
    fn test_graph_store_create_node_and_edge() {
        let dir = TempDir::new().unwrap();
        let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();

        let node_a = store
            .create_node(
                "Person".to_string(),
                "Alice".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();

        let node_b = store
            .create_node(
                "Person".to_string(),
                "Bob".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();

        let edge = store
            .create_edge(
                node_a.id.clone(),
                node_b.id.clone(),
                "knows".to_string(),
                serde_json::json!({}),
                vec![],
                0.9,
            )
            .unwrap();

        assert_eq!(edge.source, node_a.id);
        assert_eq!(edge.target, node_b.id);

        let meta = store.get_metadata().unwrap();
        assert_eq!(meta.node_count, 2);
        assert_eq!(meta.edge_count, 1);
    }

    #[test]
    fn test_graph_store_bfs_traversal() {
        let dir = TempDir::new().unwrap();
        let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();

        let a = store
            .create_node(
                "T".to_string(),
                "A".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();
        let b = store
            .create_node(
                "T".to_string(),
                "B".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();
        let c = store
            .create_node(
                "T".to_string(),
                "C".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();

        store
            .create_edge(
                a.id.clone(),
                b.id.clone(),
                "r".to_string(),
                serde_json::json!({}),
                vec![],
                1.0,
            )
            .unwrap();
        store
            .create_edge(
                b.id.clone(),
                c.id.clone(),
                "r".to_string(),
                serde_json::json!({}),
                vec![],
                1.0,
            )
            .unwrap();

        let traversed = store.traverse_bfs(&a.id, 2).unwrap();
        assert!(traversed.len() >= 3);
    }

    #[test]
    fn test_graph_store_find_path() {
        let dir = TempDir::new().unwrap();
        let store = crate::storage::GraphStore::new(dir.path().to_path_buf()).unwrap();

        let a = store
            .create_node(
                "T".to_string(),
                "A".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();
        let b = store
            .create_node(
                "T".to_string(),
                "B".to_string(),
                serde_json::json!({}),
                vec![],
            )
            .unwrap();

        store
            .create_edge(
                a.id.clone(),
                b.id.clone(),
                "r".to_string(),
                serde_json::json!({}),
                vec![],
                1.0,
            )
            .unwrap();

        let path = store.find_path(&a.id, &b.id, 5).unwrap();
        assert!(path.is_some());

        let no_path = store.find_path(&b.id, &a.id, 5).unwrap();
        assert!(no_path.is_none());
    }

    #[test]
    fn test_retrieval_engine_indexing_and_search() {
        let dir = TempDir::new().unwrap();
        let engine = crate::retrieval::RetrievalEngine::new(dir.path().to_path_buf()).unwrap();

        let memory = crate::models::MemoryObject::new(
            "test".to_string(),
            "plaintext".to_string(),
            "The quick brown fox".to_string(),
            vec!["animals".to_string()],
        );

        engine.index_memory(&memory).unwrap();

        std::fs::create_dir_all(dir.path().join("memories")).unwrap();
        let path = dir
            .path()
            .join("memories")
            .join(format!("{}.json", memory.memory_id));
        std::fs::write(&path, serde_json::to_string(&memory).unwrap()).unwrap();

        let results = engine.search("fox", 10).unwrap();
        assert!(!results.is_empty());
        assert!(results[0].score > 0.0);
    }

    #[test]
    fn test_retrieval_wildcard_returns_all() {
        let dir = TempDir::new().unwrap();
        let engine = crate::retrieval::RetrievalEngine::new(dir.path().to_path_buf()).unwrap();

        let m1 = crate::models::MemoryObject::new(
            "s".to_string(),
            "p".to_string(),
            "content one".to_string(),
            vec![],
        );
        let m2 = crate::models::MemoryObject::new(
            "s".to_string(),
            "p".to_string(),
            "content two".to_string(),
            vec![],
        );

        for m in [&m1, &m2] {
            engine.index_memory(m).unwrap();
            std::fs::create_dir_all(dir.path().join("memories")).unwrap();
            let path = dir
                .path()
                .join("memories")
                .join(format!("{}.json", m.memory_id));
            std::fs::write(&path, serde_json::to_string(m).unwrap()).unwrap();
        }

        let results = engine.search("*", 100).unwrap();
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_synthesizer_agent_empty_input() {
        use crate::agents::SynthesizerAgent;
        let agent = SynthesizerAgent::new();
        let output = agent.execute(vec![]).unwrap();
        assert_eq!(
            output.outputs[0].metadata.explanation,
            "No content to synthesize"
        );
    }

    #[test]
    fn test_verifier_agent_detects_tampered_memory() {
        use crate::agents::VerifierAgent;
        let mut memory = crate::models::MemoryObject::new(
            "test".to_string(),
            "plaintext".to_string(),
            "Original content".to_string(),
            vec![],
        );
        memory.content = "Tampered content".to_string();

        let agent = VerifierAgent::new();
        let output = agent.execute(vec![&memory]).unwrap();
        assert_eq!(output.outputs[0].score, 0.0);
        assert!(output.outputs[0].metadata.explanation.contains("failed"));
    }
}
