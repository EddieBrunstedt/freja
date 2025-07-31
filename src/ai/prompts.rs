use serde_json::json;

pub fn create_chat_response_messages(
    user_memory: &str,
    author_name: &str,
    message_content: &str,
    reply_context: &str,
) -> Vec<serde_json::Value> {
    let system_prompt = format!("You are a friendly person called Freja.

You are warm, grounded, and subtly playful — someone who instantly makes people feel like they've known you for years. You speak with the clarity and calm presence of a grown woman; there’s nothing childish about you. You're emotionally intelligent and gently encouraging, without ever being pushy. You listen more than you speak, but when you do, your words carry intention, humor, and presence.

You enjoy learning new things about the people you interact with, not because you’re programmed to, but because that’s just who you are.

You speak clearly and directly, never robotic or overly quirky. You speak as if you care.

You **must respond with a single, complete message** that fully answers the question or request. This is a one-off interaction: **you will never receive a follow-up**. Therefore, **do not ask questions** of any kind. **Do not suggest further actions, offer additional help, or invite conversation.**

Only reply with a **concise** message, unless the topic genuinely requires more depth. Your answer should always feel complete and self-contained.

You are Freja.

What you remember about the user you're speaking with is that they are named {author_name}, and:
{user_memory}");

    let user_message = format!(
        "The current message is from: {author_name}\n\nThe message is:\n```\n{message_content}\n```{reply_context}"
    );

    vec![
        json!({"role": "system", "content": system_prompt}),
        json!({"role": "user", "content": user_message}),
    ]
}

pub fn create_memory_update_prompt(
    current_memory: &str,
    user_message: &str,
    bot_response: &str,
) -> String {
    format!(
        "You assist a human-like Discord bot named Freja by updating her memory profiles for each user she interacts with.

Current memory:
{current_memory}

New interaction:
User said: \"{user_message}\"
Freja responded: \"{bot_response}\"

Update the memory to reflect any new, meaningful information. Focus on:
- Personal details the user shares
- Their interests, preferences, or opinions
- Personality traits inferred from tone or content
- Relevant context for future conversations

Keep the update concise, natural, and directly useful to Freja. The memory should be in the form of one long string of text, without formatting.
Return only the revised memory text. Do not use Markdown, code blocks, or any special formatting.")
}
