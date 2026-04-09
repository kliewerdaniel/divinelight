use crate::models::graph::{GraphEdge, GraphNode};
use crate::models::memory::MemoryObject;
use anyhow::Result;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;

pub struct RetrievalEngine {
    db: Connection,
    data_dir: PathBuf,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalResult {
    pub memory: Option<MemoryObject>,
    pub graph_node: Option<GraphNode>,
    pub graph_edge: Option<GraphEdge>,
    pub score: f64,
    pub source: String,
    pub provenance: Vec<String>,
    pub confidence: f64,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct RetrievalQuery {
    pub query: String,
    pub modes: Vec<String>,
    pub limit: usize,
    pub filters: Option<QueryFilters>,
}

#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct QueryFilters {
    pub sources: Option<Vec<String>>,
    pub tags: Option<Vec<String>>,
    pub node_types: Option<Vec<String>>,
    pub start_date: Option<String>,
    pub end_date: Option<String>,
}

impl RetrievalEngine {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        let db_path = data_dir.join("retrieval.db");
        let db = Connection::open(&db_path)?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS search_index (
                memory_id TEXT PRIMARY KEY,
                content TEXT NOT NULL,
                tags TEXT NOT NULL,
                source TEXT NOT NULL,
                created_at TEXT NOT NULL
            )",
            [],
        )?;

        Ok(Self { db, data_dir })
    }

    pub fn index_memory(&self, memory: &MemoryObject) -> Result<()> {
        self.db.execute(
            "INSERT OR IGNORE INTO search_index (memory_id, content, tags, source, created_at)
             VALUES (?1, ?2, ?3, ?4, ?5)",
            params![
                memory.memory_id,
                memory.content.clone(),
                serde_json::to_string(&memory.tags)?,
                memory.source.clone(),
                memory.created_at.to_rfc3339(),
            ],
        )?;
        Ok(())
    }

    pub fn search(&self, query: &str, limit: usize) -> Result<Vec<RetrievalResult>> {
        let query_lower = query.to_lowercase();
        let mut stmt = self
            .db
            .prepare("SELECT memory_id, content, tags, source, created_at FROM search_index")?;

        let all_memories: Vec<(String, String, String, String, String)> = stmt
            .query_map([], |row| {
                Ok((
                    row.get(0)?,
                    row.get(1)?,
                    row.get(2)?,
                    row.get(3)?,
                    row.get(4)?,
                ))
            })?
            .filter_map(|r| r.ok())
            .collect();

        tracing::debug!("Found {} memories in index", all_memories.len());

        let mut results = Vec::new();
        for (memory_id, content, tags, _source, _created_at) in all_memories {
            let content_lower = content.to_lowercase();
            let score = self.calculate_score(&query_lower, &content_lower, &tags);
            tracing::debug!("Memory {} score: {}", memory_id, score);
            if score > 0.0 {
                let memory = self.load_memory(&memory_id)?;
                results.push(RetrievalResult {
                    memory,
                    graph_node: None,
                    graph_edge: None,
                    score,
                    source: "memory".to_string(),
                    provenance: vec![memory_id.clone()],
                    confidence: score,
                });
            }
        }

        results.sort_by(|a, b| {
            b.score
                .partial_cmp(&a.score)
                .unwrap_or(std::cmp::Ordering::Equal)
        });
        results.truncate(limit);
        tracing::debug!("Returning {} results", results.len());
        Ok(results)
    }

    fn calculate_score(&self, query: &str, content: &str, tags: &str) -> f64 {
        if query == "*" || query.trim().is_empty() {
            return 0.5;
        }

        let query_terms: Vec<&str> = query.split_whitespace().collect();
        let mut score = 0.0;

        for term in query_terms {
            if content.contains(term) {
                score += 1.0;
            }
            if tags.contains(term) {
                score += 0.5;
            }
        }

        if score > 0.0 {
            let content_len = content.split_whitespace().count() as f64;
            score + (1.0 / (1.0 + (content_len / 100.0 + 1.0).ln()))
        } else {
            0.0
        }
    }

    fn load_memory(&self, memory_id: &str) -> Result<Option<MemoryObject>> {
        let file_path = self
            .data_dir
            .join("memories")
            .join(format!("{}.json", memory_id));
        if file_path.exists() {
            let content = fs::read_to_string(&file_path)?;
            let memory: MemoryObject = serde_json::from_str(&content)?;
            Ok(Some(memory))
        } else {
            Ok(None)
        }
    }

    pub fn delete_from_index(&self, memory_id: &str) -> Result<()> {
        self.db.execute(
            "DELETE FROM search_index WHERE memory_id = ?1",
            params![memory_id],
        )?;
        Ok(())
    }
}
