use std::sync::Arc;
use std::sync::Mutex;
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

mod storage;
mod api;
mod models;
mod retrieval;
mod reasoning;
mod agents;
mod backup;
mod config;

use config::Config;
use api::{AppState, create_router};
use storage::{MemoryStore, GraphStore};
use retrieval::RetrievalEngine;
use reasoning::ReasoningEngine;
use crate::backup::BackupManager;

#[tokio::main]
async fn main() {
    let config = Config::from_env();

    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env()
                .unwrap_or_else(|_| config.log_level.clone().into()),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args: Vec<String> = std::env::args().collect();
    
    if args.len() > 1 {
        match args[1].as_str() {
            "backup" => {
                if let Some(path) = args.get(2) {
                    let manager = BackupManager::new(config.data_dir.clone());
                    match manager.create_backup(&std::path::PathBuf::from(path)) {
                        Ok(manifest) => {
                            println!("Backup created successfully!");
                            println!("Memories: {}, Nodes: {}, Edges: {}", 
                                manifest.memory_count, manifest.node_count, manifest.edge_count);
                        }
                        Err(e) => println!("Backup failed: {}", e),
                    }
                } else {
                    println!("Usage: divinelight backup <path>");
                }
                return;
            }
            "restore" => {
                if let Some(path) = args.get(2) {
                    let manager = BackupManager::new(config.data_dir.clone());
                    match manager.restore_backup(&std::path::PathBuf::from(path)) {
                        Ok(_) => println!("Restore completed successfully!"),
                        Err(e) => println!("Restore failed: {}", e),
                    }
                } else {
                    println!("Usage: divinelight restore <path>");
                }
                return;
            }
            "export" => {
                if let Some(path) = args.get(2) {
                    let manager = BackupManager::new(config.data_dir.clone());
                    match manager.export_data(&std::path::PathBuf::from(path)) {
                        Ok(_) => println!("Export completed successfully!"),
                        Err(e) => println!("Export failed: {}", e),
                    }
                } else {
                    println!("Usage: divinelight export <path>");
                }
                return;
            }
            "import" => {
                if let Some(path) = args.get(2) {
                    let manager = BackupManager::new(config.data_dir.clone());
                    match manager.import_data(&std::path::PathBuf::from(path)) {
                        Ok(count) => println!("Imported {} memories", count),
                        Err(e) => println!("Import failed: {}", e),
                    }
                } else {
                    println!("Usage: divinelight import <path>");
                }
                return;
            }
            "status" => {
                let _manager = BackupManager::new(config.data_dir.clone());
                println!("Data directory: {:?}", config.data_dir);
                println!("Status: Running");
                return;
            }
            _ => {
                println!("Unknown command. Use: backup, restore, export, import, or status");
                return;
            }
        }
    }

    tracing::info!("Starting DivineLight server");
    tracing::info!("Data directory: {:?}", config.data_dir);

    let memory_store = MemoryStore::new(config.data_dir.clone())
        .expect("Failed to initialize memory store");
    let graph_store = GraphStore::new(config.data_dir.clone())
        .expect("Failed to initialize graph store");
    let retrieval_engine = RetrievalEngine::new(config.data_dir.clone())
        .expect("Failed to initialize retrieval engine");
    let reasoning_engine = ReasoningEngine::new();

    let state = Arc::new(AppState {
        memory: Mutex::new(memory_store),
        graph: Mutex::new(graph_store),
        retrieval: Mutex::new(retrieval_engine),
        reasoning: Mutex::new(reasoning_engine),
    });

    let app = create_router(state).layer(TraceLayer::new_for_http());

    let addr = config.socket_addr();
    tracing::info!("Listening on {}", addr);

    let listener = tokio::net::TcpListener::bind(addr).await.unwrap();
    axum::serve(listener, app).await.unwrap();
}
