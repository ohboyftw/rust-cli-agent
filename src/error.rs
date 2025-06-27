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
