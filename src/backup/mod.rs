use crate::models::memory::MemoryObject;
use crate::retrieval::RetrievalEngine;
use crate::storage::{GraphStore, MemoryStore};
use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::{self, File};
use std::io::{Read, Write};
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BackupManifest {
    pub version: String,
    pub created_at: String,
    pub memory_count: u64,
    pub node_count: u64,
    pub edge_count: u64,
}

pub struct BackupManager {
    data_dir: PathBuf,
}

impl BackupManager {
    pub fn new(data_dir: PathBuf) -> Self {
        Self { data_dir }
    }

    pub fn create_backup(&self, backup_path: &PathBuf) -> Result<BackupManifest> {
        fs::create_dir_all(backup_path)?;

        let memory_manifest = self.backup_memories(backup_path)?;
        let graph_manifest = self.backup_graph(backup_path)?;
        let retrieval_manifest = self.backup_retrieval(backup_path)?;

        let manifest = BackupManifest {
            version: "1.0".to_string(),
            created_at: chrono::Utc::now().to_rfc3339(),
            memory_count: memory_manifest,
            node_count: graph_manifest.0,
            edge_count: graph_manifest.1,
        };

        let manifest_json = serde_json::to_string_pretty(&manifest)?;
        let mut manifest_file = File::create(backup_path.join("manifest.json"))?;
        manifest_file.write_all(manifest_json.as_bytes())?;

        tracing::info!("Backup created at {:?}", backup_path);
        Ok(manifest)
    }

    fn backup_memories(&self, backup_path: &PathBuf) -> Result<u64> {
        let memories_dir = self.data_dir.join("memories");
        let backup_memories_dir = backup_path.join("memories");
        fs::create_dir_all(&backup_memories_dir)?;

        let mut count = 0u64;
        if memories_dir.exists() {
            for entry in fs::read_dir(&memories_dir)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "json") {
                    fs::copy(entry.path(), backup_memories_dir.join(entry.file_name()))?;
                    count += 1;
                }
            }
        }
        Ok(count)
    }

    fn backup_graph(&self, backup_path: &PathBuf) -> Result<(u64, u64)> {
        let graph_db = self.data_dir.join("graph.db");
        let backup_db = backup_path.join("graph.db");

        if graph_db.exists() {
            fs::copy(&graph_db, &backup_db)?;
        }

        let node_count = self.count_lines(&backup_db, "SELECT COUNT(*) FROM nodes")?;
        let edge_count = self.count_lines(&backup_db, "SELECT COUNT(*) FROM edges")?;

        Ok((node_count, edge_count))
    }

    fn backup_retrieval(&self, backup_path: &PathBuf) -> Result<u64> {
        let retrieval_db = self.data_dir.join("retrieval.db");
        let backup_db = backup_path.join("retrieval.db");

        if retrieval_db.exists() {
            fs::copy(&retrieval_db, &backup_db)?;
        }

        let count = self.count_lines(&backup_db, "SELECT COUNT(*) FROM search_index")?;
        Ok(count)
    }

    fn count_lines(&self, db_path: &PathBuf, _query: &str) -> Result<u64> {
        let mut file = File::open(db_path)?;
        let mut buffer = Vec::new();
        file.read_to_end(&mut buffer)?;

        Ok(if buffer.len() > 0 { 1 } else { 0 })
    }

    pub fn restore_backup(&self, backup_path: &PathBuf) -> Result<()> {
        let manifest_path = backup_path.join("manifest.json");
        if !manifest_path.exists() {
            return Err(anyhow::anyhow!("No manifest found in backup"));
        }

        let memories_backup = backup_path.join("memories");
        if memories_backup.exists() {
            let memories_dir = self.data_dir.join("memories");
            fs::create_dir_all(&memories_dir)?;

            for entry in fs::read_dir(&memories_backup)? {
                let entry = entry?;
                fs::copy(entry.path(), memories_dir.join(entry.file_name()))?;
            }
        }

        let graph_backup = backup_path.join("graph.db");
        if graph_backup.exists() {
            let graph_db = self.data_dir.join("graph.db");
            fs::copy(&graph_backup, &graph_db)?;
        }

        let retrieval_backup = backup_path.join("retrieval.db");
        if retrieval_backup.exists() {
            let retrieval_db = self.data_dir.join("retrieval.db");
            fs::copy(&retrieval_backup, &retrieval_db)?;
        }

        tracing::info!("Backup restored from {:?}", backup_path);
        Ok(())
    }

    pub fn export_data(&self, export_path: &PathBuf) -> Result<File> {
        let mut file = File::create(export_path)?;

        let memories_dir = self.data_dir.join("memories");
        if memories_dir.exists() {
            for entry in fs::read_dir(&memories_dir)? {
                let entry = entry?;
                if entry.path().extension().map_or(false, |e| e == "json") {
                    let content = fs::read_to_string(entry.path())?;
                    writeln!(file, "{}", content)?;
                }
            }
        }

        Ok(file)
    }

    pub fn import_data(&self, import_path: &PathBuf) -> Result<u64> {
        let mut count = 0u64;
        let content = fs::read_to_string(import_path)?;

        for line in content.lines() {
            if line.trim().is_empty() {
                continue;
            }
            if let Ok(memory) = serde_json::from_str::<MemoryObject>(line) {
                let file_path = self
                    .data_dir
                    .join("memories")
                    .join(format!("{}.json", memory.memory_id));
                let mut file = fs::File::create(&file_path)?;
                file.write_all(line.as_bytes())?;
                count += 1;
            }
        }

        Ok(count)
    }
}
