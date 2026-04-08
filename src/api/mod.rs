use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Json, State},
    response::Json as AxumJson,
};
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;

use crate::storage::{MemoryStore, GraphStore};
use crate::retrieval::{RetrievalEngine, RetrievalResult, RetrievalQuery};
use crate::models::{MemoryObject, GraphNode, GraphEdge, GraphMetadata};

pub struct AppState {
    pub memory: Mutex<MemoryStore>,
    pub graph: Mutex<GraphStore>,
    pub retrieval: Mutex<RetrievalEngine>,
}

#[derive(Debug, Serialize)]
pub struct IngestResponse {
    pub memory_id: String,
    pub status: String,
}

#[derive(Debug, Deserialize)]
pub struct IngestRequest {
    pub source: String,
    pub format: String,
    pub content: String,
    pub tags: Vec<String>,
}

pub fn create_router(state: Arc<AppState>) -> Router {
    Router::new()
        .route("/health", get(health))
        .route("/api/v1/memory/ingest", post(ingest_memory))
        .route("/api/v1/memory/:memory_id", get(get_memory))
        .route("/api/v1/graph/nodes", post(create_node))
        .route("/api/v1/graph/nodes/:node_id", get(get_node))
        .route("/api/v1/graph/edges", post(create_edge))
        .route("/api/v1/graph/edges/:edge_id", get(get_edge))
        .route("/api/v1/graph/metadata", get(get_graph_metadata))
        .route("/api/v1/graph/traverse", post(traverse_graph))
        .route("/api/v1/graph/path", post(find_path))
        .route("/api/v1/retrieve", post(retrieve))
        .with_state(state)
}

async fn health() -> &'static str {
    "OK"
}

async fn ingest_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<AxumJson<IngestResponse>, String> {
    let mut memory = state.memory.lock().map_err(|e| e.to_string())?;
    let result = memory.ingest(req.source.clone(), req.format, req.content.clone(), req.tags.clone())
        .map_err(|e| e.to_string())?;
    
    let retrieval = state.retrieval.lock().map_err(|e| e.to_string())?;
    retrieval.index_memory(&result).map_err(|e| e.to_string())?;
    
    Ok(AxumJson(IngestResponse {
        memory_id: result.memory_id,
        status: "created".to_string(),
    }))
}

async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<String>,
) -> Result<AxumJson<MemoryObject>, String> {
    let memory = state.memory.lock().map_err(|e| e.to_string())?;
    let result = memory.get(&memory_id).map_err(|e| e.to_string())?;
    Ok(AxumJson(result))
}

#[derive(Debug, Deserialize)]
pub struct CreateNodeRequest {
    pub node_type: String,
    pub label: String,
    pub properties: serde_json::Value,
    pub provenance: Vec<String>,
}

async fn create_node(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateNodeRequest>,
) -> Result<AxumJson<GraphNode>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let node = graph.create_node(req.node_type, req.label, req.properties, req.provenance)
        .map_err(|e| e.to_string())?;
    Ok(AxumJson(node))
}

async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<AxumJson<GraphNode>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let node = graph.get_node(&node_id).map_err(|e| e.to_string())?;
    Ok(AxumJson(node))
}

#[derive(Debug, Deserialize)]
pub struct CreateEdgeRequest {
    pub source: String,
    pub target: String,
    pub relation: String,
    pub properties: serde_json::Value,
    pub provenance: Vec<String>,
    pub confidence: f64,
}

async fn create_edge(
    State(state): State<Arc<AppState>>,
    Json(req): Json<CreateEdgeRequest>,
) -> Result<AxumJson<GraphEdge>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let edge = graph.create_edge(req.source, req.target, req.relation, req.properties, req.provenance, req.confidence)
        .map_err(|e| e.to_string())?;
    Ok(AxumJson(edge))
}

async fn get_edge(
    State(state): State<Arc<AppState>>,
    Path(edge_id): Path<String>,
) -> Result<AxumJson<GraphEdge>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let edge = graph.get_edge(&edge_id).map_err(|e| e.to_string())?;
    Ok(AxumJson(edge))
}

async fn get_graph_metadata(
    State(state): State<Arc<AppState>>,
) -> Result<AxumJson<GraphMetadata>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let metadata = graph.get_metadata().map_err(|e| e.to_string())?;
    Ok(AxumJson(metadata))
}

#[derive(Debug, Deserialize)]
pub struct TraverseRequest {
    pub start_node_id: Option<String>,
    pub depth: Option<usize>,
    pub node_types: Option<Vec<String>>,
}

#[derive(Debug, Serialize)]
pub struct TraverseResponse {
    pub nodes: Vec<GraphNode>,
    pub edges: Vec<GraphEdge>,
}

async fn traverse_graph(
    State(state): State<Arc<AppState>>,
    Json(req): Json<TraverseRequest>,
) -> Result<AxumJson<TraverseResponse>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    
    let depth = req.depth.unwrap_or(3);
    let start_node_id = req.start_node_id.unwrap_or_default();
    
    let nodes = if !start_node_id.is_empty() {
        graph.traverse_bfs(&start_node_id, depth).map_err(|e| e.to_string())?
    } else {
        graph.query_nodes(None, None, 100).map_err(|e| e.to_string())?
    };
    
    let edges: Vec<GraphEdge> = nodes.iter()
        .flat_map(|n| graph.get_node_neighbors(&n.id, depth).unwrap_or_default())
        .collect();
    
    Ok(AxumJson(TraverseResponse { nodes, edges }))
}

#[derive(Debug, Deserialize)]
pub struct FindPathRequest {
    pub start_id: String,
    pub end_id: String,
    pub max_depth: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct FindPathResponse {
    pub path: Option<Vec<String>>,
}

async fn find_path(
    State(state): State<Arc<AppState>>,
    Json(req): Json<FindPathRequest>,
) -> Result<AxumJson<FindPathResponse>, String> {
    let graph = state.graph.lock().map_err(|e| e.to_string())?;
    let max_depth = req.max_depth.unwrap_or(5);
    
    let path = graph.find_path(&req.start_id, &req.end_id, max_depth)
        .map_err(|e| e.to_string())?;
    
    Ok(AxumJson(FindPathResponse { path }))
}

#[derive(Debug, Deserialize)]
pub struct RetrieveRequest {
    pub query: String,
    pub modes: Option<Vec<String>>,
    pub limit: Option<usize>,
}

#[derive(Debug, Serialize)]
pub struct RetrieveResponse {
    pub results: Vec<RetrievalResult>,
}

async fn retrieve(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RetrieveRequest>,
) -> Result<AxumJson<RetrieveResponse>, String> {
    let retrieval = state.retrieval.lock().map_err(|e| e.to_string())?;
    let limit = req.limit.unwrap_or(10);
    
    let results = retrieval.search(&req.query, limit)
        .map_err(|e| e.to_string())?;
    
    Ok(AxumJson(RetrieveResponse { results }))
}
