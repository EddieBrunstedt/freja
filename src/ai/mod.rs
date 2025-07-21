mod client;
mod prompts;

pub use client::Client;
pub use prompts::{generate_memory_prompt, generate_response_prompt};
