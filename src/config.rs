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

    #[cfg(test)]
    pub fn test_config() -> Self {
        Self {
            openai_api_key: Some("test_openai_key".to_string()),
            anthropic_api_key: Some("test_anthropic_key".to_string()),
            google_api_key: Some("test_google_key".to_string()),
            deepseek_api_key: Some("test_deepseek_key".to_string()),
            brave_search_api_key: Some("test_brave_key".to_string()),
            ollama_base_url: "http://localhost:11434".to_string(),
            ollama_model: "llama3".to_string(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use serial_test::serial;
    use std::env;

    #[test]
    #[serial]
    fn test_config_load_with_env_vars() {
        // Set test environment variables
        env::set_var("OPENAI_API_KEY", "test_openai");
        env::set_var("ANTHROPIC_API_KEY", "test_anthropic");
        env::set_var("GOOGLE_API_KEY", "test_google");
        env::set_var("DEEPSEEK_API_KEY", "test_deepseek");
        env::set_var("BRAVE_SEARCH_API_KEY", "test_brave");
        env::set_var("OLLAMA_BASE_URL", "http://custom:8080");
        env::set_var("OLLAMA_MODEL", "custom_model");

        let config = AppConfig::load().unwrap();

        assert_eq!(config.openai_api_key, Some("test_openai".to_string()));
        assert_eq!(config.anthropic_api_key, Some("test_anthropic".to_string()));
        assert_eq!(config.google_api_key, Some("test_google".to_string()));
        assert_eq!(config.deepseek_api_key, Some("test_deepseek".to_string()));
        assert_eq!(config.brave_search_api_key, Some("test_brave".to_string()));
        assert_eq!(config.ollama_base_url, "http://custom:8080");
        assert_eq!(config.ollama_model, "custom_model");

        // Cleanup
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("GOOGLE_API_KEY");
        env::remove_var("DEEPSEEK_API_KEY");
        env::remove_var("BRAVE_SEARCH_API_KEY");
        env::remove_var("OLLAMA_BASE_URL");
        env::remove_var("OLLAMA_MODEL");
    }

    #[test]
    #[serial]
    fn test_config_load_with_defaults() {
        // Ensure no API keys are set
        env::remove_var("OPENAI_API_KEY");
        env::remove_var("ANTHROPIC_API_KEY");
        env::remove_var("GOOGLE_API_KEY");
        env::remove_var("DEEPSEEK_API_KEY");
        env::remove_var("BRAVE_SEARCH_API_KEY");
        env::remove_var("OLLAMA_BASE_URL");
        env::remove_var("OLLAMA_MODEL");

        let config = AppConfig::load().unwrap();

        assert_eq!(config.openai_api_key, None);
        assert_eq!(config.anthropic_api_key, None);
        assert_eq!(config.google_api_key, None);
        assert_eq!(config.deepseek_api_key, None);
        assert_eq!(config.brave_search_api_key, None);
        assert_eq!(config.ollama_base_url, "http://localhost:11434");
        assert_eq!(config.ollama_model, "llama3");
    }

    #[test]
    fn test_config_clone() {
        let config = AppConfig::test_config();
        let cloned = config.clone();

        assert_eq!(config.openai_api_key, cloned.openai_api_key);
        assert_eq!(config.ollama_base_url, cloned.ollama_base_url);
    }

    #[test]
    fn test_config_debug() {
        let config = AppConfig::test_config();
        let debug_str = format!("{:?}", config);
        assert!(debug_str.contains("AppConfig"));
        assert!(debug_str.contains("openai_api_key"));
    }
}