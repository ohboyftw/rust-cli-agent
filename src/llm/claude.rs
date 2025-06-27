use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::LLMClient;
use crate::error::AgentError;

pub struct ClaudeClient {
    api_key: String,
    http_client: Client,
}

#[derive(Serialize)]
struct ClaudeRequest<'a> {
    model: &'a str,
    max_tokens: u32,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct ClaudeResponse {
    content: Vec<ResponseContent>,
}

#[derive(Deserialize)]
struct ResponseContent {
    text: String,
}

impl ClaudeClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for ClaudeClient {
    async fn generate(&self, prompt: &str) -> Result<String, AgentError> {
        let request_payload = ClaudeRequest {
            model: "claude-3-opus-20240229",
            max_tokens: 4096,
            messages: vec![Message { role: "user", content: prompt }],
        };

        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("Claude API Error: {}", error_body)));
        }

        let response_data: ClaudeResponse = response.json().await?;

        response_data
            .content
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or_else(|| AgentError::ResponseParseError("No content in Claude response".to_string()))
    }
}
