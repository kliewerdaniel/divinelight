use crate::models::agent::{AgentOutput, AgentResult, AgentResultMetadata};
use crate::models::memory::MemoryObject;
use crate::retrieval::RetrievalResult;
use anyhow::Result;
use chrono::Utc;
use uuid::Uuid;

pub struct RetrieverAgent;

#[allow(dead_code)]
impl RetrieverAgent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, _query: &str, results: Vec<RetrievalResult>) -> Result<AgentOutput> {
        let task_id = format!("task_{}", &Uuid::new_v4().to_string()[..8]);

        let outputs: Vec<AgentResult> = results
            .iter()
            .map(|r| AgentResult {
                memory_id: r.memory.as_ref().map(|m| m.memory_id.clone()),
                graph_subgraph: None,
                score: r.score,
                metadata: AgentResultMetadata {
                    source: r.source.clone(),
                    explanation: format!("Retrieved with score {}", r.score),
                    time_taken_ms: 1,
                },
            })
            .collect();

        Ok(AgentOutput {
            agent_id: "retriever_v1".to_string(),
            task_id,
            outputs,
            created_at: Utc::now(),
        })
    }
}

pub struct VerifierAgent;

#[allow(dead_code)]
impl VerifierAgent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, memories: Vec<&MemoryObject>) -> Result<AgentOutput> {
        let task_id = format!("task_{}", &Uuid::new_v4().to_string()[..8]);

        let outputs: Vec<AgentResult> = memories
            .iter()
            .map(|m| {
                let is_verified = m.verify();
                AgentResult {
                    memory_id: Some(m.memory_id.clone()),
                    graph_subgraph: None,
                    score: if is_verified { 1.0 } else { 0.0 },
                    metadata: AgentResultMetadata {
                        source: "verifier".to_string(),
                        explanation: if is_verified {
                            "Checksum verified".to_string()
                        } else {
                            "Checksum verification failed".to_string()
                        },
                        time_taken_ms: 1,
                    },
                }
            })
            .collect();

        Ok(AgentOutput {
            agent_id: "verifier_v1".to_string(),
            task_id,
            outputs,
            created_at: Utc::now(),
        })
    }
}

pub struct SynthesizerAgent;

#[allow(dead_code)]
impl SynthesizerAgent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, results: Vec<RetrievalResult>) -> Result<AgentOutput> {
        let task_id = format!("task_{}", &Uuid::new_v4().to_string()[..8]);

        let all_content: Vec<String> = results
            .iter()
            .filter_map(|r| r.memory.as_ref().map(|m| m.content.clone()))
            .collect();

        let summary = if all_content.is_empty() {
            "No content to synthesize".to_string()
        } else {
            let combined = all_content.join(" ");
            let preview = if combined.len() > 200 {
                format!("{}...", &combined[..200])
            } else {
                combined
            };
            format!("Synthesized from {} sources: {}", results.len(), preview)
        };

        let avg_score = if results.is_empty() {
            0.0
        } else {
            results.iter().map(|r| r.score).sum::<f64>() / results.len() as f64
        };

        Ok(AgentOutput {
            agent_id: "synthesizer_v1".to_string(),
            task_id,
            outputs: vec![AgentResult {
                memory_id: None,
                graph_subgraph: None,
                score: avg_score,
                metadata: AgentResultMetadata {
                    source: "synthesizer".to_string(),
                    explanation: summary,
                    time_taken_ms: 1,
                },
            }],
            created_at: Utc::now(),
        })
    }
}

pub struct ContradictionDetectorAgent;

#[allow(dead_code)]
impl ContradictionDetectorAgent {
    #[allow(clippy::new_without_default)]
    pub fn new() -> Self {
        Self
    }

    pub fn execute(&self, memories: Vec<&MemoryObject>) -> Result<AgentOutput> {
        let task_id = format!("task_{}", &Uuid::new_v4().to_string()[..8]);

        let mut outputs = Vec::new();
        let contradictions = [
            ("yes", "no"),
            ("true", "false"),
            ("good", "bad"),
            ("agree", "disagree"),
            ("like", "hate"),
            ("increase", "decrease"),
        ];

        for i in 0..memories.len() {
            for j in (i + 1)..memories.len() {
                let c1 = memories[i].content.to_lowercase();
                let c2 = memories[j].content.to_lowercase();

                let mut has_contradiction = false;
                for (pos, neg) in &contradictions {
                    if (c1.contains(pos) && c2.contains(neg))
                        || (c1.contains(neg) && c2.contains(pos))
                    {
                        has_contradiction = true;
                        break;
                    }
                }

                outputs.push(AgentResult {
                    memory_id: Some(memories[i].memory_id.clone()),
                    graph_subgraph: None,
                    score: if has_contradiction { 1.0 } else { 0.0 },
                    metadata: AgentResultMetadata {
                        source: "contradiction_detector".to_string(),
                        explanation: if has_contradiction {
                            format!("Contradiction detected with {}", memories[j].memory_id)
                        } else {
                            "No contradiction".to_string()
                        },
                        time_taken_ms: 1,
                    },
                });
            }
        }

        Ok(AgentOutput {
            agent_id: "contradiction_detector_v1".to_string(),
            task_id,
            outputs,
            created_at: Utc::now(),
        })
    }
}
