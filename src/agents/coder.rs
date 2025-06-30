use std::sync::Arc;
use anyhow::Result;
use log::info;

use crate::{error::AgentError, llm::LLMClient};

pub struct CoderAgent {
    llm_client: Arc<dyn LLMClient>,
}

impl CoderAgent {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    pub async fn generate_code(&self, task_description: &str, context: &str) -> Result<String, AgentError> {
        let prompt = self.build_prompt(task_description, context);
        info!("Coder prompt:\n{}", prompt);
        let response = self.llm_client.generate(&prompt).await?;
        info!("Coder response:\n{}", response);
        Ok(self.parse_code(&response))
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
    }

    #[async_trait]
    impl LLMClient for MockLLMClient {
        async fn generate(&self, _prompt: &str) -> Result<String, AgentError> {
            Ok(self.response.clone())
        }
    }

    #[tokio::test]
    async fn test_generate_code_success() {
        let mock_code = "def hello_world():\n    print('Hello, World!')";
        let mock_client = Arc::new(MockLLMClient {
            response: mock_code.to_string(),
        });
        
        let coder = CoderAgent::new(mock_client);
        let result = coder.generate_code("Create a hello world function", "Python project").await;
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, mock_code);
    }

    #[test]
    fn test_build_prompt() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
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
        });
        let coder = CoderAgent::new(mock_client);
        
        let response = "print('Hello, World!')";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "print('Hello, World!')");
    }

    #[test]
    fn test_parse_code_with_whitespace() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
        let response = "  \n  print('Hello, World!')  \n  ";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "print('Hello, World!')");
    }

    #[test]
    fn test_parse_code_multiline() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
        let response = "def hello():\n    print('Hello')\n    return 'World'";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "def hello():\n    print('Hello')\n    return 'World'");
    }

    #[test]
    fn test_parse_code_empty() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
        let response = "";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "");
    }

    #[test]
    fn test_parse_code_whitespace_only() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
        let response = "   \n  \t  \n   ";
        let code = coder.parse_code(response);
        
        assert_eq!(code, "");
    }

    #[tokio::test]
    async fn test_generate_code_with_rust_code() {
        let mock_code = "fn main() {\n    println!(\"Hello, World!\");\n}";
        let mock_client = Arc::new(MockLLMClient {
            response: format!("  {}  ", mock_code),
        });
        
        let coder = CoderAgent::new(mock_client);
        let result = coder.generate_code("Create a Rust hello world", "Rust project").await;
        
        assert!(result.is_ok());
        let code = result.unwrap();
        assert_eq!(code, mock_code);
    }

    #[test]
    fn test_build_prompt_contains_required_elements() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let coder = CoderAgent::new(mock_client);
        
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
