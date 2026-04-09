use crate::models::graph::{GraphEdge, GraphMetadata, GraphNode, MemoryGraphLink};
use anyhow::Result;
use chrono::Utc;
use rusqlite::{params, Connection};
use std::fs;
use std::path::PathBuf;
use uuid::Uuid;

pub struct GraphStore {
    db: Connection,
    #[allow(dead_code)]
    data_dir: PathBuf,
}

impl GraphStore {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;

        let db_path = data_dir.join("graph.db");
        let db = Connection::open(&db_path)?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS nodes (
                id TEXT PRIMARY KEY,
                node_type TEXT NOT NULL,
                label TEXT NOT NULL,
                properties TEXT NOT NULL,
                provenance TEXT NOT NULL,
                version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS edges (
                id TEXT PRIMARY KEY,
                source TEXT NOT NULL,
                target TEXT NOT NULL,
                relation TEXT NOT NULL,
                properties TEXT NOT NULL,
                provenance TEXT NOT NULL,
                confidence REAL NOT NULL,
                version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS memory_graph_links (
                memory_id TEXT PRIMARY KEY,
                linked_nodes TEXT NOT NULL,
                linked_edges TEXT NOT NULL,
                link_type TEXT NOT NULL,
                confidence REAL NOT NULL,
                justification TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS metadata (
                graph_id TEXT PRIMARY KEY,
                schema_version TEXT NOT NULL,
                created_at TEXT NOT NULL,
                updated_at TEXT NOT NULL,
                node_count INTEGER NOT NULL,
                edge_count INTEGER NOT NULL,
                retention_policy TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_node_type ON nodes(node_type)",
            [],
        )?;
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_edge_source ON edges(source)",
            [],
        )?;
        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_edge_target ON edges(target)",
            [],
        )?;

        let now = Utc::now().to_rfc3339();
        db.execute(
            "INSERT OR IGNORE INTO metadata (graph_id, schema_version, created_at, updated_at, node_count, edge_count, retention_policy)
             VALUES ('main', '1.0', ?1, ?1, 0, 0, 'retain_forever')",
            params![now],
        )?;

        Ok(Self { db, data_dir })
    }

    pub fn create_node(
        &self,
        node_type: String,
        label: String,
        properties: serde_json::Value,
        provenance: Vec<String>,
    ) -> Result<GraphNode> {
        let id = format!("node-{}", Uuid::new_v4());
        let now = Utc::now();
        let version = "v1.0".to_string();

        self.db.execute(
            "INSERT INTO nodes (id, node_type, label, properties, provenance, version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8)",
            params![
                id,
                node_type,
                label,
                serde_json::to_string(&properties)?,
                serde_json::to_string(&provenance)?,
                version,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.update_metadata_count()?;

        Ok(GraphNode {
            id,
            node_type,
            label,
            properties,
            provenance,
            version,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_node(&self, id: &str) -> Result<GraphNode> {
        self.db.query_row(
            "SELECT id, node_type, label, properties, provenance, version, created_at, updated_at FROM nodes WHERE id = ?1",
            params![id],
            |row| {
                Ok(GraphNode {
                    id: row.get(0)?,
                    node_type: row.get(1)?,
                    label: row.get(2)?,
                    properties: serde_json::from_str(&row.get::<_, String>(3)?).unwrap_or(serde_json::json!({})),
                    provenance: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
                    version: row.get(5)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?).unwrap().with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?).unwrap().with_timezone(&chrono::Utc),
                })
            },
        ).map_err(|e| anyhow::anyhow!("Node not found: {}", e))
    }

    pub fn query_nodes(
        &self,
        node_type: Option<&str>,
        _label_contains: Option<&str>,
        limit: usize,
    ) -> Result<Vec<GraphNode>> {
        let mut sql = "SELECT id, node_type, label, properties, provenance, version, created_at, updated_at FROM nodes WHERE 1=1".to_string();
        if node_type.is_some() {
            sql.push_str(" AND node_type = ?1");
        }
        sql.push_str(" ORDER BY created_at DESC LIMIT ?2");

        let mut stmt = self.db.prepare(&sql)?;
        let nodes: Vec<GraphNode> = if let Some(nt) = node_type {
            stmt.query_map(params![nt, limit as i64], Self::map_node)?
                .filter_map(|r| r.ok())
                .collect()
        } else {
            stmt.query_map(params![limit as i64], Self::map_node)?
                .filter_map(|r| r.ok())
                .collect()
        };

        Ok(nodes)
    }

    fn map_node(row: &rusqlite::Row) -> rusqlite::Result<GraphNode> {
        Ok(GraphNode {
            id: row.get(0)?,
            node_type: row.get(1)?,
            label: row.get(2)?,
            properties: serde_json::from_str(&row.get::<_, String>(3)?)
                .unwrap_or(serde_json::json!({})),
            provenance: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or_default(),
            version: row.get(5)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(6)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(7)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }

    pub fn create_edge(
        &self,
        source: String,
        target: String,
        relation: String,
        properties: serde_json::Value,
        provenance: Vec<String>,
        confidence: f64,
    ) -> Result<GraphEdge> {
        let id = format!("edge-{}", Uuid::new_v4());
        let now = Utc::now();
        let version = "v1.0".to_string();

        self.db.execute(
            "INSERT INTO edges (id, source, target, relation, properties, provenance, confidence, version, created_at, updated_at)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9, ?10)",
            params![
                id,
                source,
                target,
                relation,
                serde_json::to_string(&properties)?,
                serde_json::to_string(&provenance)?,
                confidence,
                version,
                now.to_rfc3339(),
                now.to_rfc3339(),
            ],
        )?;

        self.update_metadata_count()?;

        Ok(GraphEdge {
            id,
            source,
            target,
            relation,
            properties,
            provenance,
            confidence,
            version,
            created_at: now,
            updated_at: now,
        })
    }

    pub fn get_edge(&self, id: &str) -> Result<GraphEdge> {
        self.db.query_row(
            "SELECT id, source, target, relation, properties, provenance, confidence, version, created_at, updated_at FROM edges WHERE id = ?1",
            params![id],
            |row| {
                Ok(GraphEdge {
                    id: row.get(0)?,
                    source: row.get(1)?,
                    target: row.get(2)?,
                    relation: row.get(3)?,
                    properties: serde_json::from_str(&row.get::<_, String>(4)?).unwrap_or(serde_json::json!({})),
                    provenance: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
                    confidence: row.get(6)?,
                    version: row.get(7)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?).unwrap().with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?).unwrap().with_timezone(&chrono::Utc),
                })
            },
        ).map_err(|e| anyhow::anyhow!("Edge not found: {}", e))
    }

    #[allow(dead_code)]
    pub fn query_edges(&self, limit: usize) -> Result<Vec<GraphEdge>> {
        let sql = "SELECT id, source, target, relation, properties, provenance, confidence, version, created_at, updated_at FROM edges ORDER BY created_at DESC LIMIT ?1";
        let mut stmt = self.db.prepare(sql)?;
        let edges: Vec<GraphEdge> = stmt
            .query_map(params![limit as i64], Self::map_edge)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(edges)
    }

    fn map_edge(row: &rusqlite::Row) -> rusqlite::Result<GraphEdge> {
        Ok(GraphEdge {
            id: row.get(0)?,
            source: row.get(1)?,
            target: row.get(2)?,
            relation: row.get(3)?,
            properties: serde_json::from_str(&row.get::<_, String>(4)?)
                .unwrap_or(serde_json::json!({})),
            provenance: serde_json::from_str(&row.get::<_, String>(5)?).unwrap_or_default(),
            confidence: row.get(6)?,
            version: row.get(7)?,
            created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(8)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
            updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(9)?)
                .unwrap()
                .with_timezone(&chrono::Utc),
        })
    }

    pub fn get_node_neighbors(&self, node_id: &str, depth: usize) -> Result<Vec<GraphEdge>> {
        let sql = "SELECT id, source, target, relation, properties, provenance, confidence, version, created_at, updated_at FROM edges WHERE source = ?1 OR target = ?1 ORDER BY created_at DESC LIMIT ?2";
        let mut stmt = self.db.prepare(sql)?;
        let edges: Vec<GraphEdge> = stmt
            .query_map(params![node_id, depth as i64 * 100], Self::map_edge)?
            .filter_map(|r| r.ok())
            .collect();
        Ok(edges)
    }

    pub fn traverse_bfs(&self, start_node_id: &str, max_depth: usize) -> Result<Vec<GraphNode>> {
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<(String, usize)> = vec![(start_node_id.to_string(), 0)];
        let mut result = Vec::new();

        while let Some((node_id, depth)) = queue.pop() {
            if visited.contains(&node_id) || depth > max_depth {
                continue;
            }
            visited.insert(node_id.clone());

            if let Ok(node) = self.get_node(&node_id) {
                result.push(node);
            }

            if depth < max_depth {
                let neighbors = self.get_node_neighbors(&node_id, 100)?;
                for edge in neighbors {
                    let next_node = if edge.source == node_id {
                        edge.target.clone()
                    } else {
                        edge.source.clone()
                    };
                    queue.push((next_node, depth + 1));
                }
            }
        }

        Ok(result)
    }

    pub fn find_path(
        &self,
        start_id: &str,
        end_id: &str,
        max_depth: usize,
    ) -> Result<Option<Vec<String>>> {
        let mut visited = std::collections::HashSet::new();
        let mut queue: Vec<(String, Vec<String>)> =
            vec![(start_id.to_string(), vec![start_id.to_string()])];

        while let Some((node_id, path)) = queue.pop() {
            if visited.contains(&node_id) {
                continue;
            }
            visited.insert(node_id.clone());

            if node_id == end_id {
                return Ok(Some(path));
            }

            if path.len() >= max_depth {
                continue;
            }

            let neighbors = self.get_node_neighbors(&node_id, 100)?;
            for edge in neighbors {
                if edge.source != node_id {
                    continue;
                }
                let next_node = edge.target.clone();
                if !visited.contains(&next_node) {
                    let mut new_path = path.clone();
                    new_path.push(next_node.clone());
                    queue.push((next_node, new_path));
                }
            }
        }

        Ok(None)
    }

    #[allow(dead_code)]
    pub fn has_cycle(&self, node_id: &str) -> Result<bool> {
        let mut visited = std::collections::HashSet::new();
        let mut recursion_stack = std::collections::HashSet::new();

        fn dfs(
            graph: &GraphStore,
            node: &str,
            visited: &mut std::collections::HashSet<String>,
            stack: &mut std::collections::HashSet<String>,
        ) -> bool {
            visited.insert(node.to_string());
            stack.insert(node.to_string());

            if let Ok(neighbors) = graph.get_node_neighbors(node, 1000) {
                for edge in neighbors {
                    let next = if edge.source == node {
                        edge.target.clone()
                    } else {
                        edge.source.clone()
                    };
                    if !visited.contains(&next) {
                        if dfs(graph, &next, visited, stack) {
                            return true;
                        }
                    } else if stack.contains(&next) {
                        return true;
                    }
                }
            }

            stack.remove(node);
            false
        }

        Ok(dfs(self, node_id, &mut visited, &mut recursion_stack))
    }

    #[allow(dead_code)]
    pub fn link_memory_to_graph(
        &self,
        memory_id: String,
        linked_nodes: Vec<String>,
        linked_edges: Vec<String>,
        link_type: String,
        confidence: f64,
        justification: String,
    ) -> Result<MemoryGraphLink> {
        self.db.execute(
            "INSERT OR REPLACE INTO memory_graph_links (memory_id, linked_nodes, linked_edges, link_type, confidence, justification)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6)",
            params![
                memory_id,
                serde_json::to_string(&linked_nodes)?,
                serde_json::to_string(&linked_edges)?,
                link_type,
                confidence,
                justification,
            ],
        )?;

        Ok(MemoryGraphLink {
            memory_id,
            linked_nodes,
            linked_edges,
            link_type,
            confidence,
            justification,
        })
    }

    #[allow(dead_code)]
    pub fn get_memory_link(&self, memory_id: &str) -> Result<MemoryGraphLink> {
        self.db.query_row(
            "SELECT memory_id, linked_nodes, linked_edges, link_type, confidence, justification FROM memory_graph_links WHERE memory_id = ?1",
            params![memory_id],
            |row| {
                Ok(MemoryGraphLink {
                    memory_id: row.get(0)?,
                    linked_nodes: serde_json::from_str(&row.get::<_, String>(1)?).unwrap_or_default(),
                    linked_edges: serde_json::from_str(&row.get::<_, String>(2)?).unwrap_or_default(),
                    link_type: row.get(3)?,
                    confidence: row.get(4)?,
                    justification: row.get(5)?,
                })
            },
        ).map_err(|e| anyhow::anyhow!("Link not found: {}", e))
    }

    pub fn get_metadata(&self) -> Result<GraphMetadata> {
        self.db.query_row(
            "SELECT graph_id, schema_version, created_at, updated_at, node_count, edge_count, retention_policy FROM metadata WHERE graph_id = 'main'",
            [],
            |row: &rusqlite::Row| {
                Ok(GraphMetadata {
                    graph_id: row.get(0)?,
                    schema_version: row.get(1)?,
                    created_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(2)?).unwrap().with_timezone(&chrono::Utc),
                    updated_at: chrono::DateTime::parse_from_rfc3339(&row.get::<_, String>(3)?).unwrap().with_timezone(&chrono::Utc),
                    node_count: row.get(4)?,
                    edge_count: row.get(5)?,
                    retention_policy: row.get(6)?,
                })
            },
        ).map_err(|e| anyhow::anyhow!("Metadata not found: {}", e))
    }

    fn update_metadata_count(&self) -> Result<()> {
        let node_count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM nodes", [], |row| row.get(0))?;
        let edge_count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM edges", [], |row| row.get(0))?;
        let now = Utc::now().to_rfc3339();

        self.db.execute(
            "UPDATE metadata SET node_count = ?1, edge_count = ?2, updated_at = ?3 WHERE graph_id = 'main'",
            params![node_count, edge_count, now],
        )?;
        Ok(())
    }
}
