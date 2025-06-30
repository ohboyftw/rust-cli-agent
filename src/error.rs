use thiserror::Error;

#[derive(Error, Debug)]
pub enum AgentError {
    #[error("Configuration error: {0}")]
    ConfigError(String),
    #[error("LLM provider error: {0}")]
    LLMError(String),
    #[error("API key for {0} is not set in the environment variables")]
    ApiKeyMissing(String),
    #[error("Tool execution failed: {0}")]
    ToolError(String),
    #[error("I/O error: {0}")]
    IoError(#[from] std::io::Error),
    #[error("WalkDir error: {0}")]
    WalkDirError(#[from] walkdir::Error),
    #[error("Network request failed: {0}")]
    RequestError(#[from] reqwest::Error),
    #[error("JSON serialization/deserialization failed: {0}")]
    JsonError(#[from] serde_json::Error),
    #[error("Failed to parse LLM response: {0}")]
    ResponseParseError(String),
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::io::{Error as IoError, ErrorKind};

    #[test]
    fn test_error_display() {
        let error = AgentError::ConfigError("test config error".to_string());
        assert_eq!(error.to_string(), "Configuration error: test config error");

        let error = AgentError::LLMError("test llm error".to_string());
        assert_eq!(error.to_string(), "LLM provider error: test llm error");

        let error = AgentError::ApiKeyMissing("OpenAI".to_string());
        assert_eq!(error.to_string(), "API key for OpenAI is not set in the environment variables");

        let error = AgentError::ToolError("test tool error".to_string());
        assert_eq!(error.to_string(), "Tool execution failed: test tool error");

        let error = AgentError::ResponseParseError("invalid json".to_string());
        assert_eq!(error.to_string(), "Failed to parse LLM response: invalid json");
    }

    #[test]
    fn test_error_debug() {
        let error = AgentError::ConfigError("test".to_string());
        let debug_str = format!("{:?}", error);
        assert!(debug_str.contains("ConfigError"));
        assert!(debug_str.contains("test"));
    }

    #[test]
    fn test_io_error_conversion() {
        let io_error = IoError::new(ErrorKind::NotFound, "file not found");
        let agent_error: AgentError = io_error.into();
        
        match agent_error {
            AgentError::IoError(_) => {
                assert!(agent_error.to_string().contains("I/O error"));
            }
            _ => panic!("Expected IoError variant"),
        }
    }

    #[test]
    fn test_json_error_conversion() {
        let json_str = "invalid json {";
        let json_error = serde_json::from_str::<serde_json::Value>(json_str).unwrap_err();
        let agent_error: AgentError = json_error.into();
        
        match agent_error {
            AgentError::JsonError(_) => {
                assert!(agent_error.to_string().contains("JSON serialization/deserialization failed"));
            }
            _ => panic!("Expected JsonError variant"),
        }
    }

    #[test]
    fn test_error_variants() {
        // Test that all error variants can be created
        let errors = vec![
            AgentError::ConfigError("config".to_string()),
            AgentError::LLMError("llm".to_string()),
            AgentError::ApiKeyMissing("provider".to_string()),
            AgentError::ToolError("tool".to_string()),
            AgentError::ResponseParseError("parse".to_string()),
        ];

        for error in errors {
            assert!(!error.to_string().is_empty());
        }
    }
}
