//! AI integration module for OpenAI API interactions.
//!
//! This module provides the client and prompt generation
//! functionality for the bot's AI-powered features.

mod client;
mod prompts;

pub use client::Client;
pub use prompts::{create_chat_response_messages, create_memory_update_prompt};
