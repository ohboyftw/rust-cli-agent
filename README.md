# 🤖 Rust CLI Coding Agent 🤖

**Author:** Aravind V
**Version:** 1.0.0

A sophisticated command-line coding assistant built with Rust. This agent leverages the power of multiple Large Language Models (LLMs) to understand goals, create plans, and execute them by writing code and using a variety of tools.

---

## ✨ Features

* **Multi-Provider LLM Support:** Seamlessly switch between different AI models using a simple command-line flag.
    * `--provider openai`
    * `--provider gemini`
    * `--provider claude`
    * `--provider deepseek`
    * `--provider ollama` (For running local models)
* **Intelligent Orchestration:** A reasoning agent creates a step-by-step plan for your goal and executes it intelligently.
* **Extensible Tool System:** The agent can interact with its environment to:
    * Read and write files (`ReadFile`, `WriteFile`).
    * Execute arbitrary shell commands (`RunCommand`).
    * Perform real-time web searches for up-to-date information (`Search`).
    * List directory contents to understand project structure (`ListFiles`).
* **Context-Aware Operation:** Maintains a history of actions and results to make informed decisions and self-correct.
* **Asynchronous & Performant:** Built on `tokio` for efficient, non-blocking operations.
* **Secure Configuration:** Manages API keys and other secrets via a `.env` file, keeping them out of the source code.

---

## 🚀 Getting Started

### Prerequisites

* **Rust:** Install the Rust toolchain from [rustup.rs](https://rustup.rs/).
* **API Keys:** You'll need API keys for the LLM services you intend to use.

### Installation & Setup

1.  **Clone the Repository:**
    ```bash
    git clone <repository-url>
    cd rust-cli-coding-agent
    ```

2.  **Configure Environment Variables:**
    * Copy the example `.env` file:
        ```bash
        cp .env.example .env
        ```
    * Open the `.env` file and add your API keys. Only the keys for the providers you use are required.
        ```dotenv
        # For OpenAI
        OPENAI_API_KEY="your-openai-api-key"

        # For Google Gemini
        GOOGLE_API_KEY="your-google-api-key"

        # For Anthropic Claude
        ANTHROPIC_API_KEY="your-anthropic-api-key"

        # For DeepSeek
        DEEPSEEK_API_KEY="your-deepseek-api-key"

        # For the Search Tool (using Brave Search API)
        BRAVE_SEARCH_API_KEY="your-brave-search-api-key"

        # For Ollama (if using a custom base URL)
        OLLAMA_BASE_URL="http://localhost:11434"
        ```

3.  **Build the Project:**
    ```bash
    cargo build --release
    ```
    The executable will be located at `target/release/cli_coding_agent`.

---

## USAGE

Run the agent from the command line, providing a high-level goal.

### Basic Syntax

```bash
cargo run -- "<your goal here>"
```

### Examples

**Using the default provider (OpenAI):**
```bash
cargo run -- "Create a Rust function that calculates the factorial of a number and write it to a file named `factorial.rs`."
```

**Switching to Google Gemini:**
```bash
cargo run -- --provider gemini "Implement a simple web server using the Axum framework that returns 'Hello, World!' at the root URL."
```

**Using a local model with Ollama:**
```bash
# Make sure your Ollama server is running with a model like 'llama3'
cargo run -- --provider ollama "Read the `Cargo.toml` file and explain its dependencies."
```

**Using the Search Tool:**
```bash
cargo run -- "Search for the latest version of the `tokio` crate and create a new Rust project that uses it."
```

---

## 🏛️ Architecture Overview

* `main.rs`: Entry point, CLI parsing.
* `orchestrator.rs`: The core reasoning engine that manages the plan and state.
* `llm/`: Module containing all LLM client implementations, unified under the `LLMClient` trait.
* `agents/`: Contains specialized agents (`PlannerAgent`, `CoderAgent`) responsible for specific tasks.
* `tools/`: Defines and implements the tools the agent can use.
* `state.rs`: Manages the application state, including history and context.
* `config.rs`: Handles loading configuration from the `.env` file.
* `error.rs`: Custom error types for robust error handling.
