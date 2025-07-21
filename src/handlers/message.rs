use crate::{ai, data::Data};
use poise::serenity_prelude as serenity;
use serde_json::json;

type Error = Box<dyn std::error::Error + Send + Sync>;

pub async fn handle_event(
    ctx: &serenity::Context,
    event: &serenity::FullEvent,
    _framework: poise::FrameworkContext<'_, Data, Error>,
    data: &Data,
) -> Result<(), Error> {
    if let serenity::FullEvent::Message { new_message } = event {
        // Ignore bot messages
        if new_message.author.bot {
            return Ok(());
        }

        if should_respond(new_message, data.bot_id) {
            handle_message(ctx, new_message, data).await?;
        }
    }
    Ok(())
}

fn should_respond(message: &serenity::Message, bot_id: serenity::UserId) -> bool {
    let content_lower = message.content.to_lowercase();
    content_lower.contains("freja") || message.mentions.iter().any(|user| user.id == bot_id)
}

async fn handle_message(
    ctx: &serenity::Context,
    message: &serenity::Message,
    data: &Data,
) -> Result<(), Error> {
    let _typing = message.channel_id.start_typing(&ctx.http);

    // Get user memory
    let user_memory = data
        .user_memories
        .get(&message.author.id)
        .await
        .map(|m| m.memory)
        .unwrap_or_else(|| "No previous interactions with this user.".to_string());

    // Get reply context
    let reply_context = get_reply_context(ctx, message, data.bot_id).await;

    // Generate response
    let prompt = ai::generate_response_prompt(
        &user_memory,
        message.author.display_name(),
        &message.content,
        &reply_context,
    );

    let messages = vec![
        json!({ "role": "system", "content": prompt }),
        json!({ "role": "user", "content": &message.content }),
    ];

    match data
        .ai_client
        .chat_completion(messages, data.config.max_response_tokens)
        .await
    {
        Ok(response) => {
            message.channel_id.say(&ctx.http, &response).await?;

            // Update memory in background
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
                    eprintln!("Error updating user memory: {e:?}");
                }
            });
        }
        Err(e) => {
            eprintln!("AI response error: {e:?}");
            message
                .channel_id
                .say(
                    &ctx.http,
                    "*sighs* I'm having trouble thinking right now... 😔",
                )
                .await?;
        }
    }

    Ok(())
}

async fn get_reply_context(
    ctx: &serenity::Context,
    message: &serenity::Message,
    bot_id: serenity::UserId,
) -> String {
    // Try cached reference first
    if let Some(referenced) = &message.referenced_message {
        return format_reply_context(referenced, bot_id);
    }

    // Try to fetch if not cached
    if let Some(reference) = &message.message_reference {
        if let Some(message_id) = reference.message_id {
            if let Ok(referenced) = message.channel_id.message(&ctx.http, message_id).await {
                return format_reply_context(&referenced, bot_id);
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
) -> Result<(), Error> {
    let prompt = ai::generate_memory_prompt(current_memory, new_message, freja_response);

    let messages = vec![json!({ "role": "user", "content": prompt })];

    let updated_memory = ai_client.chat_completion(messages, max_tokens).await?;
    user_memories.update(user_id, updated_memory).await?;

    Ok(())
}
