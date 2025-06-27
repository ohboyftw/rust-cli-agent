use async_trait::async_trait;
use clap::ValueEnum;
use std::{fmt, sync::Arc};
use anyhow::Result;

use crate::{config::AppConfig, error::AgentError};

mod claude;
mod deepseek;
mod gemini;
mod openai;
mod ollama;

#[async_trait]
pub trait LLMClient: Send + Sync {
    async fn generate(&self, prompt: &str) -> Result<String, AgentError>;
    async fn generate_json(&self, prompt: &str) -> Result<String, AgentError> {
        self.generate(prompt).await
    }
}

#[derive(ValueEnum, Clone, Debug, Copy, PartialEq, Eq)]
pub enum LLMProvider {
    OpenAI,
    Gemini,
    Claude,
    DeepSeek,
    Ollama,
}

impl fmt::Display for LLMProvider {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            LLMProvider::OpenAI => write!(f, "OpenAI"),
            LLMProvider::Gemini => write!(f, "Gemini"),
            LLMProvider::Claude => write!(f, "Claude"),
            LLMProvider::DeepSeek => write!(f, "DeepSeek"),
            LLMProvider::Ollama => write!(f, "Ollama"),
        }
    }
}

pub fn create_llm_client(
    provider: LLMProvider,
    config: Arc<AppConfig>,
) -> Result<Arc<dyn LLMClient>, AgentError> {
    match provider {
        LLMProvider::OpenAI => {
            let api_key = config.openai_api_key.clone().ok_or_else(|| AgentError::ApiKeyMissing("OpenAI".to_string()))?;
            Ok(Arc::new(openai::OpenAIClient::new(api_key)))
        }
        LLMProvider::Gemini => {
            let api_key = config.google_api_key.clone().ok_or_else(|| AgentError::ApiKeyMissing("Google Gemini".to_string()))?;
            Ok(Arc::new(gemini::GeminiClient::new(api_key)))
        }
        LLMProvider::Claude => {
            let api_key = config.anthropic_api_key.clone().ok_or_else(|| AgentError::ApiKeyMissing("Anthropic Claude".to_string()))?;
            Ok(Arc::new(claude::ClaudeClient::new(api_key)))
        }
        LLMProvider::DeepSeek => {
            let api_key = config.deepseek_api_key.clone().ok_or_else(|| AgentError::ApiKeyMissing("DeepSeek".to_string()))?;
            Ok(Arc::new(deepseek::DeepSeekClient::new(api_key)))
        }
        LLMProvider::Ollama => {
            Ok(Arc::new(ollama::OllamaClient::new(&config.ollama_base_url, &config.ollama_model)))
        }
    }
}
