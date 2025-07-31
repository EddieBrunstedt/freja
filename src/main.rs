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

    tracing::init_subscriber(&config);

    if let Err(e) = run(config).await {
        error!("{}", e.to_string());
        std::process::exit(1);
    }
}

async fn run(config: Config) -> Result<()> {
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
        .setup(move |ctx, _ready, framework| Box::pin(setup_bot(ctx, framework, config)))
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

async fn setup_bot(
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

    let memory_store = memory::MemoryStore::new(&config.memory_file)
        .await
        .context("Failed to initialize memory store")?;

    let ai_client = ai::Client::new(config.openai_api_key.clone());

    info!("Bot initialization complete");

    Ok(Data {
        user_memories: Arc::new(memory_store),
        ai_client: Arc::new(ai_client),
        bot_id: ctx.cache.current_user().id,
        config: Arc::new(config),
    })
}
