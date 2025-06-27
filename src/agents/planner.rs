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
