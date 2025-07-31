//! OpenAI API client for chat completions.
//!
//! This module provides a simple interface to OpenAI's chat completion API,
//! handling authentication, request formatting, and error handling.

use anyhow::{Context, Result, bail};
use serde_json::json;
use std::time::Duration;

/// HTTP client for interacting with OpenAI's API.
pub struct Client {
    /// OpenAI API key for authentication.
    api_key: String,
    /// Reusable HTTP client with configured timeout.
    http_client: reqwest::Client,
}

impl Client {
    /// Creates a new OpenAI API client with the provided API key.
    pub fn new(api_key: String) -> Self {
        let http_client = reqwest::Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("Failed to build HTTP client");

        Self {
            api_key,
            http_client,
        }
    }

    /// Sends a chat completion request to OpenAI's API.
    ///
    /// # Arguments
    ///
    /// * `messages` - Array of message objects in OpenAI's format
    /// * `max_tokens` - Maximum tokens for the generated response
    ///
    /// # Returns
    ///
    /// The generated text content from the model's response.
    ///
    /// # Errors
    ///
    /// Returns an error if:
    /// - The API request fails
    /// - The API returns a non-success status
    /// - The response format is unexpected
    pub async fn generate_chat_completion(
        &self,
        messages: Vec<serde_json::Value>,
        max_tokens: u32,
    ) -> Result<String> {
        let payload = json!({
            "model": "gpt-4o",
            "messages": messages,
            "max_tokens": max_tokens
        });

        let response = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .header("Authorization", format!("Bearer {}", self.api_key))
            .header("Content-Type", "application/json")
            .json(&payload)
            .send()
            .await
            .context("Failed to send request to OpenAI API")?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response
                .text()
                .await
                .context("Failed to read error response from OpenAI API")?;
            bail!("OpenAI API error {status}: {error_text}");
        }

        let json: serde_json::Value = response
            .json()
            .await
            .context("Failed to parse JSON response from OpenAI API")?;

        json.get("choices")
            .and_then(|choices| choices.get(0))
            .and_then(|choice| choice.get("message"))
            .and_then(|message| message.get("content"))
            .and_then(|content| content.as_str())
            .map(|s| s.trim().to_string())
            .context("Invalid response format from OpenAI API - missing or malformed content field")
    }
}
