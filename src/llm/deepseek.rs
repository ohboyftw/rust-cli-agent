// Note: DeepSeek API is compatible with OpenAI's API format.
use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::LLMClient;
use crate::error::AgentError;

pub struct DeepSeekClient {
    api_key: String,
    http_client: Client,
}

#[derive(Serialize)]
struct DeepSeekRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Deserialize)]
struct DeepSeekResponse {
    choices: Vec<Choice>,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}


impl DeepSeekClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for DeepSeekClient {
    async fn generate(&self, prompt: &str) -> Result<String, AgentError> {
        let request_payload = DeepSeekRequest {
            model: "deepseek-coder",
            messages: vec![Message { role: "user", content: prompt }],
        };

        let response = self
            .http_client
            .post("https://api.deepseek.com/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("DeepSeek API Error: {}", error_body)));
        }

        let response_data: DeepSeekResponse = response.json().await?;

        response_data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AgentError::ResponseParseError("No content in DeepSeek response".to_string()))
    }
}
