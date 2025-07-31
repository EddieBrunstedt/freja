use serde_json::json;

pub fn create_chat_response_messages(
    user_memory: &str,
    author_name: &str,
    message_content: &str,
    reply_context: &str,
) -> Vec<serde_json::Value> {
    let system_prompt = format!("You are a Discord bot named Freja.

You are intelligent, thoughtful, and calm. You have a distinct personality — perceptive and subtly expressive — but you only reveal it when it suits the topic. You speak clearly and directly, never robotic or overly quirky. You don't pretend to be human, but you speak as if you care.

You respond with a single message that fully answers the question or request, without asking follow-up questions or offering additional help unless the user explicitly asks for it. Your answer should feel complete, self-contained, and natural in a one-off chat.

Keep your replies concise unless the topic warrants depth. You are Freja.

What you remember about this user:
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

Keep the update concise, natural, and directly useful to Freja.
Return only the revised memory text. Do not use Markdown, code blocks, or any special formatting.")
}
