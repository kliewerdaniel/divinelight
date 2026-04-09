use crate::models::agent::AgentOutput;
use crate::models::belief::{BeliefState, ConflictFlag, Interpretation};
use crate::retrieval::RetrievalResult;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

pub struct ReasoningEngine;

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterpretRequest {
    pub query: String,
    pub context_ids: Option<Vec<String>>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct InterpretResponse {
    pub belief_state: BeliefState,
    pub interpretations: Vec<Interpretation>,
    pub provenance: Vec<String>,
}

impl ReasoningEngine {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn interpret(
        &self,
        query: &str,
        retrieval_results: Vec<RetrievalResult>,
    ) -> Result<InterpretResponse> {
        let belief_id = format!("belief_{}", &Uuid::new_v4().to_string()[..8]);

        let interpretations = self.generate_interpretations(query, &retrieval_results);

        let conflict_flags = self.detect_conflicts(&retrieval_results);

        let belief_state = BeliefState {
            belief_id: belief_id.clone(),
            timestamp: Utc::now(),
            interpretations: interpretations.clone(),
            conflict_flags: conflict_flags.clone(),
            state: "open".to_string(),
        };

        let provenance: Vec<String> = retrieval_results
            .iter()
            .flat_map(|r| r.provenance.clone())
            .collect();

        Ok(InterpretResponse {
            belief_state,
            interpretations,
            provenance,
        })
    }

    fn generate_interpretations(
        &self,
        _query: &str,
        results: &[RetrievalResult],
    ) -> Vec<Interpretation> {
        let mut interpretations = Vec::new();

        if results.is_empty() {
            return interpretations;
        }

        let top_result = &results[0];
        let summary = if let Some(ref memory) = top_result.memory {
            format!(
                "Found relevant memory: {}",
                &memory.content[..memory.content.len().min(100)]
            )
        } else {
            "No direct memory match found".to_string()
        };

        let interpretation = Interpretation {
            interpretation_id: format!("interp_{}", &Uuid::new_v4().to_string()[..8]),
            summary,
            confidence: top_result.confidence,
            supporting_memory_ids: results
                .iter()
                .filter_map(|r| r.memory.as_ref().map(|m| m.memory_id.clone()))
                .collect(),
            supporting_graph_edges: vec![],
            contradictions: vec![],
        };

        interpretations.push(interpretation);
        interpretations
    }

    fn detect_conflicts(&self, results: &[RetrievalResult]) -> Vec<ConflictFlag> {
        let mut conflicts = Vec::new();

        if results.len() < 2 {
            return conflicts;
        }

        let memories: Vec<_> = results.iter().filter_map(|r| r.memory.as_ref()).collect();

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                if self.has_contradiction(&memories[i].content, &memories[j].content) {
                    conflicts.push(ConflictFlag {
                        conflict_type: "memory_vs_memory".to_string(),
                        description: format!(
                            "Potential contradiction between {} and {}",
                            memories[i].memory_id, memories[j].memory_id
                        ),
                        affected_edges: vec![],
                    });
                }
            }
        }

        conflicts
    }

    fn has_contradiction(&self, content1: &str, content2: &str) -> bool {
        let contradictions = [
            ("yes", "no"),
            ("true", "false"),
            ("good", "bad"),
            ("agree", "disagree"),
            ("like", "hate"),
            ("increase", "decrease"),
        ];

        let c1_lower = content1.to_lowercase();
        let c2_lower = content2.to_lowercase();

        for (pos, neg) in &contradictions {
            if c1_lower.contains(pos) && c2_lower.contains(neg) {
                return true;
            }
            if c1_lower.contains(neg) && c2_lower.contains(pos) {
                return true;
            }
        }

        false
    }

    #[allow(dead_code)]
    pub fn create_graph_query_agent(&self) -> GraphQueryAgent {
        GraphQueryAgent::new()
    }
}

pub struct GraphQueryAgent;

#[allow(dead_code)]
impl GraphQueryAgent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(
        &self,
        query: &str,
        graph_nodes: &[crate::models::graph::GraphNode],
        graph_edges: &[crate::models::graph::GraphEdge],
    ) -> Result<AgentOutput> {
        let task_id = format!("task_{}", &Uuid::new_v4().to_string()[..8]);

        let relevant_nodes: Vec<_> = graph_nodes
            .iter()
            .filter(|n| {
                let label_lower = n.label.to_lowercase();
                let query_lower = query.to_lowercase();
                query_lower
                    .split_whitespace()
                    .any(|t| label_lower.contains(t))
            })
            .collect();

        let score = if relevant_nodes.is_empty() {
            0.0
        } else {
            0.5 + (relevant_nodes.len() as f64 * 0.1).min(0.4)
        };

        Ok(AgentOutput {
            agent_id: "graph_query_v1".to_string(),
            task_id,
            outputs: vec![crate::models::agent::AgentResult {
                memory_id: None,
                graph_subgraph: Some(crate::models::agent::GraphSubgraph {
                    nodes: relevant_nodes.iter().map(|n| n.id.clone()).collect(),
                    edges: graph_edges.iter().map(|e| e.id.clone()).collect(),
                }),
                score,
                metadata: crate::models::agent::AgentResultMetadata {
                    source: "graph_query".to_string(),
                    explanation: format!("Found {} relevant nodes for query", relevant_nodes.len()),
                    time_taken_ms: 1,
                },
            }],
            created_at: Utc::now(),
        })
    }
}
