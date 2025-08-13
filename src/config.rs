//! Configuration module for Freja Discord bot.
//!
//! This module handles loading configuration from environment variables and provides
//! the core settings structure used throughout the application.

use anyhow::{Context, Result, bail};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

/// Main configuration structure for the Freja bot.
///
/// All configuration values are loaded from environment variables on startup.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    /// Discord bot token for authentication.
    ///
    /// Required. Obtained from Discord Developer Portal.
    /// Environment variable: `DISCORD_TOKEN`
    pub discord_token: String,

    /// Optional Discord guild ID for registering slash commands.
    ///
    /// When provided, commands will be registered to this specific guild
    /// in addition to global registration. This enables faster command updates
    /// during development.
    /// Environment variable: `GUILD_ID`
    pub guild_id: Option<serenity::GuildId>,

    /// File path for persisting user memories.
    ///
    /// The bot saves conversation summaries to this JSON file to maintain
    /// context across restarts. Path can be relative or absolute.
    /// Default: "memories.json"
    /// Environment variable: `MEMORY_FILE`
    pub memory_file: String,

    /// OpenAI API key for GPT model access.
    ///
    /// Required. Used for generating responses and updating memory summaries.
    /// Environment variable: `OPENAI_API_KEY`
    pub openai_api_key: String,

    /// Enable pretty-printed logs for development.
    ///
    /// When true, logs are formatted for human readability.
    /// When false, logs use structured format suitable for production.
    /// Default: false
    /// Environment variable: `PRETTY_LOG`
    pub pretty_log: bool,
}

impl Config {
    /// Creates a new Config instance by reading from environment variables.
    ///
    /// # Errors
    ///
    /// Returns an error if required environment variables are missing or invalid:
    /// - `DISCORD_TOKEN` must be set and non-empty
    /// - `OPENAI_API_KEY` must be set and non-empty
    pub fn from_env() -> Result<Self> {
        let discord_token = get_required_env_var("DISCORD_TOKEN")?;

        let guild_id = std::env::var("GUILD_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(serenity::GuildId::new);

        let memory_file =
            std::env::var("MEMORY_FILE").unwrap_or_else(|_| "memories.json".to_string());

        let openai_api_key = get_required_env_var("OPENAI_API_KEY")?;

        let pretty_log = std::env::var("PRETTY_LOG")
            .unwrap_or_else(|_| "false".to_string())
            .parse()
            .unwrap_or(false);

        Ok(Config {
            discord_token,
            guild_id,
            memory_file,
            openai_api_key,
            pretty_log,
        })
    }
}

/// Retrieves a required environment variable by name.
///
/// # Errors
///
/// Returns an error if:
/// - The environment variable is not set
/// - The environment variable is set but empty
fn get_required_env_var(name: &str) -> Result<String> {
    let value = std::env::var(name)
        .with_context(|| format!("Missing required {name} environment variable"))?;

    if value.is_empty() {
        bail!("{} environment variable is set but empty", name);
    }

    Ok(value)
}
