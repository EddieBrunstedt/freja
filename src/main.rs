//! Freja - A Discord bot with persistent memory.

use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use std::sync::Arc;
use tracing::{error, info};

mod ai;
mod commands;
mod config;
mod data;
mod handlers;
mod memory;
mod tracing;

use config::Config;
use data::Data;

#[tokio::main]
async fn main() {
    let config = Config::from_env().expect("Failed to parse configuration");

    tracing::initialize_tracing(&config);

    if let Err(e) = run_bot(config).await {
        error!("{}", e.to_string());
        std::process::exit(1);
    }
}

/// Initializes and runs the Discord bot with the provided configuration.
///
/// This function sets up the Discord client, registers commands, and starts
/// the bot's event loop.
///
/// # Errors
///
/// Returns an error if:
/// - Discord client creation fails
/// - Bot startup fails
async fn run_bot(config: Config) -> Result<()> {
    info!("Starting Discord bot");

    let intents =
        serenity::GatewayIntents::non_privileged() | serenity::GatewayIntents::MESSAGE_CONTENT;

    let discord_token = config.discord_token.clone();

    let framework = poise::Framework::builder()
        .options(poise::FrameworkOptions {
            commands: vec![commands::memory::memory()],
            event_handler: |ctx, event, framework, data| {
                Box::pin(handlers::message::handle_event(ctx, event, framework, data))
            },
            ..Default::default()
        })
        .setup(move |ctx, _ready, framework| Box::pin(initialize_bot_data(ctx, framework, config)))
        .build();

    let mut client = serenity::ClientBuilder::new(&discord_token, intents)
        .framework(framework)
        .await
        .context("Failed to create Discord client")?;

    client
        .start()
        .await
        .context("Failed to start Discord bot")?;

    Ok(())
}

/// Initializes bot data and registers commands.
///
/// This function is called once when the bot connects to Discord. It:
/// - Registers slash commands globally and optionally to a specific guild
/// - Initializes the memory store from disk
/// - Creates the AI client for OpenAI API access
///
/// # Errors
///
/// Returns an error if:
/// - Command registration fails
/// - Memory store initialization fails
async fn initialize_bot_data(
    ctx: &serenity::Context,
    framework: &poise::Framework<Data, data::Error>,
    config: Config,
) -> Result<Data> {
    poise::builtins::register_globally(ctx, &framework.options().commands)
        .await
        .context("Failed to register commands globally")?;

    info!("Registered commands globally");

    if let Some(guild_id) = config.guild_id {
        poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id)
            .await
            .context("Failed to register commands in guild")?;

        info!("Also registered commands in guild {guild_id}");
    }

    let user_memory_store = memory::MemoryStore::new(&config.memory_file)
        .await
        .context("Failed to initialize memory store")?;

    let openai_client = ai::Client::new(config.openai_api_key.clone());

    info!("Bot initialization complete");

    Ok(Data {
        user_memories: Arc::new(user_memory_store),
        ai_client: Arc::new(openai_client),
        bot_id: ctx.cache.current_user().id,
        _config: Arc::new(config),
    })
}
