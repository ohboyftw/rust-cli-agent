use crate::error::AgentError;
use std::env;

#[derive(Debug, Clone)]
pub struct AppConfig {
    pub openai_api_key: Option<String>,
    pub anthropic_api_key: Option<String>,
    pub google_api_key: Option<String>,
    pub deepseek_api_key: Option<String>,
    pub brave_search_api_key: Option<String>,
    pub ollama_base_url: String,
    pub ollama_model: String,
}

impl AppConfig {
    pub fn load() -> Result<Self, AgentError> {
        Ok(Self {
            openai_api_key: env::var("OPENAI_API_KEY").ok(),
            anthropic_api_key: env::var("ANTHROPIC_API_KEY").ok(),
            google_api_key: env::var("GOOGLE_API_KEY").ok(),
            deepseek_api_key: env::var("DEEPSEEK_API_KEY").ok(),
            brave_search_api_key: env::var("BRAVE_SEARCH_API_KEY").ok(),
            ollama_base_url: env::var("OLLAMA_BASE_URL").unwrap_or_else(|_| "http://localhost:11434".to_string()),
            ollama_model: env::var("OLLAMA_MODEL").unwrap_or_else(|_| "llama3".to_string()),
        })
    }
}
