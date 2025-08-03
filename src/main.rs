use anyhow::Result;
use clap::Parser;
use colored::*;

use log::{info, error};
use std::io::{self, Write};
use std::sync::Arc;

mod agents;
mod config;
mod cost_tracker;
mod error;
mod llm;
mod orchestrator;
mod state;
mod tools;

use config::AppConfig;
use llm::{create_llm_client, LLMProvider};
use orchestrator::Orchestrator;
use crate::cost_tracker::CostTracker;

/// A CLI Coding Agent powered by Large Language Models
#[derive(Parser, Debug)]
#[command(author, version, about, long_about = None)]
struct Cli {
    /// The LLM provider to use for generation
    #[arg(long, value_enum, default_value_t = LLMProvider::OpenAI)]
    provider: LLMProvider,
}

#[tokio::main]
async fn main() -> Result<()> {
    let current_exe = std::env::current_exe()?;                                                                                             
    let project_root = current_exe.parent().and_then(|p| p.parent()).and_then(|p| p.parent()).unwrap_or_else(|| std::path::Path::new("."));  
    let dotenv_path = project_root.join(".env");                                                                                             
    dotenvy::from_path(dotenv_path).ok();   

    env_logger::builder().filter_level(log::LevelFilter::Info).init();

    let cli = Cli::parse();
    info!("CLI arguments parsed successfully.");

    println!("{}", "===================================".cyan());
    println!("{}", "🤖 Rust CLI Coding Agent Initialized 🤖".bold().cyan());
    println!("{}", "===================================".cyan());
    println!("{} {}", "🧠 Using LLM Provider:".bold().yellow(), cli.provider);
    println!();

    println!("{}", "//>––––––––––––––––––––––––––––––––––––––––––––––––––––––––––––<\\\\".yellow().bold());
    println!();
    println!("{}", "  AUGMENTATION-ASSISTED TASK HANDLER [VER 0.0.1]".bold().cyan());
    println!("{}", "  SYSTEM STATUS:".bold().white());
    println!("{} {}", "  > AGENT CORTEX:".dimmed(), "ONLINE".green().bold());
    println!();

    // Rephrased labels to sound more like in-game UI elements.
    // "Directive" instead of "Goal", and "Neural Link" for the LLM Provider.
    // Display the provider as a string using Debug formatting
    println!("{} {}", "//: NEURAL LINK VIA:".yellow().bold(), format!("{:?}", cli.provider).white());
    println!();



    let config = Arc::new(AppConfig::load()?);
    info!("Configuration loaded.");

    loop {
        println!("{}", "//: PRIMARY DIRECTIVE:".yellow().bold());

        io::stdout().flush()?;

        let mut goal = String::new();
        io::stdin().read_line(&mut goal)?;
        let goal = goal.trim();

        if goal.eq_ignore_ascii_case("quit") || goal.eq_ignore_ascii_case("exit") {
            println!("{}", "Exiting agent. Goodbye!".bold().cyan());
            break;
        }

        if goal.is_empty() {
            println!("{}", "Goal cannot be empty. Please enter a valid goal.".red());
            continue;
        }

        // Deus Ex Inspired: "Objective" and gold/blue color scheme
        println!(
            "{} {}",
            "🗝️ OBJECTIVE:".bold().truecolor(212, 175, 55), // gold
            goal.truecolor(51, 153, 255) // blue
        );
        
        let llm_client = create_llm_client(cli.provider, config.clone())?;
        info!("LLM client created for provider: {}", cli.provider);
        
        let reasoning_client = create_llm_client(LLMProvider::OpenAI, config.clone())?;
        info!("Reasoning client (OpenAI GPT-4o) created for planning and tool decisions.");

        // Display cost information (Phase 1.2)
        println!("{} {}{}", "💰 Current Session Cost:".bold().green(), "$".bold().green(), 0.00); // Placeholder for now

        let cost_tracker = Arc::new(CostTracker::new());
        let mut orchestrator = Orchestrator::new(goal.to_string(), llm_client, reasoning_client, cost_tracker.clone());
        info!("Orchestrator initialized.");

        // Display cost information (Phase 1.2)
        println!("{} {}{:.4}", "💰 Current Session Cost:".bold().green(), "$".bold().green(), cost_tracker.get_total_cost());

        match orchestrator.run().await {
            Ok(_) => println!("{}", "✅ Task Completed Successfully!".bold().green()),
            Err(e) => {
                error!("Orchestrator failed: {:?}", e);
                println!("{} {}", "❌ Task Failed:".bold().red(), e);
            }
        }
        println!("{}", "===================================".cyan());
    }

    Ok(())
}
