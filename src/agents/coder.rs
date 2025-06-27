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
