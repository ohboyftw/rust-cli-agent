use async_trait::async_trait;
use reqwest::Client;
use serde::{Deserialize, Serialize};

use super::{LLMClient, AIResponse, ModelInfo};
use crate::error::AgentError;

pub struct GeminiClient {
    api_key: String,
    http_client: Client,
    model: String,
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
    prompt_feedback: Option<PromptFeedback>,
    usage_metadata: Option<UsageMetadata>,
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

#[derive(Deserialize)]
struct PromptFeedback {
    block_reason: Option<String>,
}

#[derive(Deserialize)]
struct UsageMetadata {
    prompt_token_count: u32,
    candidates_token_count: u32,
    total_token_count: u32,
}

impl GeminiClient {
    pub fn new(api_key: String, model: Option<String>) -> Self {
        Self {
            api_key,
            http_client: Client::new(),
            model: model.unwrap_or_else(|| "gemini-1.5-flash-2.5-pro".to_string()),
        }
    }
}

#[async_trait]
impl LLMClient for GeminiClient {
    async fn generate(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        let url = format!("https://generativelanguage.googleapis.com/v1beta/models/{}:generateContent?key={}", self.model, self.api_key);
        
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

        if let Some(feedback) = response_data.prompt_feedback {
            if let Some(reason) = feedback.block_reason {
                return Err(AgentError::LLMError(format!("Gemini API blocked prompt: {}", reason)));
            }
        }

        let content = response_data
            .candidates
            .into_iter()
            .next()
            .and_then(|c| c.content.parts.into_iter().next())
            .map(|p| p.text)
            .ok_or_else(|| AgentError::ResponseParseError("No content in Gemini response".to_string()))?;

        let (input_tokens, output_tokens) = if let Some(usage) = response_data.usage_metadata {
            (usage.prompt_token_count, usage.candidates_token_count)
        } else {
            (0, 0) // Fallback if usage_metadata is not present
        };

        let cost = self.calculate_cost(input_tokens, output_tokens);

        Ok(AIResponse {
            content,
            input_tokens,
            output_tokens,
            cost,
            model: self.model.clone(),
            provider: "Gemini".to_string(),
        })
    }

    async fn generate_json(&self, prompt: &str) -> Result<AIResponse, AgentError> {
        // Gemini API does not have a direct JSON mode like OpenAI.
        // We'll just call the regular generate and hope for JSON in the response.
        self.generate(prompt).await
    }

    async fn get_model_info(&self) -> ModelInfo {
        // These are example costs for gemini-1.5-flash-2.5-pro. Real costs should be fetched or configured.
        ModelInfo {
            name: self.model.clone(),
            input_cost_per_token: 0.00000035, // Example: $0.35 per 1M tokens
            output_cost_per_token: 0.00000105, // Example: $1.05 per 1M tokens
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        let model_info = futures::executor::block_on(self.get_model_info());
        (input_tokens as f64 * model_info.input_cost_per_token) +
        (output_tokens as f64 * model_info.output_cost_per_token)
    }
}
