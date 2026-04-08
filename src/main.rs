use std::net::SocketAddr;
use std::sync::Arc;
use std::sync::Mutex;
use axum::Router;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod storage;
mod api;
mod models;
mod retrieval;
mod reasoning;
mod agents;

use api::{AppState, create_router};
use storage::{MemoryStore, GraphStore};
use retrieval::RetrievalEngine;
use reasoning::ReasoningEngine;

#[tokio::main]
async fn main() {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| "divinelight=debug,tower_http=debug".into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    tracing::info!("Starting DivineLight server");

    let data_dir = dirs::data_local_dir()
        .unwrap_or_else(|| std::path::PathBuf::from("."))
        .join("divinelight");

    tracing::info!("Data directory: {:?}", data_dir);

    let memory_store = MemoryStore::new(data_dir.clone())
        .expect("Failed to initialize memory store");
    let graph_store = GraphStore::new(data_dir.clone())
        .expect("Failed to initialize graph store");
    let retrieval_engine = RetrievalEngine::new(data_dir.clone())
        .expect("Failed to initialize retrieval engine");
    let reasoning_engine = ReasoningEngine::new();

    let state = Arc::new(AppState {
        memory: Mutex::new(memory_store),
        graph: Mutex::new(graph_store),
        retrieval: Mutex::new(retrieval_engine),
        reasoning: Mutex::new(reasoning_engine),
    });

    let app = create_router(state).layer(TraceLayer::new_for_http());

    let addr = SocketAddr::from(([127, 0, 0, 1], 8080));
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
