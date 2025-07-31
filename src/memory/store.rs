//! Persistent storage for user conversation memories.
//!
//! This module provides thread-safe storage and retrieval of user-specific
//! conversation summaries that persist across bot restarts.

use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Represents a stored memory for a single user.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UserMemory {
    /// The AI-generated summary of past conversations with this user.
    pub memory: String,
    /// Timestamp of when this memory was last updated.
    pub last_updated: chrono::DateTime<chrono::Utc>,
}

/// Thread-safe storage for all user memories with automatic persistence.
///
/// Memories are saved to disk after each update to ensure data persistence
/// across bot restarts.
pub struct MemoryStore {
    /// In-memory storage of user memories, protected by RwLock for concurrent access.
    memories: RwLock<HashMap<serenity::UserId, UserMemory>>,
    /// Path to the JSON file where memories are persisted.
    file_path: String,
}

impl MemoryStore {
    /// Creates a new MemoryStore instance, loading existing memories from disk if available.
    ///
    /// If the memory file exists but cannot be parsed, a warning is logged and
    /// an empty store is created instead of failing.
    ///
    /// # Arguments
    ///
    /// * `file_path` - Path to the JSON file for memory persistence
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

    /// Retrieves a user's memory if it exists.
    ///
    /// Returns None if no memory has been stored for the given user.
    pub async fn get(&self, user_id: &serenity::UserId) -> Option<UserMemory> {
        self.memories.read().await.get(user_id).cloned()
    }

    /// Updates or creates a memory entry for a user.
    ///
    /// Automatically persists the updated memory to disk after modification.
    ///
    /// # Errors
    ///
    /// Returns an error if saving to disk fails.
    pub async fn update(&self, user_id: serenity::UserId, memory_content: String) -> Result<()> {
        {
            let mut memories = self.memories.write().await;
            memories.insert(
                user_id,
                UserMemory {
                    memory: memory_content,
                    last_updated: chrono::Utc::now(),
                },
            );
        }

        self.persist_to_disk().await?;
        Ok(())
    }

    /// Saves the current in-memory state to the configured file path.
    ///
    /// Uses pretty-printing for human readability of the JSON file.
    async fn persist_to_disk(&self) -> Result<()> {
        let memories = self.memories.read().await;
        let json = serde_json::to_string_pretty(&*memories)
            .context("Failed to serialize memories to JSON")?;
        tokio::fs::write(&self.file_path, json)
            .await
            .with_context(|| format!("Failed to write memory file to {}", self.file_path))?;
        Ok(())
    }
}
