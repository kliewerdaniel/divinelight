use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BeliefState {
    pub belief_id: String,
    pub timestamp: DateTime<Utc>,
    pub interpretations: Vec<Interpretation>,
    pub conflict_flags: Vec<ConflictFlag>,
    pub state: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Interpretation {
    pub interpretation_id: String,
    pub summary: String,
    pub confidence: f64,
    pub supporting_memory_ids: Vec<String>,
    pub supporting_graph_edges: Vec<String>,
    pub contradictions: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConflictFlag {
    pub conflict_type: String,
    pub description: String,
    pub affected_edges: Vec<String>,
}
