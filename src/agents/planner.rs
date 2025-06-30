use std::sync::Arc;
use anyhow::Result;
use log::info;

use crate::{error::AgentError, llm::LLMClient};

pub struct PlannerAgent {
    llm_client: Arc<dyn LLMClient>,
}

impl PlannerAgent {
    pub fn new(llm_client: Arc<dyn LLMClient>) -> Self {
        Self { llm_client }
    }

    pub async fn create_plan(&self, goal: &str, context: &str) -> Result<Vec<String>, AgentError> {
        let prompt = self.build_prompt(goal, context);
        info!("Planner prompt:\n{}", prompt);
        let response = self.llm_client.generate(&prompt).await?;
        info!("Planner response:\n{}", response);
        Ok(self.parse_plan(&response))
    }

    fn build_prompt(&self, goal: &str, context: &str) -> String {
        format!(r#"
You are a master planner AI. Your job is to create a detailed, step-by-step plan to accomplish a given programming goal.
The user's goal is: "{goal}"

--- CONTEXT ---
Here is the current context, including existing files and previous actions:
{context}
--- END CONTEXT ---

Break down the goal into a numbered list of simple, single-purpose steps. The plan should be logical and efficient.
A good plan often starts with information gathering (listing or reading files, searching), then implementation (writing code), and finally verification (running tests or commands).

Output ONLY the numbered list of steps, with each step on a new line. Do not include a preamble or conclusion.
"#)
    }

    fn parse_plan(&self, response: &str) -> Vec<String> {
        response
            .lines()
            .map(|line| line.trim())
            .filter(|line| !line.is_empty())
            .filter_map(|line| {
                if let Some(pos) = line.find(". ") {
                    Some(line[pos + 2..].to_string())
                } else {
                    Some(line.to_string())
                }
            })
            .collect()
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
    async fn test_create_plan_success() {
        let mock_response = "1. Read existing files\n2. Write new code\n3. Run tests";
        let mock_client = Arc::new(MockLLMClient {
            response: mock_response.to_string(),
        });
        
        let planner = PlannerAgent::new(mock_client);
        let result = planner.create_plan("Create a function", "No context").await;
        
        assert!(result.is_ok());
        let plan = result.unwrap();
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], "Read existing files");
        assert_eq!(plan[1], "Write new code");
        assert_eq!(plan[2], "Run tests");
    }

    #[test]
    fn test_build_prompt() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let prompt = planner.build_prompt("Test goal", "Test context");
        
        assert!(prompt.contains("Test goal"));
        assert!(prompt.contains("Test context"));
        assert!(prompt.contains("master planner AI"));
        assert!(prompt.contains("numbered list"));
    }

    #[test]
    fn test_parse_plan_numbered() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "1. First step\n2. Second step\n3. Third step";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], "First step");
        assert_eq!(plan[1], "Second step");
        assert_eq!(plan[2], "Third step");
    }

    #[test]
    fn test_parse_plan_unnumbered() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "First step\nSecond step\nThird step";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], "First step");
        assert_eq!(plan[1], "Second step");
        assert_eq!(plan[2], "Third step");
    }

    #[test]
    fn test_parse_plan_with_empty_lines() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "1. First step\n\n2. Second step\n   \n3. Third step\n";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], "First step");
        assert_eq!(plan[1], "Second step");
        assert_eq!(plan[2], "Third step");
    }

    #[test]
    fn test_parse_plan_mixed_format() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "1. First step\nSecond step without number\n3. Third step";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 3);
        assert_eq!(plan[0], "First step");
        assert_eq!(plan[1], "Second step without number");
        assert_eq!(plan[2], "Third step");
    }

    #[test]
    fn test_parse_plan_empty_response() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 0);
    }

    #[test]
    fn test_parse_plan_whitespace_only() {
        let mock_client = Arc::new(MockLLMClient {
            response: "".to_string(),
        });
        let planner = PlannerAgent::new(mock_client);
        
        let response = "   \n  \n\t\n  ";
        let plan = planner.parse_plan(response);
        
        assert_eq!(plan.len(), 0);
    }
}
