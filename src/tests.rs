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
}
