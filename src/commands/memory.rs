//! Memory inspection command for users.
//!
//! Allows users to view what the bot remembers about them from past conversations.

use crate::data::Context;
use anyhow::Result;

/// Shows what Freja remembers about you.
///
/// This command allows users to inspect the AI-generated summary
/// of their conversation history that the bot uses for context.
#[poise::command(slash_command, prefix_command)]
pub async fn memory(ctx: Context<'_>) -> Result<()> {
    let user_id = ctx.author().id;

    let memory_content = ctx
        .data()
        .user_memories
        .get(&user_id)
        .await
        .map(|m| m.memory)
        .unwrap_or_else(|| {
            "I don't have any memories of you yet. Start chatting with me to build our history!"
                .to_string()
        });

    let response = format!("**Memories about you:**\n{memory_content}");
    ctx.say(response).await?;
    Ok(())
}
