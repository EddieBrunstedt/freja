use anyhow::{Context, Result, bail};
use poise::serenity_prelude as serenity;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Config {
    pub discord_token: String,
    pub guild_id: Option<serenity::GuildId>,
    pub max_memory_tokens: u32,
    pub max_response_tokens: u32,
    pub memory_file: String,
    pub openai_api_key: String,
}

impl Config {
    pub fn from_env() -> Result<Self> {
        let discord_token = get_required_env_var("DISCORD_TOKEN")?;
        let openai_api_key = get_required_env_var("OPENAI_API_KEY")?;

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
            guild_id,
            max_memory_tokens,
            max_response_tokens,
            memory_file,
            openai_api_key,
        })
    }
}

fn get_required_env_var(name: &str) -> Result<String> {
    let value = std::env::var(name)
        .with_context(|| format!("Missing required {name} environment variable"))?;

    if value.is_empty() {
        bail!("{} environment variable is set but empty", name);
    }

    Ok(value)
}
