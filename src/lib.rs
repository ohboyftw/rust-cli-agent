//! rust-cli-agent - A sophisticated CLI coding assistant
//!
//! This crate provides a multi-provider AI coding assistant that can understand goals,
//! create plans, and execute them using various tools and LLM providers.

pub mod agents;
pub mod config;
pub mod error;
pub mod llm;
pub mod orchestrator;
pub mod state;
pub mod tools;

// Re-export commonly used types for easier access in tests and external usage
pub use config::AppConfig;
pub use error::AgentError;
pub use llm::{create_llm_client, LLMClient, LLMProvider};
pub use orchestrator::Orchestrator;
pub use state::AppState;
pub use tools::{run_tool, Tool, ToolResult, Decision, get_decision_prompt};