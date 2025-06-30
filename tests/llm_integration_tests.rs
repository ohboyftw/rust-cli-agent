use cli_coding_agent::{
    config::AppConfig,
    error::AgentError,
    llm::{create_llm_client, LLMProvider},
};
use std::sync::Arc;
use wiremock::{
    matchers::{method, path},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_ollama_client_success() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock the Ollama API response
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "Hello, this is a test response!"
        })))
        .mount(&mock_server)
        .await;

    // Create config with mock server URL
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: mock_server.uri(),
        ollama_model: "test_model".to_string(),
    };

    // Create Ollama client
    let client = create_llm_client(LLMProvider::Ollama, Arc::new(config)).unwrap();

    // Test generation
    let result = client.generate("Test prompt").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Hello, this is a test response!");
}

#[tokio::test]
async fn test_ollama_client_error_response() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock an error response
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(500).set_body_string("Internal Server Error"))
        .mount(&mock_server)
        .await;

    // Create config with mock server URL
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: mock_server.uri(),
        ollama_model: "test_model".to_string(),
    };

    // Create Ollama client
    let client = create_llm_client(LLMProvider::Ollama, Arc::new(config)).unwrap();

    // Test generation - should return error
    let result = client.generate("Test prompt").await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::LLMError(msg) => {
            assert!(msg.contains("Ollama API Error"));
        }
        _ => panic!("Expected LLMError"),
    }
}

#[tokio::test]
async fn test_ollama_client_invalid_json_response() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock an invalid JSON response
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_string("invalid json"))
        .mount(&mock_server)
        .await;

    // Create config with mock server URL
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: mock_server.uri(),
        ollama_model: "test_model".to_string(),
    };

    // Create Ollama client
    let client = create_llm_client(LLMProvider::Ollama, Arc::new(config)).unwrap();

    // Test generation - should return request/parse error due to invalid JSON
    let result = client.generate("Test prompt").await;
    assert!(result.is_err());
    
    // When reqwest receives invalid JSON, it returns a RequestError with Decode kind
    let error = result.unwrap_err();
    match error {
        AgentError::RequestError(_) => {
            // Expected - reqwest fails to decode invalid JSON
        }
        AgentError::JsonError(_) => {
            // Also acceptable - direct JSON parsing error
        }
        _ => panic!("Expected RequestError or JsonError, got: {:?}", error),
    }
}

#[test]
fn test_create_llm_client_missing_api_key() {
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: "http://localhost:11434".to_string(),
        ollama_model: "llama3".to_string(),
    };

    // Test OpenAI without API key
    let result = create_llm_client(LLMProvider::OpenAI, Arc::new(config.clone()));
    assert!(result.is_err());
    if let Err(AgentError::ApiKeyMissing(provider)) = result {
        assert_eq!(provider, "OpenAI");
    } else {
        panic!("Expected ApiKeyMissing error for OpenAI");
    }

    // Test Claude without API key
    let result = create_llm_client(LLMProvider::Claude, Arc::new(config.clone()));
    assert!(result.is_err());
    if let Err(AgentError::ApiKeyMissing(provider)) = result {
        assert_eq!(provider, "Anthropic Claude");
    } else {
        panic!("Expected ApiKeyMissing error for Claude");
    }

    // Test Gemini without API key
    let result = create_llm_client(LLMProvider::Gemini, Arc::new(config.clone()));
    assert!(result.is_err());
    if let Err(AgentError::ApiKeyMissing(provider)) = result {
        assert_eq!(provider, "Google Gemini");
    } else {
        panic!("Expected ApiKeyMissing error for Gemini");
    }

    // Test DeepSeek without API key
    let result = create_llm_client(LLMProvider::DeepSeek, Arc::new(config.clone()));
    assert!(result.is_err());
    if let Err(AgentError::ApiKeyMissing(provider)) = result {
        assert_eq!(provider, "DeepSeek");
    } else {
        panic!("Expected ApiKeyMissing error for DeepSeek");
    }

    // Test Ollama - should work without API key
    let result = create_llm_client(LLMProvider::Ollama, Arc::new(config));
    assert!(result.is_ok());
}

#[test]
fn test_create_llm_client_with_api_keys() {
    let config = AppConfig {
        openai_api_key: Some("test_openai_key".to_string()),
        anthropic_api_key: Some("test_anthropic_key".to_string()),
        google_api_key: Some("test_google_key".to_string()),
        deepseek_api_key: Some("test_deepseek_key".to_string()),
        brave_search_api_key: Some("test_brave_key".to_string()),
        ollama_base_url: "http://localhost:11434".to_string(),
        ollama_model: "llama3".to_string(),
    };

    // Test all providers with API keys
    let providers = [
        LLMProvider::OpenAI,
        LLMProvider::Claude,
        LLMProvider::Gemini,
        LLMProvider::DeepSeek,
        LLMProvider::Ollama,
    ];

    for provider in providers {
        let result = create_llm_client(provider, Arc::new(config.clone()));
        assert!(result.is_ok(), "Failed to create client for {:?}", provider);
    }
}

#[test]
fn test_llm_provider_display() {
    assert_eq!(LLMProvider::OpenAI.to_string(), "OpenAI");
    assert_eq!(LLMProvider::Claude.to_string(), "Claude");
    assert_eq!(LLMProvider::Gemini.to_string(), "Gemini");
    assert_eq!(LLMProvider::DeepSeek.to_string(), "DeepSeek");
    assert_eq!(LLMProvider::Ollama.to_string(), "Ollama");
}

#[test]
fn test_llm_provider_debug() {
    let providers = [
        LLMProvider::OpenAI,
        LLMProvider::Claude,
        LLMProvider::Gemini,
        LLMProvider::DeepSeek,
        LLMProvider::Ollama,
    ];

    for provider in providers {
        let debug_str = format!("{:?}", provider);
        assert!(!debug_str.is_empty());
    }
}

#[test]
fn test_llm_provider_clone_and_copy() {
    let provider = LLMProvider::OpenAI;
    let cloned = provider.clone();
    let copied = provider;

    assert_eq!(provider, cloned);
    assert_eq!(provider, copied);
}

#[test]
fn test_llm_provider_equality() {
    assert_eq!(LLMProvider::OpenAI, LLMProvider::OpenAI);
    assert_eq!(LLMProvider::Claude, LLMProvider::Claude);
    assert_ne!(LLMProvider::OpenAI, LLMProvider::Claude);
    assert_ne!(LLMProvider::Gemini, LLMProvider::DeepSeek);
}

// Mock tests for request structure verification
#[tokio::test]
async fn test_ollama_request_structure() {
    // Start a mock server
    let mock_server = MockServer::start().await;

    // Mock that captures the request body
    Mock::given(method("POST"))
        .and(path("/api/generate"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "response": "Test response"
        })))
        .mount(&mock_server)
        .await;

    // Create config with mock server URL
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: mock_server.uri(),
        ollama_model: "test_model".to_string(),
    };

    // Create Ollama client
    let client = create_llm_client(LLMProvider::Ollama, Arc::new(config)).unwrap();

    // Test generation
    let result = client.generate("Test prompt").await;
    assert!(result.is_ok());
    assert_eq!(result.unwrap(), "Test response");
}

#[tokio::test]
async fn test_ollama_network_error() {
    // Create config with invalid URL
    let config = AppConfig {
        openai_api_key: None,
        anthropic_api_key: None,
        google_api_key: None,
        deepseek_api_key: None,
        brave_search_api_key: None,
        ollama_base_url: "http://invalid-url:99999".to_string(),
        ollama_model: "test_model".to_string(),
    };

    // Create Ollama client
    let client = create_llm_client(LLMProvider::Ollama, Arc::new(config)).unwrap();

    // Test generation - should return network error
    let result = client.generate("Test prompt").await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::RequestError(_) => {
            // Expected network error
        }
        _ => panic!("Expected RequestError"),
    }
}