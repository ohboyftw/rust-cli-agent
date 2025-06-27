use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::LLMClient;
use crate::error::AgentError;

pub struct GeminiClient {
    api_key: String,
    http_client: Client,
}

#[derive(Serialize)]
struct GeminiRequest<'a> {
    contents: Vec<Content<'a>>,
}

#[derive(Serialize)]
struct Content<'a> {
    parts: Vec<Part<'a>>,
}

#[derive(Serialize)]
struct Part<'a> {
    text: &'a str,
}

#[derive(Deserialize)]
struct GeminiResponse {
    candidates: Vec<Candidate>,
}

#[derive(Deserialize)]
struct Candidate {
    content: ResponseContent,
}

#[derive(Deserialize)]
struct ResponseContent {
    parts: Vec<ResponsePart>,
}

#[derive(Deserialize)]
struct ResponsePart {
    text: String,
}

impl GeminiClient {
    pub fn new(api_key: String) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
        }
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn generate(&self, prompt: &str) -> Result<String, AgentError> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/gemini-1.5-flash-2.5-pro:generateContent?key={}", self.api_key);
        
        let request_payload = GeminiRequest {
            contents: vec![Content {
                parts: vec![Part { text: prompt }],
            }],
        };

        let response = self
            .http_client
            .post(&url)
            .json(&request_payload)
            .send()
            .await?;

        if !response.status().is_success() {
            let error_body = response.text().await?;
            return Err(AgentError::LLMError(format!("Gemini API Error: {}", error_body)));
        }

        let response_data: GeminiResponse = response.json().await?;

        response_data
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| AgentError::ResponseParseError("No content in Gemini response".to_string()))
    }
}
