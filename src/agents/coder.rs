use std::sync::Arc;
use anyhow::Result;
use log::info;

use crate::{error::AgentError, llm::{LLMClient, AIResponse, ModelInfo}, cost_tracker::CostTracker};

pub struct CoderAgent {
    llm_client: Arc<dyn LLMClient>,
    cost_tracker: Arc<CostTracker>,
}

impl CoderAgent {
    pub fn new(llm_client: Arc<dyn LLMClient>, cost_tracker: Arc<CostTracker>) -> Self {
        Self { llm_client, cost_tracker }
    }

    pub async fn generate_code(&self, task_description: &str, context: &str) -> Result<String, AgentError> {
        let prompt = self.build_prompt(task_description, context);
        info!("Coder prompt:\n{}", prompt);
        let response = self.llm_client.generate(&prompt).await?;
        self.cost_tracker.add_cost(response.cost);
        info!("Coder response:\n{}", response.content);
        Ok(self.parse_code(&response.content))
    }

    fn build_prompt(&self, task_description: &str, context: &str) -> String {
        format!(r#"
You are an expert programmer. Your sole responsibility is to write clean, efficient, and correct code.
You will be given the overall context of the project and a specific task to complete.

--- Context ---
{context}
--- End Context ---

Your current task is: "{task_description}"

Based on the context and the task, write the necessary code. By default, you should write python code, but if the task requires a different language, use that language instead.
IMPORTANT: Output ONLY the raw code. Do not include any explanations, comments about the code, or markdown code fences like ```rust.
"#)
    }

    fn parse_code(&self, response: &str) -> String {
        response.trim().to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use async_trait::async_trait;
    use std::sync::Arc;

    // Mock LLM client for testing
    struct MockLLMClient {
        response: String,
        cost: f64,
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate(&self, _prompt: &str) -> Result<AIResponse, AgentError> {
            Ok(AIResponse {
                content: self.response.clone(),
                input_tokens: 10,
                output_tokens: 20,
                cost: self.cost,
                model: "mock-model".to_string(),
                provider: "mock-provider".to_string(),
            })
        }
        async fn generate_json(&self, _prompt: &str) -> Result<AIResponse, AgentError> {
            self.generate(_prompt).await
        }
        async fn get_model_info(&self) -> ModelInfo {
            ModelInfo {
                name: "mock-model".to_string(),
                input_cost_per_token: 0.0,
                output_cost_per_token: 0.0,
            }
        }
        fn calculate_cost(&self, _input_tokens: u32, _output_tokens: u32) -> f64 {
            0.0
        }
    }

    #[tokio::test]
    async fn test_generate_code_success() {
        let mock_code = "def hello_world():\n    print('Hello, World!')";
        let mock_client = Arc::new(MockLLMClient {
            response: mock_code.to_string(),
            cost: 0.001,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        
        let coder = CoderAgent::new(mock_client, cost_tracker.clone());
        let result = coder.generate_code("Create a hello world function", "Python project").await;
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, mock_code);
        assert_eq!(cost_tracker.get_total_cost(), 0.001);
    }

    #[test]
    fn test_build_prompt() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let prompt = coder.build_prompt("Create a function", "Test context");
        
        assert!(prompt.contains("Create a function"));
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("expert programmer"));
        assert!(prompt.contains("ONLY the raw code"));
    }

    #[test]
    fn test_parse_code_simple() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let response = "print('Hello, World!')";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "print('Hello, World!')");
    }

    #[test]
    fn test_parse_code_with_whitespace() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let response = "  \n  print('Hello, World!')  \n  ";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "print('Hello, World!')");
    }

    #[test]
    fn test_parse_code_multiline() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let response = "def hello():\n    print('Hello')\n    return 'World'";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "def hello():\n    print('Hello')\n    return 'World'");
    }

    #[test]
    fn test_parse_code_empty() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let response = "";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "");
    }

    #[test]
    fn test_parse_code_whitespace_only() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let response = "   \n  \t  \n   ";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "");
    }

    #[tokio::test]
    async fn test_generate_code_with_rust_code() {
        let mock_code = "fn main() {\n    println!(\"Hello, World!\");\n}";
        let mock_client = Arc::new(MockLLMClient {
            response: format!("  {}  ", mock_code),
            cost: 0.002,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        
        let coder = CoderAgent::new(mock_client, cost_tracker.clone());
        let result = coder.generate_code("Create a Rust hello world", "Rust project").await;
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, mock_code);
        assert_eq!(cost_tracker.get_total_cost(), 0.002);
    }

    #[test]
    fn test_build_prompt_contains_required_elements() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
            cost: 0.0,
        });
        let cost_tracker = Arc::new(CostTracker::new());
        let coder = CoderAgent::new(mock_client, cost_tracker);
        
        let task = "Write a sorting algorithm";
        let context = "This is a data structures project";
        let prompt = coder.build_prompt(task, context);
        
        // Check all required elements are present
        assert!(prompt.contains(task));
        assert!(prompt.contains(context));
        assert!(prompt.contains("expert programmer"));
        assert!(prompt.contains("clean, efficient, and correct code"));
        assert!(prompt.contains("python code"));
        assert!(prompt.contains("different language"));
        assert!(prompt.contains("ONLY the raw code"));
        assert!(prompt.contains("markdown code fences"));
    }
}
