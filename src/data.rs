use crate::{ai, config::Config, memory::MemoryStore};
use poise::serenity_prelude as serenity;
use std::sync::Arc;

pub struct Data {
    pub user_memories: Arc<MemoryStore>,
    pub ai_client: Arc<ai::Client>,
    pub bot_id: serenity::UserId,
    pub config: Arc<Config>,
}

pub type Error = Box<dyn std::error::Error + Send + Sync>;
pub type Context<'a> = poise::Context<'a, Data, Error>;
