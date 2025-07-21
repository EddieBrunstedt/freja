use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub openai_api_key: String,
    pub guild_id: Option<serenity::GuildId>,
    pub memory_file: String,
    pub max_response_tokens: u32,
    pub max_memory_tokens: u32,
}

impl Config {
    pub fn from_env() -> Result<Self, Box<dyn std::error::Error + Send + Sync>> {
        let discord_token = std::env::var("DISCORD_TOKEN")?;
        let openai_api_key = std::env::var("OPENAI_API_KEY")?;

        let guild_id = std::env::var("GUILD_ID")
            .ok()
            .and_then(|s| s.parse::<u64>().ok())
            .map(serenity::GuildId::new);

        let memory_file =
            std::env::var("MEMORY_FILE").unwrap_or_else(|_| "memories.json".to_string());

        let max_response_tokens = std::env::var("MAX_RESPONSE_TOKENS")
            .unwrap_or_else(|_| "400".to_string())
            .parse()
            .unwrap_or(400);

        let max_memory_tokens = std::env::var("MAX_MEMORY_TOKENS")
            .unwrap_or_else(|_| "400".to_string())
            .parse()
            .unwrap_or(400);

        Ok(Config {
            discord_token,
            openai_api_key,
            guild_id,
            memory_file,
            max_response_tokens,
            max_memory_tokens,
        })
    }
}
