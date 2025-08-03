use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LLMClient, AIResponse, ModelInfo};
use crate::error::AgentError;

pub struct OpenAIClient {
    api_key: String,
    http_client: Client,
    model: String,
}

#[derive(Serialize)]
struct OpenAIRequest<'a> {
    model: &'a str,
    messages: Vec<Message<'a>>,
    temperature: f32,
    response_format: Option<ResponseFormat<'a>>,
}

#[derive(Serialize)]
struct Message<'a> {
    role: &'a str,
    content: &'a str,
}

#[derive(Serialize)]
struct ResponseFormat<'a> {
    r#type: &'a str,
}

#[derive(Deserialize)]
struct OpenAIResponse {
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

impl OpenAIClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            model: model.unwrap_or_else(|| "gpt-4o".to_string()),
        }
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn generate(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let request_payload = OpenAIRequest {
            model: &self.model,
            messages: vec![Message { role: "user", content: prompt }],
            temperature: 0.2,
            response_format: None,
        };
        self.send_request(request_payload).await
    }
    
    async fn generate_json(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let request_payload = OpenAIRequest {
            model: &self.model,
            messages: vec![Message { role: "user", content: prompt }],
            temperature: 0.0,
            response_format: Some(ResponseFormat { r#type: "json_object" }),
        };
        self.send_request(request_payload).await
    }

    async fn get_model_info(&self) -> ModelInfo {
        // These are example costs for gpt-4o. Real costs should be fetched or configured.
        ModelInfo {
            name: self.model.clone(),
            input_cost_per_token: 0.000005, // Example: $5 per 1M tokens
            output_cost_per_token: 0.000015, // Example: $15 per 1M tokens
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let model_info = futures::executor::block_on(self.get_model_info());
        (input_tokens as f64 * model_info.input_cost_per_token) + 
        (output_tokens as f64 * model_info.output_cost_per_token)
    }
}

impl OpenAIClient {
    async fn send_request(&self, payload: OpenAIRequest<'_>) -> Result<AIResponse, AgentError> {
        let response = self
            .http_client
            .post("https://api.openai.com/v1/chat/completions")
            .bearer_auth(&self.api_key)
            .json(&payload)
            .send()
            .await?;
            
        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("OpenAI API Error: {}", error_body)));
        }

        let response_data: OpenAIResponse = response.json().await?;
        let content = response_data.choices.into_iter().next().map(|c| c.message.content)
            .ok_or_else(|| AgentError::ResponseParseError("No content in OpenAI response".to_string()))?;

        let input_tokens = response_data.usage.prompt_tokens;
        let output_tokens = response_data.usage.completion_tokens;
        let cost = self.calculate_cost(input_tokens, output_tokens);

        Ok(AIResponse {
            content,
            input_tokens,
            output_tokens,
            cost,
            model: self.model.clone(),
            provider: "OpenAI".to_string(),
        })
    }
}
