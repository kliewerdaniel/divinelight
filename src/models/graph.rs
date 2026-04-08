use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphNode {
    pub id: String,
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub provenance: Vec<String>,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphEdge {
    pub id: String,
    pub source: String,
    pub target: String,
    pub relation: String,
    pub properties: serde_json::Value,
    pub provenance: Vec<String>,
    pub confidence: f64,
    pub version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryGraphLink {
    pub memory_id: String,
    pub linked_nodes: Vec<String>,
    pub linked_edges: Vec<String>,
    pub link_type: String,
    pub confidence: f64,
    pub justification: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GraphMetadata {
    pub graph_id: String,
    pub schema_version: String,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub node_count: u64,
    pub edge_count: u64,
    pub retention_policy: String,
}
