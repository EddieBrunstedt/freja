//! Message event handler for processing Discord messages.
//!
//! This module handles incoming messages, determines when the bot should respond,
//! and manages the interaction flow including AI response generation and memory updates.

use crate::{
    ai,
    data::{Data, Error},
};
use anyhow::{Context, Result};
use poise::serenity_prelude as serenity;
use serde_json::json;
use std::time::Duration;
use tracing::{error, info, warn};

/// Handles all Discord events, primarily focusing on message processing.
///
/// This is the main event handler that receives all Discord events and
/// routes them appropriately. Currently handles:
/// - Ready events for logging bot startup
/// - Message events for user interactions
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
            if should_bot_respond(new_message, data.bot_id)
                && let Err(e) = process_message_and_respond(ctx, new_message, data).await
            {
                error!(
                    "Failed to handle message from {}: {e:?}",
                    new_message.author.display_name()
                );
            }
        }
        _ => (),
    }
    Ok(())
}

/// Determines whether the bot should respond to a given message.
///
/// The bot responds when:
/// - It is directly mentioned in the message
/// - Its name "freja" appears in the message content (case-insensitive)
fn should_bot_respond(message: &serenity::Message, bot_id: serenity::UserId) -> bool {
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

/// Processes a message that the bot should respond to.
///
/// This function:
/// 1. Shows typing indicator while processing
/// 2. Retrieves user's conversation memory
/// 3. Gets reply context if the message is a reply
/// 4. Generates AI response
/// 5. Sends the response to Discord
/// 6. Updates user memory in the background
async fn process_message_and_respond(
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

    let reply_context = fetch_reply_context(ctx, message, data.bot_id).await;

    let messages = ai::create_chat_response_messages(
        &user_memory,
        message.author.display_name(),
        &message.content,
        &reply_context,
    );

    match data.ai_client.generate_chat_completion(messages).await {
        Ok(response) => {
            if let Err(e) =
                send_discord_message_with_retry(ctx, message.channel_id, &response, 2).await
            {
                error!("Failed to send response message to Discord after retries: {e:?}");

                // Try to send a fallback error message to let the user know something went wrong
                if let Err(fallback_err) = send_discord_message_without_retry(
                    ctx,
                    message.channel_id,
                    "Sorry, I had an issue sending my response. Please try again! 😅",
                )
                .await
                {
                    error!("Failed to send fallback error message: {fallback_err:?}");
                }
                return Ok(());
            }

            let user_memories = data.user_memories.clone();
            let ai_client = data.ai_client.clone();
            let user_id = message.author.id;
            let message_content = message.content.clone();

            tokio::spawn(async move {
                if let Err(e) = update_user_conversation_memory(
                    &user_memories,
                    &ai_client,
                    user_id,
                    &user_memory,
                    &message_content,
                )
                .await
                {
                    error!("Failed to update user memory: {e:?}");
                }
            });
        }
        Err(e) => {
            error!("AI response error: {e:?}");
            if let Err(send_err) = send_discord_message_with_retry(
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

/// Retrieves context if the current message is a reply to another message.
///
/// Fetches the referenced message content and formats it for inclusion
/// in the AI prompt to maintain conversation continuity.
async fn fetch_reply_context(
    ctx: &serenity::Context,
    message: &serenity::Message,
    bot_id: serenity::UserId,
) -> String {
    if let Some(referenced) = &message.referenced_message {
        return format_message_as_reply_context(referenced, bot_id);
    }

    if let Some(reference) = &message.message_reference
        && let Some(message_id) = reference.message_id
    {
        match message.channel_id.message(&ctx.http, message_id).await {
            Ok(referenced) => return format_message_as_reply_context(&referenced, bot_id),
            Err(e) => {
                info!("Could not fetch referenced message {message_id}: {e}");
            }
        }
    }

    String::new()
}

/// Formats a referenced message into a context string for the AI.
///
/// Identifies whether the referenced message was from the bot itself
/// or another user to provide appropriate context.
fn format_message_as_reply_context(
    message: &serenity::Message,
    bot_id: serenity::UserId,
) -> String {
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

/// Updates the stored memory for a user based on the latest interaction.
///
/// Uses AI to intelligently merge new conversation details with existing
/// memory, focusing on meaningful information worth remembering.
///
/// # Errors
///
/// Returns an error if:
/// - AI memory generation fails
/// - Memory store update fails
async fn update_user_conversation_memory(
    user_memories: &crate::memory::MemoryStore,
    ai_client: &ai::Client,
    user_id: serenity::UserId,
    current_memory: &str,
    user_message: &str,
) -> Result<()> {
    let prompt = ai::create_memory_update_prompt(current_memory, user_message);

    let messages = vec![json!({ "role": "user", "content": prompt })];

    let updated_memory = ai_client
        .generate_chat_completion(messages)
        .await
        .context("Failed to generate updated memory via AI")?;
    user_memories
        .update(user_id, updated_memory)
        .await
        .context("Failed to save updated memory to store")?;

    Ok(())
}

/// Sends a simple Discord message without retries or file handling.
///
/// Used as a fallback when the main message sending fails to avoid recursion.
async fn send_discord_message_without_retry(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    content: &str,
) -> Result<serenity::Message> {
    channel_id.say(&ctx.http, content).await.map_err(Into::into)
}

/// Sends a Discord message with automatic retry on failure.
///
/// Implements exponential backoff for handling transient Discord API errors.
/// If the content exceeds Discord's 2000 character limit, it sends the content as a file instead.
///
/// # Arguments
///
/// * `retries` - Number of retry attempts after initial failure
async fn send_discord_message_with_retry(
    ctx: &serenity::Context,
    channel_id: serenity::ChannelId,
    content: &str,
    max_retries: u8,
) -> Result<serenity::Message> {
    let mut last_err = None;

    for attempt in 0..=max_retries {
        let result = if content.len() > 2000 {
            // Send as file if content is too long
            let attachment = serenity::CreateAttachment::bytes(content.as_bytes(), "response.txt");
            let message = serenity::CreateMessage::new()
                .content("Response is too long, sending as file:")
                .add_file(attachment);
            channel_id.send_message(&ctx.http, message).await
        } else {
            // Send as regular message
            channel_id.say(&ctx.http, content).await
        };

        match result {
            Ok(msg) => return Ok(msg),
            Err(e) => {
                last_err = Some(e);
                if attempt < max_retries {
                    warn!(
                        "Discord API call failed (attempt {}/{}), retrying...",
                        attempt + 1,
                        max_retries + 1
                    );
                    tokio::time::sleep(Duration::from_millis(500 * (attempt as u64 + 1))).await;
                }
            }
        }
    }

    Err(last_err.unwrap().into())
}
