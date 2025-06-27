use anyhow::Result;
use clap::Parser;
use colored::*;
use dotenvy::dotenv;
use log::info;
use std::sync::Arc;

mod agents;
mod config;
mod error;
mod llm;
mod orchestrator;
mod state;
mod tools;

use config::AppConfig;
use llm::{create_llm_client, LLMProvider};
use orchestrator::Orchestrator;

/// A CLI Coding Agent powered by Large Language Models
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The main goal or task for the coding agent
    #[arg(index = 1)]
    goal: String,

    /// The LLM provider to use for generation
    #[arg(long, value_enum, default_value_t = LLMProvider::OpenAI)]
    provider: LLMProvider,
}

#[tokio::main]
async fn main() -> Result<()> {
    dotenv().ok();
    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let cli = Cli::parse();
    info!("CLI arguments parsed successfully.");


    println!("{}", "//>––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––<\\\\".yellow().bold());
    println!();
    println!("{}", "  AUGMENTATION-ASSISTED TASK HANDLER [VER 0.0.1]".bold().cyan());
    println!("{}", "  SYSTEM STATUS:".bold().white());
    println!("{} {}", "  > AGENT CORTEX:".dimmed(), "ONLINE".green().bold());
    println!();

    // Rephrased labels to sound more like in-game UI elements.
    // "Directive" instead of "Goal", and "Neural Link" for the LLM Provider.
    println!("{} {}", "//: PRIMARY DIRECTIVE:".yellow().bold(), cli.goal.white());
    // Display the provider as a string using Debug formatting
    println!("{} {}", "//: NEURAL LINK VIA:".yellow().bold(), format!("{:?}", cli.provider).white());
    println!();

    // A matching footer to close the block.
    println!("{}", "\\\\>––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––<//".yellow().bold());

    let config = Arc::new(AppConfig::load()?);
    info!("Configuration loaded.");

    let llm_client = create_llm_client(cli.provider, config.clone())?;
    info!("LLM client created for provider: {}", cli.provider);
    
    let reasoning_client = create_llm_client(LLMProvider::OpenAI, config.clone())?;
    info!("Reasoning client (OpenAI GPT-4o) created for planning and tool decisions.");


    let mut orchestrator = Orchestrator::new(cli.goal, llm_client, reasoning_client);
    info!("Orchestrator initialized.");

    orchestrator.run().await?;

    println!("\n{}", "✅ Task Completed Successfully!".bold().green());
    Ok(())
}
