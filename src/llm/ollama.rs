use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::LLMClient;
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
    async fn generate(&self, prompt: &str) -> Result<String, AgentError> {
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

        Ok(response_data.response)
    }
}
