use serde_json::json;

pub struct Client {
    api_key: String,
    http_client: reqwest::Client,
}

impl Client {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: reqwest::Client::new(),
        }
    }

    pub async fn chat_completion(
        &self,
        messages: Vec<serde_json::Value>,
        max_tokens: u32,
    ) -> Result<String, Box<dyn std::error::Error + Send + Sync>> {
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
            .await?;

        if !response.status().is_success() {
            let status = response.status();
            let error_text = response.text().await?;
            return Err(format!("OpenAI API error {status}: {error_text}").into());
        }

        let json: serde_json::Value = response.json().await?;

        json["choices"][0]["message"]["content"]
            .as_str()
            .map(|s| s.trim().to_string())
            .ok_or_else(|| "Invalid response format from OpenAI".into())
    }
}
