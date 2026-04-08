use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentOutput {
    pub agent_id: String,
    pub task_id: String,
    pub outputs: Vec<AgentResult>,
    pub created_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResult {
    pub memory_id: Option<String>,
    pub graph_subgraph: Option<GraphSubgraph>,
    pub score: f64,
    pub metadata: AgentResultMetadata,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphSubgraph {
    pub nodes: Vec<String>,
    pub edges: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AgentResultMetadata {
    pub source: String,
    pub explanation: String,
    pub time_taken_ms: u64,
}
