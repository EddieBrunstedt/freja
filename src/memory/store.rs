use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMemory {
    pub memory: String,
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

pub struct MemoryStore {
    memories: RwLock<HashMap<serenity::UserId, UserMemory>>,
    file_path: String,
}

impl MemoryStore {
    pub async fn new(file_path: &str) -> Result<Self> {
        let memories = if std::path::Path::new(file_path).exists() {
            let contents = tokio::fs::read_to_string(file_path).await?;
            match serde_json::from_str(&contents) {
                Ok(memories) => memories,
                Err(e) => {
                    tracing::warn!(
                        "Failed to parse memory file {}: {}. Starting with empty memory store.",
                        file_path,
                        e
                    );
                    HashMap::new()
                }
            }
        } else {
            HashMap::new()
        };

        Ok(Self {
            memories: RwLock::new(memories),
            file_path: file_path.to_string(),
        })
    }

    pub async fn get(&self, user_id: &serenity::UserId) -> Option<UserMemory> {
        self.memories.read().await.get(user_id).cloned()
    }

    pub async fn update(&self, user_id: serenity::UserId, memory: String) -> Result<()> {
        {
            let mut memories = self.memories.write().await;
            memories.insert(
                user_id,
                UserMemory {
                    memory,
                    last_updated: chrono::Utc::now(),
                },
            );
        }

        self.save_to_disk().await?;
        Ok(())
    }

    async fn save_to_disk(&self) -> Result<()> {
        let memories = self.memories.read().await;
        let json = serde_json::to_string_pretty(&*memories)
            .context("Failed to serialize memories to JSON")?;
        tokio::fs::write(&self.file_path, json)
            .await
            .with_context(|| format!("Failed to write memory file to {}", self.file_path))?;
        Ok(())
    }
}
