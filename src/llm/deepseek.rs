use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LLMClient, AIResponse, ModelInfo};
use crate::error::AgentError;

pub struct DeepSeekClient {
    api_key: String,
    http_client: Client,
    model: String,
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
    usage: Usage,
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

#[derive(Deserialize)]
struct Usage {
    prompt_tokens: u32,
    completion_tokens: u32,
    total_tokens: u32,
}

impl DeepSeekClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            model: model.unwrap_or_else(|| "deepseek-coder".to_string()),
        }
    }
}

#[async_trait]
impl LLMClient for DeepSeekClient {
    async fn generate(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let request_payload = DeepSeekRequest {
            model: &self.model,
            messages: vec![Message { role: "user", content: prompt }],
        };
        self.send_request(request_payload).await
    }

    async fn generate_json(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        // DeepSeek API is compatible with OpenAI's JSON mode
        let request_payload = DeepSeekRequest {
            model: &self.model,
            messages: vec![Message { role: "user", content: prompt }],
        };
        self.send_request(request_payload).await
    }

    async fn get_model_info(&self) -> ModelInfo {
        // These are example costs for deepseek-coder. Real costs should be fetched or configured.
        ModelInfo {
            name: self.model.clone(),
            input_cost_per_token: 0.0000001, // Example: $0.1 per 1M tokens
            output_cost_per_token: 0.0000001, // Example: $0.1 per 1M tokens
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let model_info = futures::executor::block_on(self.get_model_info());
        (input_tokens as f64 * model_info.input_cost_per_token) +
        (output_tokens as f64 * model_info.output_cost_per_token)
    }
}

impl DeepSeekClient {
    async fn send_request(&self, payload: DeepSeekRequest<'_>) -> Result<AIResponse, AgentError> {
        let response = self
            .http_client
            .post("https://api.deepseek.com/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("DeepSeek API Error: {}", error_body)));
        }

        let response_data: DeepSeekResponse = response.json().await?;

        let content = response_data
            .choices
            .into_iter()
            .next()
            .map(|c| c.message.content)
            .ok_or_else(|| AgentError::ResponseParseError("No content in DeepSeek response".to_string()))?;

        let input_tokens = response_data.usage.prompt_tokens;
        let output_tokens = response_data.usage.completion_tokens;
        let cost = self.calculate_cost(input_tokens, output_tokens);

        Ok(AIResponse {
            content,
            input_tokens,
            output_tokens,
            cost,
            model: self.model.clone(),
            provider: "DeepSeek".to_string(),
        })
    }
}
