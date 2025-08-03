use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LLMClient, AIResponse, ModelInfo};
use crate::error::AgentError;

pub struct OllamaClient {
    base_url: String,
    model: String,
    http_client: Client,
}

#[derive(Serialize)]
struct OllamaRequest<'a> {
    model: &'a str,
    prompt: &'a str,
    stream: bool,
}

#[derive(Deserialize)]
struct OllamaResponse {
    response: String,
    prompt_eval_count: Option<u32>,
    eval_count: Option<u32>,
}

impl OllamaClient {
    pub fn new(base_url: &str, model: &str) -> Self {
        Self {
            base_url: base_url.to_string(),
            model: model.to_string(),
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for OllamaClient {
    async fn generate(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let url = format!("{}/api/generate", self.base_url);
        
        let request_payload = OllamaRequest {
            model: &self.model,
            prompt,
            stream: false,
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("Ollama API Error: {}", error_body)));
        }

        let response_data: OllamaResponse = response.json().await?;

        let input_tokens = response_data.prompt_eval_count.unwrap_or(0);
        let output_tokens = response_data.eval_count.unwrap_or(0);
        let cost = self.calculate_cost(input_tokens, output_tokens);

        Ok(AIResponse {
            content: response_data.response,
            input_tokens,
            output_tokens,
            cost,
            model: self.model.clone(),
            provider: "Ollama".to_string(),
        })
    }

    async fn generate_json(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        // Ollama does not have a direct JSON mode. We'll just call generate.
        self.generate(prompt).await
    }

    async fn get_model_info(&self) -> ModelInfo {
        // Ollama models are typically free or self-hosted, so cost is 0.
        ModelInfo {
            name: self.model.clone(),
            input_cost_per_token: 0.0,
            output_cost_per_token: 0.0,
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let model_info = futures::executor::block_on(self.get_model_info());
        (input_tokens as f64 * model_info.input_cost_per_token) +
        (output_tokens as f64 * model_info.output_cost_per_token)
    }
}
