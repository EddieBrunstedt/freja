use crate::{
    ai,
    data::{Data, Error},
};
use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<()> {
    match event {
        serenity::FullEvent::Ready { data_about_bot } => {
            info!(
                "Bot connected and ready as \"{}\"",
                data_about_bot.user.name
            );
        }
        serenity::FullEvent::Message { new_message } => {
            if new_message.author.bot {
                return Ok(());
            }
            if should_respond(new_message, data.bot_id) {
                if let Err(e) = handle_message(ctx, new_message, data).await {
                    error!(
                        "Failed to handle message from {}: {e:?}",
                        new_message.author.display_name()
                    );
                }
            }
        }
        _ => (),
    }
    Ok(())
}

fn should_respond(message: &serenity::Message, bot_id: serenity::UserId) -> bool {
    if message.mentions.iter().any(|user| user.id == bot_id) {
        info!("Bot mentioned in message, will attempt to respond");
        return true;
    }

    let content_lower = message.content.to_lowercase();
    if content_lower.contains("freja") {
        info!("Bot name detected in message content, responding");
        return true;
    }

    false
}

async fn handle_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &Data,
) -> Result<()> {
    let _typing = message.channel_id.start_typing(&ctx.http);

    let user_memory = data
        .user_memories
        .get(&message.author.id)
        .await
        .map(|m| m.memory)
        .unwrap_or_else(|| "No previous interactions with this user.".to_string());

    let reply_context = get_reply_context(ctx, message, data.bot_id).await;

    let messages = ai::generate_response_prompt(
        &user_memory,
        message.author.display_name(),
        &message.content,
        &reply_context,
    );

    match data
        .ai_client
        .chat_completion(messages, data.config.max_response_tokens)
        .await
    {
        Ok(response) => {
            send_message_with_retry(ctx, message.channel_id, &response, 2)
                .await
                .context("Failed to send response message to Discord after retries")?;

            let user_memories = data.user_memories.clone();
            let ai_client = data.ai_client.clone();
            let user_id = message.author.id;
            let message_content = message.content.clone();
            let max_memory_tokens = data.config.max_memory_tokens;

            tokio::spawn(async move {
                if let Err(e) = update_user_memory(
                    &user_memories,
                    &ai_client,
                    user_id,
                    &user_memory,
                    &message_content,
                    &response,
                    max_memory_tokens,
                )
                .await
                {
                    error!("Failed to update user memory: {e:?}");
                }
            });
        }
        Err(e) => {
            error!("AI response error: {e:?}");
            if let Err(send_err) = send_message_with_retry(
                ctx,
                message.channel_id,
                "*sighs* I'm having trouble thinking right now... 😔",
                2,
            )
            .await
            {
                error!("Failed to send error message to Discord after retries: {send_err:?}");
            }
        }
    }

    Ok(())
}

async fn get_reply_context(
    ctx: &serenity::Context,
    message: &serenity::Message,
    bot_id: serenity::UserId,
) -> String {
    if let Some(referenced) = &message.referenced_message {
        return format_reply_context(referenced, bot_id);
    }

    if let Some(reference) = &message.message_reference {
        if let Some(message_id) = reference.message_id {
            match message.channel_id.message(&ctx.http, message_id).await {
                Ok(referenced) => return format_reply_context(&referenced, bot_id),
                Err(e) => {
                    info!("Could not fetch referenced message {message_id}: {e}");
                }
            }
        }
    }

    String::new()
}

fn format_reply_context(message: &serenity::Message, bot_id: serenity::UserId) -> String {
    let author_name = if message.author.id == bot_id {
        "You (Freja)".to_string()
    } else {
        message.author.display_name().to_string()
    };

    format!(
        "\n\nFor context, this user's message was a reply to {} who said: {}",
        author_name, message.content
    )
}

async fn update_user_memory(
    user_memories: &crate::memory::MemoryStore,
    ai_client: &ai::Client,
    user_id: serenity::UserId,
    current_memory: &str,
    new_message: &str,
    freja_response: &str,
    max_tokens: u32,
) -> Result<()> {
    let prompt = ai::generate_memory_prompt(current_memory, new_message, freja_response);

    let messages = vec![json!({ "role": "user", "content": prompt })];

    let updated_memory = ai_client
        .chat_completion(messages, max_tokens)
        .await
        .context("Failed to generate updated memory via AI")?;
    user_memories
        .update(user_id, updated_memory)
        .await
        .context("Failed to save updated memory to store")?;

    Ok(())
}

async fn send_message_with_retry(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    content: &str,
    retries: u8,
) -> Result<serenity::Message> {
    let mut last_err = None;

    for attempt in 0..=retries {
        match channel_id.say(&ctx.http, content).await {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                last_err = Some(e);
                if attempt < retries {
                    warn!(
                        "Discord API call failed (attempt {}/{}), retrying...",
                        attempt + 1,
                        retries + 1
                    );
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                }
            }
        }
    }

    Err(last_err.unwrap().into())
}
