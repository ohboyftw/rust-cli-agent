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

After building, the agent runs in an interactive mode, prompting you for goals.

### Running the Agent

Navigate to the project root and run:

```bash
cargo run --
```

Or, if you've added the executable to your PATH (see below):

```bash
cli_coding_agent
```

Once running, you will be prompted to enter your goal:

```
Enter your goal (or 'quit' to exit): Create a Rust function that calculates the factorial of a number and write it to a file named `factorial.rs`.
```

### Using Different LLM Providers

You can specify the LLM provider when starting the agent:

```bash
cargo run -- --provider gemini
```

Or, if using the direct executable:

```bash
cli_coding_agent --provider ollama
```

### Making the Agent Globally Accessible (Optional)

To run `cli_coding_agent` from any directory without specifying its full path, you can add its executable to your system's PATH or create a symbolic link.

**For Windows:**

1.  Open **Command Prompt as Administrator**.
2.  Navigate to the project root: `cd rust-cli-coding-agent`
3.  Run the provided script: `install_path_windows.bat`
4.  **Important:** Restart your command prompt or PowerShell for changes to take effect.

**For Linux/macOS:**

1.  Open your terminal.
2.  Navigate to the project root: `cd rust-cli-coding-agent`
3.  Make the script executable: `chmod +x install_path_linux.sh`
4.  Run the script with `sudo`: `sudo ./install_path_linux.sh`

This will allow you to simply type `cli_coding_agent` in any directory to start the interactive agent.

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
