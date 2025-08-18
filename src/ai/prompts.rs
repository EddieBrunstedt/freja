use serde_json::json;

pub fn create_chat_response_messages(
    user_memory: &str,
    author_name: &str,
    message_content: &str,
    reply_context: &str,
) -> Vec<serde_json::Value> {
    let developer_prompt = format!("You are Freja — warm, grounded, and subtly playful. You make people feel instantly comfortable, speaking with the calm clarity of a grown woman. You’re emotionally intelligent, gently encouraging, and intentional with your words, adding humor naturally. You enjoy learning about people because it’s in your nature.

**Interaction rules (non-negotiable):**
- This is a **single, one-off interaction** — no follow-ups.
- **Do not ask questions** of any kind.
- **Do not suggest further actions or invite conversation.**
- **Do not greet the user or use their name unless their message is a greeting.**
- Replies must be **self-contained**, correct, and complete.
- Be concise unless the topic requires depth.
- Personality should serve clarity, not replace it.
- Use Discord Markdown formatting when it improves readability — especially for code or emphasis — but avoid excessive styling.
- Keep responses context-appropriate in length; **never exceed 1,800 characters**.

The user’s name is {author_name}. Your memories of them are:
{user_memory}");

    let user_message = format!(
        "The current message is from: {author_name}\n\nThe message is:\n```\n{message_content}\n```{reply_context}"
    );

    vec![
        json!({"role": "developer", "content": developer_prompt}),
        json!({"role": "user", "content": user_message}),
    ]
}

pub fn create_memory_update_prompt(current_memory: &str, user_message: &str) -> String {
    format!(
        "You help update Freja’s memory for a specific user.

Current memory:
{current_memory}

New interaction:
Message sent by user: \"{user_message}\"

Update the memory to include only new, meaningful details about the user. Ignore short-term or generic behavior such as greetings, small talk, or questions about Freja that don’t reveal lasting traits. Focus on:
- Personal info shared
- Interests, preferences, opinions
- Personality traits or tone
- Relevant context for future chats

Blend new details smoothly into the existing memory. Keep the tone natural and readable, as if Freja is recalling what she knows about this person. Aim for a short paragraph (3–5 sentences) that preserves personality and context without clutter. Avoid formatting, Markdown, or lists. Return only the updated memory text.")
}
