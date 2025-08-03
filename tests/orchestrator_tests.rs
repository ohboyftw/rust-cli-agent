use cli_coding_agent::{
    error::AgentError,
    llm::{LLMClient, AIResponse, ModelInfo},
    orchestrator::Orchestrator,
    state::AppState,
    tools::{Tool, Decision},
};
use async_trait::async_trait;
use std::sync::{Arc, Mutex};
use tokio_test;

// Mock LLM client for testing
#[derive(Clone)]
struct MockLLMClient {
    responses: Arc<Mutex<Vec<String>>>,
    call_count: Arc<Mutex<usize>>,
}

impl MockLLMClient {
    fn new(responses: Vec<String>) -> Self {
        Self {
            responses: Arc::new(Mutex::new(responses)),
            call_count: Arc::new(Mutex::new(0)),
        }
    }

    fn get_call_count(&self) -> usize {
        *self.call_count.lock().unwrap()
    }
}

#[async_trait]
impl LLMClient for MockLLMClient {
    async fn generate(&self, _prompt: &str) -> Result<AIResponse, AgentError> {
        let mut count = self.call_count.lock().unwrap();
        let responses = self.responses.lock().unwrap();
        
        if *count < responses.len() {
            let response = responses[*count].clone();
            *count += 1;
            Ok(AIResponse {
                content: response,
                input_tokens: 100,
                output_tokens: 50,
                cost: 0.001,
                model: "mock-model".to_string(),
                provider: "Mock".to_string(),
            })
        } else {
            Err(AgentError::LLMError("No more mock responses".to_string()))
        }
    }

    async fn get_model_info(&self) -> ModelInfo {
        ModelInfo {
            name: "mock-model".to_string(),
            input_cost_per_token: 0.00001,
            output_cost_per_token: 0.00002,
        }
    }

    fn calculate_cost(&self, input_tokens: u32, output_tokens: u32) -> f64 {
        (input_tokens as f64 * 0.00001) + (output_tokens as f64 * 0.00002)
    }
}

#[tokio::test]
async fn test_orchestrator_creation() {
    let mock_client = Arc::new(MockLLMClient::new(vec![]));
    let reasoning_client = mock_client.clone();
    
    let orchestrator = Orchestrator::new(
        "Test goal".to_string(),
        mock_client,
        reasoning_client,
    );

    // Orchestrator should be created successfully
    // Note: We can't directly test internal state since fields are private
    // This test mainly ensures the constructor works
}

#[tokio::test]
async fn test_orchestrator_run_basic_flow() {
    // Mock responses for planner, decision making, and coder
    let mock_responses = vec![
        // Planner response
        "1. Read existing files\n2. Write new code\n3. Test the code".to_string(),
        // Decision for step 1
        r#"{"thought": "I need to list files", "tool_name": "ListFiles", "parameters": {"path": "."}}"#.to_string(),
        // Decision for step 2  
        r#"{"thought": "I need to generate code", "tool_name": "CodeGeneration", "parameters": {"task": "Write new code"}, "file_path": "output.py"}"#.to_string(),
        // Coder response for step 2
        "print('Hello, World!')".to_string(),
        // Decision for step 3
        r#"{"thought": "I need to run tests", "tool_name": "RunCommand", "parameters": {"command": "python -m pytest"}}"#.to_string(),
    ];

    let mock_client = Arc::new(MockLLMClient::new(mock_responses));
    let reasoning_client = mock_client.clone();
    
    let mut orchestrator = Orchestrator::new(
        "Create a hello world program".to_string(),
        mock_client.clone(),
        reasoning_client,
    );

    // Note: This test would require modifications to Orchestrator to make it more testable
    // For example, dependency injection for the file system operations
    // Currently, the run() method interacts with the real file system
    
    // The test demonstrates the structure but would need the Orchestrator
    // to be refactored for better testability
}

#[test]
fn test_app_state_integration() {
    let mut state = AppState::new("Test goal".to_string());
    
    // Simulate orchestrator workflow
    state.plan = vec![
        "Read files".to_string(),
        "Write code".to_string(),
        "Run tests".to_string(),
    ];
    
    // Simulate adding history entries
    state.add_history("Directory Listing", "file1.txt\nfile2.txt");
    state.add_history("Generated Code", "print('Hello')");
    state.add_history("Test Results", "All tests passed");
    
    // Verify state
    assert_eq!(state.plan.len(), 3);
    assert_eq!(state.history.len(), 3);
    
    let context = state.get_context();
    assert!(context.contains("Test goal"));
    assert!(context.contains("Directory Listing"));
    assert!(context.contains("Generated Code"));
    assert!(context.contains("Test Results"));
}

#[test]
fn test_decision_parsing() {
    // Test valid decision JSON
    let json = r#"{
        "thought": "I need to read a file to understand the codebase",
        "tool_name": "ReadFile",
        "parameters": {
            "path": "src/main.rs"
        },
        "file_path": null
    }"#;
    
    let decision: Result<Decision, _> = serde_json::from_str(json);
    assert!(decision.is_ok());
    
    let decision = decision.unwrap();
    assert_eq!(decision.thought, "I need to read a file to understand the codebase");
    assert_eq!(decision.file_path, None);
    
    match decision.tool {
        Tool::ReadFile { path } => {
            assert_eq!(path, "src/main.rs");
        }
        _ => panic!("Expected ReadFile tool"),
    }
}

#[test]
fn test_decision_parsing_with_file_path() {
    let json = r#"{
        "thought": "I will generate code and save it",
        "tool_name": "CodeGeneration", 
        "parameters": {
            "task": "Create a function"
        },
        "file_path": "output.py"
    }"#;
    
    let decision: Result<Decision, _> = serde_json::from_str(json);
    assert!(decision.is_ok());
    
    let decision = decision.unwrap();
    assert_eq!(decision.thought, "I will generate code and save it");
    assert_eq!(decision.file_path, Some("output.py".to_string()));
    
    match decision.tool {
        Tool::CodeGeneration { task } => {
            assert_eq!(task, "Create a function");
        }
        _ => panic!("Expected CodeGeneration tool"),
    }
}

#[test]
fn test_decision_parsing_invalid_json() {
    let invalid_json = r#"{"thought": "incomplete json"#;
    
    let decision: Result<Decision, _> = serde_json::from_str(invalid_json);
    assert!(decision.is_err());
}

#[test]
fn test_decision_parsing_missing_fields() {
    let json = r#"{"thought": "Missing tool"}"#;
    
    let decision: Result<Decision, _> = serde_json::from_str(json);
    assert!(decision.is_err());
}

#[test]
fn test_decision_parsing_invalid_tool() {
    let json = r#"{
        "thought": "Test",
        "tool_name": "InvalidTool",
        "parameters": {}
    }"#;
    
    let decision: Result<Decision, _> = serde_json::from_str(json);
    assert!(decision.is_err());
}

// Integration test for the orchestrator flow components
#[tokio::test]
async fn test_orchestrator_components_integration() {
    use cli_coding_agent::agents::{planner::PlannerAgent, coder::CoderAgent};
    
    // Test planner agent
    let mock_planner_response = "1. Analyze requirements\n2. Design solution\n3. Implement code";
    let planner_client = Arc::new(MockLLMClient::new(vec![mock_planner_response.to_string()]));
    let planner = PlannerAgent::new(planner_client.clone());
    
    let plan = planner.create_plan("Create a calculator", "No existing files").await;
    assert!(plan.is_ok());
    
    let plan = plan.unwrap();
    assert_eq!(plan.len(), 3);
    assert_eq!(plan[0], "Analyze requirements");
    assert_eq!(plan[1], "Design solution");
    assert_eq!(plan[2], "Implement code");
    
    // Test coder agent
    let mock_coder_response = "def add(a, b):\n    return a + b";
    let coder_client = Arc::new(MockLLMClient::new(vec![mock_coder_response.to_string()]));
    let coder = CoderAgent::new(coder_client.clone());
    
    let code = coder.generate_code("Create an add function", "Python project").await;
    assert!(code.is_ok());
    
    let code = code.unwrap();
    assert_eq!(code, "def add(a, b):\n    return a + b");
    
    // Verify LLM clients were called
    assert_eq!(planner_client.get_call_count(), 1);
    assert_eq!(coder_client.get_call_count(), 1);
}

#[test]
fn test_orchestrator_error_scenarios() {
    // Test creating orchestrator with different client configurations
    let empty_client = Arc::new(MockLLMClient::new(vec![]));
    let error_client = Arc::new(MockLLMClient::new(vec![])); // Will return error on first call
    
    // Test creation with different client combinations
    let _orchestrator1 = Orchestrator::new(
        "Test".to_string(),
        empty_client.clone(),
        empty_client.clone(),
    );
    
    let _orchestrator2 = Orchestrator::new(
        "Test".to_string(),
        empty_client.clone(),
        error_client.clone(),
    );
    
    // Orchestrators should be created successfully regardless of client state
    // Errors would occur during execution, not creation
}

#[tokio::test]
async fn test_mock_llm_client_behavior() {
    let responses = vec![
        "First response".to_string(),
        "Second response".to_string(),
    ];
    
    let client = MockLLMClient::new(responses);
    
    // First call
    let result1 = client.generate("prompt1").await;
    assert!(result1.is_ok());
    assert_eq!(result1.unwrap(), "First response");
    assert_eq!(client.get_call_count(), 1);
    
    // Second call
    let result2 = client.generate("prompt2").await;
    assert!(result2.is_ok());
    assert_eq!(result2.unwrap(), "Second response");
    assert_eq!(client.get_call_count(), 2);
    
    // Third call should fail (no more responses)
    let result3 = client.generate("prompt3").await;
    assert!(result3.is_err());
    match result3.unwrap_err() {
        AgentError::LLMError(msg) => {
            assert_eq!(msg, "No more mock responses");
        }
        _ => panic!("Expected LLMError"),
    }
    assert_eq!(client.get_call_count(), 2); // Count shouldn't increment on error
}

#[test]
fn test_mock_llm_client_clone() {
    let client = MockLLMClient::new(vec!["Test".to_string()]);
    let cloned = client.clone();
    
    // Both should share the same state
    assert_eq!(client.get_call_count(), 0);
    assert_eq!(cloned.get_call_count(), 0);
}

// Test helper functions for orchestrator workflow simulation
fn simulate_orchestrator_state() -> AppState {
    let mut state = AppState::new("Create a web scraper".to_string());
    
    state.plan = vec![
        "Install required dependencies".to_string(),
        "Create main scraper function".to_string(),
        "Add error handling".to_string(),
        "Write tests".to_string(),
    ];
    
    state.add_history("Dependencies", "requests, beautifulsoup4");
    state.add_history("Code", "def scrape_url(url): ...");
    state.current_step = 2;
    
    state
}

#[test]
fn test_orchestrator_state_simulation() {
    let state = simulate_orchestrator_state();
    
    assert_eq!(state.goal, "Create a web scraper");
    assert_eq!(state.plan.len(), 4);
    assert_eq!(state.history.len(), 2);
    assert_eq!(state.current_step, 2);
    
    let context = state.get_context();
    assert!(context.contains("web scraper"));
    assert!(context.contains("Dependencies"));
    assert!(context.contains("requests, beautifulsoup4"));
    assert!(context.contains("Code"));
}