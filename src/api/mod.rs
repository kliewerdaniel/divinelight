use axum::{
    Router,
    routing::{get, post},
    extract::{Path, Json, State},
    response::{Json as AxumJson, Response},
};
use axum::http::Method;
use axum::response::IntoResponse;
use serde::{Deserialize, Serialize};
use std::sync::Arc;
use std::sync::Mutex;
use tower_http::cors::{Any, CorsLayer};

use crate::storage::{MemoryStore, GraphStore};
use crate::retrieval::{RetrievalEngine, RetrievalResult};
use crate::reasoning::{ReasoningEngine, InterpretResponse};
use crate::agents::{RetrieverAgent, VerifierAgent, SynthesizerAgent, ContradictionDetectorAgent};
use crate::models::{MemoryObject, GraphNode, GraphEdge, GraphMetadata, AgentOutput, BeliefState};

pub struct AppError(anyhow::Error);

impl IntoResponse for AppError {
    fn into_response(self) -> Response {
        (axum::http::StatusCode::INTERNAL_SERVER_ERROR, format!("Error: {}", self.0)).into_response()
    }
}

impl<E: Into<anyhow::Error>> From<E> for AppError {
    fn from(err: E) -> Self {
        Self(err.into())
    }
}

pub struct AppState {
    pub memory: Mutex<MemoryStore>,
    pub graph: Mutex<GraphStore>,
    pub retrieval: Mutex<RetrievalEngine>,
    pub reasoning: Mutex<ReasoningEngine>,
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
    let cors = CorsLayer::new()
        .allow_origin(Any)
        .allow_methods([Method::GET, Method::POST, Method::OPTIONS])
        .allow_headers(Any);

    Router::new()
        .route("/health", get(health))
        .route("/api/v1/memory/ingest", post(ingest_memory))
        .route("/api/v1/memory/:memory_id", get(get_memory))
        .route("/api/v1/memory/list", get(list_memories))
        .route("/api/v1/graph/nodes", post(create_node))
        .route("/api/v1/graph/nodes/:node_id", get(get_node))
        .route("/api/v1/graph/edges", post(create_edge))
        .route("/api/v1/graph/edges/:edge_id", get(get_edge))
        .route("/api/v1/graph/metadata", get(get_graph_metadata))
        .route("/api/v1/graph/traverse", post(traverse_graph))
        .route("/api/v1/graph/path", post(find_path))
        .route("/api/v1/retrieve", post(retrieve))
        .route("/api/v1/reason/interpret", post(interpret))
        .route("/api/v1/reason/beliefs/:belief_id", get(get_belief))
        .route("/api/v1/reason/conflicts", post(detect_conflicts))
        .route("/api/v1/agents/retriever", post(run_retriever))
        .route("/api/v1/agents/verifier", post(run_verifier))
        .route("/api/v1/agents/synthesizer", post(run_synthesizer))
        .route("/api/v1/agents/contradiction", post(run_contradiction_detector))
        .route("/api/v1/backup/create", post(create_backup))
        .route("/api/v1/backup/restore", post(restore_backup))
        .route("/api/v1/backup/export", post(export_data))
        .route("/api/v1/backup/import", post(import_data))
        .with_state(state)
        .layer(cors)
}

#[derive(Debug, Serialize)]
struct HealthResponse {
    status: String,
    version: String,
}

async fn health() -> AxumJson<HealthResponse> {
    AxumJson(HealthResponse {
        status: "ok".to_string(),
        version: env!("CARGO_PKG_VERSION").to_string(),
    })
}

async fn ingest_memory(
    State(state): State<Arc<AppState>>,
    Json(req): Json<IngestRequest>,
) -> Result<AxumJson<IngestResponse>, AppError> {
    let mut memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let result = memory.ingest(req.source.clone(), req.format, req.content.clone(), req.tags.clone())?;
    
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    retrieval.index_memory(&result)?;
    
    Ok(AxumJson(IngestResponse {
        memory_id: result.memory_id,
        status: "created".to_string(),
    }))
}

async fn get_memory(
    State(state): State<Arc<AppState>>,
    Path(memory_id): Path<String>,
) -> Result<AxumJson<MemoryObject>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let result = memory.get(&memory_id)?;
    Ok(AxumJson(result))
}

#[derive(Debug, Deserialize)]
struct ListMemoriesQuery {
    limit: Option<usize>,
    offset: Option<usize>,
}

#[derive(Debug, Serialize)]
struct ListMemoriesResponse {
    memories: Vec<MemoryObject>,
    total: u64,
}

async fn list_memories(
    State(state): State<Arc<AppState>>,
    axum::extract::Query(params): axum::extract::Query<ListMemoriesQuery>,
) -> Result<AxumJson<ListMemoriesResponse>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let limit = params.limit.unwrap_or(50);
    let offset = params.offset.unwrap_or(0);
    let memories = memory.list_all(limit, offset)?;
    let total = memory.count()?;
    Ok(AxumJson(ListMemoriesResponse { memories, total }))
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
) -> Result<AxumJson<GraphNode>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let node = graph.create_node(req.node_type, req.label, req.properties, req.provenance)?;
    Ok(AxumJson(node))
}

async fn get_node(
    State(state): State<Arc<AppState>>,
    Path(node_id): Path<String>,
) -> Result<AxumJson<GraphNode>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let node = graph.get_node(&node_id)?;
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
) -> Result<AxumJson<GraphEdge>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let edge = graph.create_edge(req.source, req.target, req.relation, req.properties, req.provenance, req.confidence)?;
    Ok(AxumJson(edge))
}

async fn get_edge(
    State(state): State<Arc<AppState>>,
    Path(edge_id): Path<String>,
) -> Result<AxumJson<GraphEdge>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let edge = graph.get_edge(&edge_id)?;
    Ok(AxumJson(edge))
}

async fn get_graph_metadata(
    State(state): State<Arc<AppState>>,
) -> Result<AxumJson<GraphMetadata>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let metadata = graph.get_metadata()?;
    Ok(AxumJson(metadata))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
) -> Result<AxumJson<TraverseResponse>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let depth = req.depth.unwrap_or(3);
    let start_node_id = req.start_node_id.unwrap_or_default();
    
    let nodes = if !start_node_id.is_empty() {
        graph.traverse_bfs(&start_node_id, depth)?
    } else {
        graph.query_nodes(None, None, 100)?
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
) -> Result<AxumJson<FindPathResponse>, AppError> {
    let graph = state.graph.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let max_depth = req.max_depth.unwrap_or(5);
    let path = graph.find_path(&req.start_id, &req.end_id, max_depth)?;
    Ok(AxumJson(FindPathResponse { path }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
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
) -> Result<AxumJson<RetrieveResponse>, AppError> {
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let limit = req.limit.unwrap_or(10);
    let results = retrieval.search(&req.query, limit)?;
    Ok(AxumJson(RetrieveResponse { results }))
}

#[derive(Debug, Deserialize)]
#[allow(dead_code)]
pub struct InterpretRequest {
    pub query: String,
    pub context_ids: Option<Vec<String>>,
}

async fn interpret(
    State(state): State<Arc<AppState>>,
    Json(req): Json<InterpretRequest>,
) -> Result<AxumJson<InterpretResponse>, AppError> {
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let results = retrieval.search(&req.query, 10)?;
    
    let reasoning = state.reasoning.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let response = reasoning.interpret(&req.query, results)?;
    
    Ok(AxumJson(response))
}

async fn get_belief(
    State(_state): State<Arc<AppState>>,
    Path(_belief_id): Path<String>,
) -> Result<AxumJson<BeliefState>, AppError> {
    Ok(AxumJson(BeliefState {
        belief_id: _belief_id,
        timestamp: chrono::Utc::now(),
        interpretations: vec![],
        conflict_flags: vec![],
        state: "open".to_string(),
    }))
}

#[derive(Debug, Deserialize)]
pub struct DetectConflictsRequest {
    pub memory_ids: Vec<String>,
}

#[derive(Debug, Serialize)]
pub struct DetectConflictsResponse {
    pub conflicts: Vec<String>,
}

async fn detect_conflicts(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DetectConflictsRequest>,
) -> Result<AxumJson<DetectConflictsResponse>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut memories = Vec::new();
    
    for id in &req.memory_ids {
        if let Ok(m) = memory.get(id) {
            memories.push(m);
        }
    }
    
    let detector = ContradictionDetectorAgent::new();
    let agent_output = detector.execute(memories.iter().collect())?;
    
    let conflicts: Vec<String> = agent_output.outputs.iter()
        .filter(|o| o.score > 0.0)
        .map(|o| o.metadata.explanation.clone())
        .collect();
    
    Ok(AxumJson(DetectConflictsResponse { conflicts }))
}

async fn run_retriever(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RetrieveRequest>,
) -> Result<AxumJson<AgentOutput>, AppError> {
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let results = retrieval.search(&req.query, req.limit.unwrap_or(10))?;
    
    let agent = RetrieverAgent::new();
    let output = agent.execute(&req.query, results)?;
    
    Ok(AxumJson(output))
}

async fn run_verifier(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DetectConflictsRequest>,
) -> Result<AxumJson<AgentOutput>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut memories = Vec::new();
    
    for id in &req.memory_ids {
        if let Ok(m) = memory.get(id) {
            memories.push(m);
        }
    }
    
    let agent = VerifierAgent::new();
    let output = agent.execute(memories.iter().collect())?;
    
    Ok(AxumJson(output))
}

async fn run_synthesizer(
    State(state): State<Arc<AppState>>,
    Json(req): Json<RetrieveRequest>,
) -> Result<AxumJson<AgentOutput>, AppError> {
    let retrieval = state.retrieval.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let results = retrieval.search(&req.query, req.limit.unwrap_or(10))?;
    
    let agent = SynthesizerAgent::new();
    let output = agent.execute(results)?;
    
    Ok(AxumJson(output))
}

async fn run_contradiction_detector(
    State(state): State<Arc<AppState>>,
    Json(req): Json<DetectConflictsRequest>,
) -> Result<AxumJson<AgentOutput>, AppError> {
    let memory = state.memory.lock().map_err(|e| anyhow::anyhow!("{}", e))?;
    let mut memories = Vec::new();
    
    for id in &req.memory_ids {
        if let Ok(m) = memory.get(id) {
            memories.push(m);
        }
    }
    
    let agent = ContradictionDetectorAgent::new();
    let output = agent.execute(memories.iter().collect())?;
    
    Ok(AxumJson(output))
}

use crate::backup::{BackupManager, BackupManifest};

#[derive(Debug, Deserialize)]
pub struct BackupRequest {
    pub path: String,
}

#[derive(Debug, Serialize)]
pub struct BackupResponse {
    pub status: String,
    pub manifest: Option<BackupManifest>,
}

async fn create_backup(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<AxumJson<BackupResponse>, AppError> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("divinelight");
    
    let backup_path = std::path::PathBuf::from(&req.path);
    let manager = BackupManager::new(data_dir);
    
    let manifest = manager.create_backup(&backup_path)?;
    Ok(AxumJson(BackupResponse {
        status: "success".to_string(),
        manifest: Some(manifest),
    }))
}

async fn restore_backup(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<AxumJson<BackupResponse>, AppError> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("divinelight");
    
    let backup_path = std::path::PathBuf::from(&req.path);
    let manager = BackupManager::new(data_dir);
    
    manager.restore_backup(&backup_path)?;
    Ok(AxumJson(BackupResponse {
        status: "success".to_string(),
        manifest: None,
    }))
}

async fn export_data(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<AxumJson<BackupResponse>, AppError> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("divinelight");
    
    let export_path = std::path::PathBuf::from(&req.path);
    let manager = BackupManager::new(data_dir);
    
    manager.export_data(&export_path)?;
    Ok(AxumJson(BackupResponse {
        status: "success".to_string(),
        manifest: None,
    }))
}

async fn import_data(
    State(_state): State<Arc<AppState>>,
    Json(req): Json<BackupRequest>,
) -> Result<AxumJson<BackupResponse>, AppError> {
    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("divinelight");
    
    let import_path = std::path::PathBuf::from(&req.path);
    let manager = BackupManager::new(data_dir);
    
    let count = manager.import_data(&import_path)?;
    Ok(AxumJson(BackupResponse {
        status: format!("Imported {} memories", count),
        manifest: None,
    }))
}
