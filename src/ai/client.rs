//! OpenAI API client for chat completions.
//!
//! This module provides a simple interface to OpenAI's chat completion API,
//! handling authentication, request formatting, and error handling.

use anyhow::{Context, Result, bail};
use serde_json::{Value, json};
use std::time::Duration;

const RAW_DUMP_LIMIT: usize = 8_000; // keep logs sane

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
            .timeout(Duration::from_secs(120))
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
    ) -> Result<String> {
        let payload = json!({
            "model": "gpt-5",
            "input": messages
        });

        let response = self
            .http_client
            .post("https://api.openai.com/v1/responses")
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

        extract_message_from_response(&json)
    }
}

pub fn extract_message_from_response(json: &Value) -> Result<String> {
    // Helpful status diagnostics if present
    if let Some(status) = json.get("status").and_then(|x| x.as_str())
        && status != "completed"
    {
        let reason = json
            .get("incomplete_details")
            .and_then(|d| d.get("reason"))
            .and_then(|r| r.as_str())
            .unwrap_or("unknown");

        return Err(anyhow::anyhow!(
            "Response not completed (status: {status}, reason: {reason}). Raw: {}",
            json_snippet(json)
        ));
    }

    // 1) Fast path: top-level output_text
    if let Some(s) = json.get("output_text").and_then(|v| v.as_str()) {
        return Ok(s.trim().to_string());
    }

    // 2) Common path: first output item's content blocks
    if let Some(arr) = json
        .get("output")
        .and_then(|o| o.get(0))
        .and_then(|item| item.get("content"))
        .and_then(|c| c.as_array())
    {
        // Prefer explicit text-like blocks
        if let Some(s) = arr
            .iter()
            .find_map(|c| c.get("text").and_then(|t| t.as_str()))
        {
            return Ok(s.trim().to_string());
        }
        // Some payloads may label it differently; try a broader scan
        if let Some(s) = arr
            .iter()
            .find_map(|c| c.get("output_text").and_then(|t| t.as_str()))
        {
            return Ok(s.trim().to_string());
        }
    }

    // 3) Global fallback: scan all output items/contents for any text field
    if let Some(items) = json.get("output").and_then(|o| o.as_array()) {
        for item in items {
            if let Some(contents) = item.get("content").and_then(|c| c.as_array()) {
                for c in contents {
                    if let Some(t) = c
                        .get("text")
                        .and_then(|t| t.as_str())
                        .or_else(|| c.get("output_text").and_then(|t| t.as_str()))
                    {
                        return Ok(t.trim().to_string());
                    }
                }
            }
        }
    }

    Err(anyhow::anyhow!(
        "Invalid response format: no text found. Raw: {}",
        json_snippet(json)
    ))
}

fn json_snippet(v: &Value) -> String {
    let s = serde_json::to_string_pretty(v).unwrap_or_else(|_| "<unprintable JSON>".into());
    if s.len() <= RAW_DUMP_LIMIT {
        s
    } else {
        format!(
            "{}… [truncated {} chars]",
            &s[..RAW_DUMP_LIMIT],
            s.len() - RAW_DUMP_LIMIT
        )
    }
}
