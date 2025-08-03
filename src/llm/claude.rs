use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LLMClient, AIResponse, ModelInfo};
use crate::error::AgentError;

pub struct ClaudeClient {
    api_key: String,
    http_client: Client,
    model: String,
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
    usage: Usage,
}

#[derive(Deserialize)]
struct ResponseContent {
    text: String,
}

#[derive(Deserialize)]
struct Usage {
    input_tokens: u32,
    output_tokens: u32,
}

impl ClaudeClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            model: model.unwrap_or_else(|| "claude-3-opus-20240229".to_string()),
        }
    }
}

#[async_trait]
impl LLMClient for ClaudeClient {
    async fn generate(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let request_payload = ClaudeRequest {
            model: &self.model,
            max_tokens: 4096,
            messages: vec![Message { role: "user", content: prompt }],
        };
        self.send_request(request_payload).await
    }

    async fn generate_json(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        // Claude API does not have a direct JSON mode like OpenAI.
        // We'll just call the regular generate and hope for JSON in the response.
        self.generate(prompt).await
    }

    async fn get_model_info(&self) -> ModelInfo {
        // These are example costs for claude-3-opus. Real costs should be fetched or configured.
        ModelInfo {
            name: self.model.clone(),
            input_cost_per_token: 0.000015, // Example: $15 per 1M tokens
            output_cost_per_token: 0.000075, // Example: $75 per 1M tokens
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let model_info = futures::executor::block_on(self.get_model_info());
        (input_tokens as f64 * model_info.input_cost_per_token) +
        (output_tokens as f64 * model_info.output_cost_per_token)
    }
}

impl ClaudeClient {
    async fn send_request(&self, payload: ClaudeRequest<'_>) -> Result<AIResponse, AgentError> {
        let response = self
            .http_client
            .post("https://api.anthropic.com/v1/messages")
            .header("x-api-key", &self.api_key)
            .header("anthropic-version", "2023-06-01")
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("Claude API Error: {}", error_body)));
        }

        let response_data: ClaudeResponse = response.json().await?;

        let content = response_data
            .content
            .into_iter()
            .next()
            .map(|c| c.text)
            .ok_or_else(|| AgentError::ResponseParseError("No content in Claude response".to_string()))?;

        // Parse actual token usage from Claude API response
        let input_tokens = response_data.usage.input_tokens;
        let output_tokens = response_data.usage.output_tokens;
        let cost = self.calculate_cost(input_tokens, output_tokens);

        Ok(AIResponse {
            content,
            input_tokens,
            output_tokens,
            cost,
            model: self.model.clone(),
            provider: "Claude".to_string(),
        })
    }
}
