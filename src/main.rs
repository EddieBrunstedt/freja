use poise::serenity_prelude as serenity;
use std::sync::Arc;

mod ai;
mod commands;
mod config;
mod data;
mod handlers;
mod memory;

use config::Config;
use data::{Data, Error};

#[tokio::main]
async fn main() -> Result<(), Error> {
    // Load configuration
    let config = Config::from_env()?;

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
        .await?;

    client.start().await?;
    Ok(())
}

async fn setup_bot(
    ctx: &serenity::Context,
    framework: &poise::Framework<Data, Error>,
    config: Config,
) -> Result<Data, Error> {
    // Register commands
    poise::builtins::register_globally(ctx, &framework.options().commands).await?;
    println!("✓ Registered commands globally");

    // Optionally register in specific guild for faster updates during development
    if let Some(guild_id) = config.guild_id {
        poise::builtins::register_in_guild(ctx, &framework.options().commands, guild_id).await?;
        println!("✓ Also registered commands in guild {guild_id}");
    }

    // Initialize data
    let memory_store = memory::MemoryStore::new(&config.memory_file).await?;
    let ai_client = ai::Client::new(config.openai_api_key.clone());

    Ok(Data {
        user_memories: Arc::new(memory_store),
        ai_client: Arc::new(ai_client),
        bot_id: ctx.cache.current_user().id,
        config: Arc::new(config),
    })
}
