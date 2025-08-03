use anyhow::Result;
use serde::{Deserialize, Serialize};
use log::info;
use walkdir::WalkDir;
use crate::config::AppConfig;
use crate::error::AgentError;

#[derive(Debug, Serialize, Deserialize, Clone)]
#[serde(tag = "tool_name", content = "parameters")]
pub enum Tool {
    ReadFile { path: String },
    WriteFile { path: String, content: String },
    RunCommand { command: String },
    Search { query: String },
    ListFiles { path: String },
    CodeGeneration { task: String },
}

#[derive(Debug, Serialize, Deserialize, Clone)]
pub struct Decision {
    pub thought: String,
    #[serde(flatten)]
    pub tool: Tool,
    #[serde(default)]
    pub file_path: Option<String>,
}

#[derive(Debug)]
pub enum ToolResult {
    Success(String),
}

pub async fn run_tool(tool: Tool) -> Result<ToolResult, AgentError> {
    match tool {
        Tool::ReadFile { path } => {
            let content = tokio::fs::read_to_string(path).await?;
            Ok(ToolResult::Success(content))
        }
        Tool::WriteFile { path, content } => {
            tokio::fs::write(path, content).await?;
            Ok(ToolResult::Success("File written successfully.".to_string()))
        }
        Tool::RunCommand { command } => {
            let output = tokio::process::Command::new("sh").arg("-c").arg(command).output().await?;
            let result = if output.status.success() {
                String::from_utf8_lossy(&output.stdout).to_string()
            } else {
                format!("STDOUT:\n{}\nSTDERR:\n{}", 
                    String::from_utf8_lossy(&output.stdout),
                    String::from_utf8_lossy(&output.stderr)
                )
            };
            Ok(ToolResult::Success(result))
        }
        Tool::Search { query } => {
            info!("Performing web search for: {}", query);
            let config = AppConfig::load()?;
            let api_key = config.brave_search_api_key.ok_or_else(|| AgentError::ApiKeyMissing("Brave Search".to_string()))?;
            let client = reqwest::Client::new();
            let url = format!("https://api.search.brave.com/res/v1/web/search?q={}", query);
            let response = client.get(url).header("X-Subscription-Token", api_key).send().await?;
            
            if !response.status().is_success() {
                return Err(AgentError::ToolError(format!("Brave Search API Error: {}", response.text().await?)));
            }
            
            #[derive(Deserialize)]
            struct BraveResponse { web: Web }
            #[derive(Deserialize)]
            struct Web { results: Vec<BraveResult> }
            #[derive(Deserialize)]
            struct BraveResult { title: String, url: String, description: String }

            let body: BraveResponse = response.json().await?;
            let mut result_string = String::new();
            for (i, res) in body.web.results.into_iter().take(3).enumerate() {
                result_string.push_str(&format!("[Result {}]\nTitle: {}\nURL: {}\nSnippet: {}\n\n", i+1, res.title, res.url, res.description));
            }
            Ok(ToolResult::Success(result_string))
        }
        Tool::ListFiles { path } => {
            let mut files = String::new();
            for entry in WalkDir::new(path).into_iter().filter_map(|e| e.ok()) {
                let path = entry.path().display().to_string();
                if !path.contains("target/") && !path.contains(".git/") {
                     files.push_str(&path);
                     files.push('\n');
                }
            }
            Ok(ToolResult::Success(files))
        },
        Tool::CodeGeneration {..} => {
            Err(AgentError::ToolError("CodeGeneration is not a runnable tool.".to_string()))
        }
    }
}

pub fn get_decision_prompt(step: &str, context: &str) -> String {
    format!(r#"
You are the reasoning engine for a CLI agent. Your job is to decide which tool to use to accomplish the current step of a plan.
You must respond in a specific JSON format.

--- CONTEXT ---
{context}
--- END CONTEXT ---

--- CURRENT STEP ---
{step}
--- END CURRENT STEP ---

Based on the context and the current step, which tool should be used?
Here are the available tools:
1. `ReadFile {{ "path": "path/to/file.ext" }}`: Use when you need to examine the contents of an existing file.
2. `WriteFile {{ "path": "path/to/save.ext", "content": "The content to write" }}`: Use when saving content. For code, use CodeGeneration instead.
3. `RunCommand {{ "command": "e.g., cargo test" }}`: Use for executing shell commands, like running tests, building code, or installing dependencies.
4. `Search {{ "query": "Your search query" }}`: Use when you need up-to-date information or to research a library/API.
5. `ListFiles {{ "path": "." }}`: Use to see the layout of the current directory.
6. `CodeGeneration {{ "task": "A clear, specific instruction for the coder agent" }}`: Use this when the step explicitly requires writing code. The `task` should be a detailed prompt for another AI that will *only* write the code.

--- RESPONSE FORMAT ---
You MUST respond with a single JSON object matching this structure:
{{
  "thought": "Your reasoning for choosing this tool. Explain why this tool is the best choice for the current step.",
  "tool_name": "Name of the chosen tool (e.g., 'ReadFile')",
  "parameters": {{
    // parameters for the chosen tool, e.g., "path": "..."
  }},
  "file_path": "path/to/save.ext" // ONLY for CodeGeneration, specify where the generated code should be saved. Otherwise, omit this field.
}}

Now, make your decision for the current step.
"#)
}
