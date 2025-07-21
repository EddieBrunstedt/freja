use crate::data::{Context, Error};

/// Shows what Freja remembers about you
#[poise::command(slash_command, prefix_command)]
pub async fn memory(ctx: Context<'_>) -> Result<(), Error> {
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

    let response = format!("**What I remember about you:**\n{memory_content}");
    ctx.say(response).await?;
    Ok(())
}
