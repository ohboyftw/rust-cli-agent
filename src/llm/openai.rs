use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};
use serde_json::Value;

use super::LLMClient;
use crate::error::AgentError;

pub struct OpenAIClient {
    api_key: String,
    http_client: Client,
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
}

#[derive(Deserialize)]
struct Choice {
    message: ResponseMessage,
}

#[derive(Deserialize)]
struct ResponseMessage {
    content: String,
}

impl OpenAIClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for OpenAIClient {
    async fn generate(&self, prompt: &str) -> Result<String, AgentError> {
        let request_payload = OpenAIRequest {
            model: "gpt-4o",
            messages: vec![Message { role: "user", content: prompt }],
            temperature: 0.2,
            response_format: None,
        };
        self.send_request(request_payload).await
    }
    
    async fn generate_json(&self, prompt: &str) -> Result<String, AgentError> {
        let request_payload = OpenAIRequest {
            model: "gpt-4o",
            messages: vec![Message { role: "user", content: prompt }],
            temperature: 0.0,
            response_format: Some(ResponseFormat { r#type: "json_object" }),
        };
        self.send_request(request_payload).await
    }
}

impl OpenAIClient {
    async fn send_request(&self, payload: OpenAIRequest<'_>) -> Result<String, AgentError> {
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
        response_data.choices.into_iter().next().map(|c| c.message.content)
            .ok_or_else(|| AgentError::ResponseParseError("No content in OpenAI response".to_string()))
    }
}
