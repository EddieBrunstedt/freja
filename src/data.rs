//! Shared data structures and type aliases for the Freja bot.
//!
//! This module defines the global application state that is shared
//! across all command handlers and event handlers.

use crate::{ai, config::Config, memory::MemoryStore};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

/// Global application state shared across all handlers.
///
/// This structure contains all the shared resources needed by the bot,
/// wrapped in Arc for thread-safe sharing across async tasks.
pub struct Data {
    /// Persistent storage for user conversation memories.
    pub user_memories: Arc<MemoryStore>,
    /// OpenAI API client for generating responses and updating memories.
    pub ai_client: Arc<ai::Client>,
    /// The bot's own Discord user ID for self-identification.
    pub bot_id: serenity::UserId,
    /// Application configuration loaded from environment variables.
    pub config: Arc<Config>,
}

/// Type alias for error handling throughout the application.
pub type Error = anyhow::Error;

/// Type alias for Poise command context with our custom data and error types.
pub type Context<'a> = poise::Context<'a, Data, Error>;
