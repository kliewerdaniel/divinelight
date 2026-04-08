use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use sha2::{Digest, Sha256};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryObject {
    pub memory_id: String,
    pub created_at: DateTime<Utc>,
    pub source: String,
    pub format: String,
    pub content: String,
    pub tags: Vec<String>,
    pub checksum: String,
    pub version: u32,
    pub notes: String,
}

impl MemoryObject {
    pub fn new(source: String, format: String, content: String, tags: Vec<String>) -> Self {
        let memory_id = generate_memory_id();
        let created_at = Utc::now();
        let checksum = calculate_checksum(&content);

        Self {
            memory_id,
            created_at,
            source,
            format,
            content,
            tags,
            checksum,
            version: 1,
            notes: String::new(),
        }
    }

    pub fn verify(&self) -> bool {
        self.checksum == calculate_checksum(&self.content)
    }
}

fn generate_memory_id() -> String {
    let now = Utc::now();
    let uuid_short = uuid::Uuid::new_v4().to_string()[..8].to_string();
    format!("mp_{}_{}", now.format("%Y%m%d_%H%M%S"), uuid_short)
}

fn calculate_checksum(content: &str) -> String {
    let mut hasher = Sha256::new();
    hasher.update(content.as_bytes());
    let result = hasher.finalize();
    format!("sha256:{}", hex::encode(result))
}
