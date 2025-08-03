use std::sync::Arc;
use anyhow::Result;
use colored::*;
use log::{info, warn};

use crate::{
    agents::{coder::CoderAgent, planner::PlannerAgent},
    error::AgentError,
    llm::LLMClient,
    state::AppState,
    tools::{self, Tool, ToolResult, Decision},
    cost_tracker::CostTracker,
};

pub struct Orchestrator {
    state: AppState,
    llm_client: Arc<dyn LLMClient>,
    reasoning_client: Arc<dyn LLMClient>,
    cost_tracker: Arc<CostTracker>,
}

impl Orchestrator {
    pub fn new(goal: String, llm_client: Arc<dyn LLMClient>, reasoning_client: Arc<dyn LLMClient>, cost_tracker: Arc<CostTracker>) -> Self {
        Self {
            state: AppState::new(goal),
            llm_client,
            reasoning_client,
            cost_tracker,
        }
    }

    pub async fn run(&mut self) -> Result<()> {
        self.gather_initial_context().await?;
        self.create_plan().await?;
        self.execute_plan().await?;
        Ok(())
    }

    async fn gather_initial_context(&mut self) -> Result<(), AgentError> {
        println!("{}", "🔍 Gathering initial context...".yellow());
        let result = tools::run_tool(Tool::ListFiles { path: ".".to_string() }).await?;
        let ToolResult::Success(output) = result;
             self.state.add_history("Initial Directory Listing", &output);
             println!("   {}", "Found existing file structure.".green());
        Ok(())
    }

    async fn create_plan(&mut self) -> Result<(), AgentError> {
        println!("{}", "🤔 Thinking... Creating a plan...".yellow());
        let planner = PlannerAgent::new(self.reasoning_client.clone(), self.cost_tracker.clone());
        let plan = planner.create_plan(&self.state.goal, &self.state.get_context()).await?;
        self.state.plan = plan;
        println!("{}", "📝 Plan Created:".bold().green());
        for (i, step) in self.state.plan.iter().enumerate() {
            println!("   {}. {}", i + 1, step);
        }
        println!();
        info!("Plan created with {} steps.", self.state.plan.len());
        Ok(())
    }

    async fn execute_plan(&mut self) -> Result<(), AgentError> {
        let coder = CoderAgent::new(self.llm_client.clone(), self.cost_tracker.clone());
        for i in 0..self.state.plan.len() {
            self.state.current_step = i;
            let step = &self.state.plan[i].clone();
            println!("{}", format!("\n▶️  Executing Step {}: {}", i + 1, step).bold().cyan());
            
            let decision = self.decide_action(step, &self.state.get_context()).await?;
            
            match decision.tool {
                Tool::CodeGeneration { task } => {
                    println!("   {} {}...", "✍️ Writing Code for:".magenta(), task);
                    let code = coder.generate_code(&task, &self.state.get_context()).await?;
                    println!("{}", "Generated Code:".bold().green());
                    println!("{}", code.trim().green());
                    self.state.add_history("Generated Code", &code);

                    if let Some(path) = decision.file_path {
                         println!("   {} '{}'...", "💾 Saving code to file".magenta(), path);
                         match tools::run_tool(Tool::WriteFile { path: path.clone(), content: code }).await {
                             Ok(_) => println!("   {} Code saved to {}", "✅ Success:".green(), path),
                             Err(e) => println!("   {} Failed to save code: {}", "❌ Error:".red(), e),
                         }
                    }
                },
                other_tool => {
                    println!("   {} {:?}...", "🛠️ Using Tool:".magenta(), other_tool);
                    let result = tools::run_tool(other_tool).await;
                    match result {
                        Ok(ToolResult::Success(output)) => {
                            let summarized = if output.len() > 300 { format!("{}...", &output[..300]) } else { output.clone() };
                            println!("   {} {}", "✅ Tool Success:".green(), summarized);
                            self.state.add_history("Tool Output", &output);
                        },
                        Err(e) => {
                             println!("   {} {}", "❌ Tool Error:".red(), e);
                             warn!("Tool execution failed for step {}: {}", i + 1, e);
                             self.state.add_history("Tool Error", &e.to_string());
                        }
                    }
                }
            }
        }
        Ok(())
    }

    async fn decide_action(&self, step: &str, context: &str) -> Result<Decision, AgentError> {
        let prompt = tools::get_decision_prompt(step, context);
        info!("Decision prompt:\n{}", prompt);
        
        let response = self.reasoning_client.generate_json(&prompt).await?;
        self.cost_tracker.add_cost(response.cost);
        info!("Decision response:\n{}", response.content);
        
        serde_json::from_str(&response.content)
            .map_err(|e| AgentError::ResponseParseError(format!("Failed to parse tool decision: {}. Response: {}", e, response.content)))
    }
}
