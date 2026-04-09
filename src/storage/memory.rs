use crate::models::MemoryObject;
use anyhow::{anyhow, Result};
use rusqlite::{params, Connection};
use std::fs;
use std::io::Write;
use std::path::PathBuf;

pub struct MemoryStore {
    db: Connection,
    data_dir: PathBuf,
}

impl MemoryStore {
    pub fn new(data_dir: PathBuf) -> Result<Self> {
        fs::create_dir_all(&data_dir)?;
        fs::create_dir_all(data_dir.join("memories"))?;

        let db_path = data_dir.join("divinelight.db");
        let db = Connection::open(&db_path)?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS memories (
                memory_id TEXT PRIMARY KEY,
                created_at TEXT NOT NULL,
                source TEXT NOT NULL,
                format TEXT NOT NULL,
                content TEXT NOT NULL,
                tags TEXT NOT NULL,
                checksum TEXT NOT NULL,
                version INTEGER NOT NULL,
                notes TEXT NOT NULL
            )",
            [],
        )?;

        db.execute(
            "CREATE TABLE IF NOT EXISTS tags (
                memory_id TEXT NOT NULL,
                tag TEXT NOT NULL,
                PRIMARY KEY (memory_id, tag),
                FOREIGN KEY (memory_id) REFERENCES memories(memory_id)
            )",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_created_at ON memories(created_at)",
            [],
        )?;

        db.execute(
            "CREATE INDEX IF NOT EXISTS idx_source ON memories(source)",
            [],
        )?;

        Ok(Self { db, data_dir })
    }

    pub fn ingest(
        &mut self,
        source: String,
        format: String,
        content: String,
        tags: Vec<String>,
    ) -> Result<MemoryObject> {
        let memory = MemoryObject::new(source, format, content, tags.clone());
        let memory_id = memory.memory_id.clone();

        let file_path = self
            .data_dir
            .join("memories")
            .join(format!("{}.json", memory_id));
        let mut file = fs::File::create(&file_path)?;
        file.write_all(serde_json::to_string_pretty(&memory)?.as_bytes())?;

        self.db.execute(
            "INSERT INTO memories (memory_id, created_at, source, format, content, tags, checksum, version, notes)
             VALUES (?1, ?2, ?3, ?4, ?5, ?6, ?7, ?8, ?9)",
            params![
                memory.memory_id,
                memory.created_at.to_rfc3339(),
                memory.source,
                memory.format,
                memory.content,
                serde_json::to_string(&memory.tags)?,
                memory.checksum,
                memory.version,
                memory.notes,
            ],
        )?;

        for tag in &tags {
            self.db.execute(
                "INSERT OR IGNORE INTO tags (memory_id, tag) VALUES (?1, ?2)",
                params![memory_id, tag],
            )?;
        }

        Ok(memory)
    }

    pub fn get(&self, memory_id: &str) -> Result<MemoryObject> {
        let file_path = self
            .data_dir
            .join("memories")
            .join(format!("{}.json", memory_id));
        let content = fs::read_to_string(&file_path)?;
        let memory: MemoryObject = serde_json::from_str(&content)?;

        if !memory.verify() {
            return Err(anyhow!(
                "Checksum verification failed for memory {}",
                memory_id
            ));
        }

        Ok(memory)
    }

    #[allow(dead_code)]
    pub fn query_by_tag(&self, tag: &str, limit: usize) -> Result<Vec<MemoryObject>> {
        let mut stmt = self.db.prepare(
            "SELECT m.memory_id FROM memories m
             JOIN tags t ON m.memory_id = t.memory_id
             WHERE t.tag = ?1
             ORDER BY m.created_at DESC
             LIMIT ?2",
        )?;

        let memory_ids: Vec<String> = stmt
            .query_map(params![tag, limit as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut results = Vec::new();
        for id in memory_ids {
            if let Ok(memory) = self.get(&id) {
                results.push(memory);
            }
        }
        Ok(results)
    }

    #[allow(dead_code)]
    pub fn query_by_source(&self, source: &str, limit: usize) -> Result<Vec<MemoryObject>> {
        let mut stmt = self.db.prepare(
            "SELECT memory_id FROM memories WHERE source = ?1 ORDER BY created_at DESC LIMIT ?2",
        )?;

        let memory_ids: Vec<String> = stmt
            .query_map(params![source, limit as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut results = Vec::new();
        for id in memory_ids {
            if let Ok(memory) = self.get(&id) {
                results.push(memory);
            }
        }
        Ok(results)
    }

    pub fn list_all(&self, limit: usize, offset: usize) -> Result<Vec<MemoryObject>> {
        let mut stmt = self.db.prepare(
            "SELECT memory_id FROM memories ORDER BY created_at DESC LIMIT ?1 OFFSET ?2",
        )?;

        let memory_ids: Vec<String> = stmt
            .query_map(params![limit as i64, offset as i64], |row| row.get(0))?
            .filter_map(|r| r.ok())
            .collect();

        let mut results = Vec::new();
        for id in memory_ids {
            if let Ok(memory) = self.get(&id) {
                results.push(memory);
            }
        }
        Ok(results)
    }

    pub fn count(&self) -> Result<u64> {
        let count: i64 = self
            .db
            .query_row("SELECT COUNT(*) FROM memories", [], |row| row.get(0))?;
        Ok(count as u64)
    }
}
