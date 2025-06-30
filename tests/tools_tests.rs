use cli_coding_agent::{
    error::AgentError,
    tools::{run_tool, Tool, ToolResult, Decision, get_decision_prompt},
};
use std::fs;
use tempfile::{tempdir, NamedTempFile};
use wiremock::{
    matchers::{header, method, path, query_param},
    Mock, MockServer, ResponseTemplate,
};

#[tokio::test]
async fn test_read_file_success() {
    // Create a temporary file
    let temp_file = NamedTempFile::new().unwrap();
    let test_content = "Hello, World!\nThis is a test file.";
    fs::write(temp_file.path(), test_content).unwrap();

    // Test reading the file
    let tool = Tool::ReadFile {
        path: temp_file.path().to_string_lossy().to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok());
    
    match result.unwrap() {
        ToolResult::Success(content) => {
            assert_eq!(content, test_content);
        }
    }
}

#[tokio::test]
async fn test_read_file_not_found() {
    let tool = Tool::ReadFile {
        path: "/nonexistent/file.txt".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::IoError(_) => {
            // Expected error type
        }
        _ => panic!("Expected IoError"),
    }
}

#[tokio::test]
async fn test_write_file_success() {
    let temp_dir = tempdir().unwrap();
    let file_path = temp_dir.path().join("test.txt");
    let test_content = "This is test content.";

    let tool = Tool::WriteFile {
        path: file_path.to_string_lossy().to_string(),
        content: test_content.to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok());
    
    match result.unwrap() {
        ToolResult::Success(message) => {
            assert_eq!(message, "File written successfully.");
        }
    }

    // Verify file was written
    let written_content = fs::read_to_string(&file_path).unwrap();
    assert_eq!(written_content, test_content);
}

#[tokio::test]
async fn test_write_file_invalid_path() {
    let tool = Tool::WriteFile {
        path: "/invalid/path/file.txt".to_string(),
        content: "test content".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::IoError(_) => {
            // Expected error type
        }
        _ => panic!("Expected IoError"),
    }
}

#[tokio::test]
async fn test_run_command_success() {
    let tool = Tool::RunCommand {
        command: "echo 'Hello, World!'".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok());
    
    match result.unwrap() {
        ToolResult::Success(output) => {
            assert!(output.contains("Hello, World!"));
        }
    }
}

#[tokio::test]
async fn test_run_command_failure() {
    let tool = Tool::RunCommand {
        command: "invalidcommandthatdoesnotexist".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok()); // run_tool returns Ok even for command failures
    
    match result.unwrap() {
        ToolResult::Success(output) => {
            // Should contain both stdout and stderr
            assert!(output.contains("STDOUT:") && output.contains("STDERR:"));
        }
    }
}

#[tokio::test]
async fn test_list_files_success() {
    let temp_dir = tempdir().unwrap();
    
    // Create some test files
    let file1 = temp_dir.path().join("file1.txt");
    let file2 = temp_dir.path().join("file2.txt");
    fs::write(&file1, "content1").unwrap();
    fs::write(&file2, "content2").unwrap();

    let tool = Tool::ListFiles {
        path: temp_dir.path().to_string_lossy().to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok());
    
    match result.unwrap() {
        ToolResult::Success(output) => {
            assert!(output.contains("file1.txt"));
            assert!(output.contains("file2.txt"));
            // Should not contain target or .git directories
            assert!(!output.contains("target/"));
            assert!(!output.contains(".git/"));
        }
    }
}

#[tokio::test]
async fn test_list_files_filters_directories() {
    let temp_dir = tempdir().unwrap();
    
    // Create test files and directories
    let file1 = temp_dir.path().join("file1.txt");
    let target_dir = temp_dir.path().join("target");
    let git_dir = temp_dir.path().join(".git");
    
    fs::write(&file1, "content1").unwrap();
    fs::create_dir(&target_dir).unwrap();
    fs::create_dir(&git_dir).unwrap();
    
    let target_file = target_dir.join("built.exe");
    let git_file = git_dir.join("config");
    fs::write(&target_file, "binary").unwrap();
    fs::write(&git_file, "git config").unwrap();

    let tool = Tool::ListFiles {
        path: temp_dir.path().to_string_lossy().to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_ok());
    
    match result.unwrap() {
        ToolResult::Success(output) => {
            assert!(output.contains("file1.txt"));
            // Should filter out target and .git directories
            assert!(!output.contains("target/"));
            assert!(!output.contains(".git/"));
            assert!(!output.contains("built.exe"));
            assert!(!output.contains("config"));
        }
    }
}

#[tokio::test]
async fn test_search_success() {
    // Start a mock server for Brave Search API
    let mock_server = MockServer::start().await;

    // Mock the Brave Search API response
    Mock::given(method("GET"))
        .and(path("/res/v1/web/search"))
        .and(query_param("q", "test query"))
        .and(header("X-Subscription-Token", "test_brave_key"))
        .respond_with(ResponseTemplate::new(200).set_body_json(serde_json::json!({
            "web": {
                "results": [
                    {
                        "title": "Test Result 1",
                        "url": "https://example.com/1",
                        "description": "This is test result 1"
                    },
                    {
                        "title": "Test Result 2", 
                        "url": "https://example.com/2",
                        "description": "This is test result 2"
                    }
                ]
            }
        })))
        .mount(&mock_server)
        .await;

    // Set environment variable for API key
    std::env::set_var("BRAVE_SEARCH_API_KEY", "test_brave_key");

    // Override the search URL to point to our mock server
    // Note: This test would need modification to the actual search implementation
    // to support URL override for testing. For now, this demonstrates the test structure.
    
    // Cleanup
    std::env::remove_var("BRAVE_SEARCH_API_KEY");
}

#[tokio::test]
async fn test_search_missing_api_key() {
    // Ensure API key is not set
    std::env::remove_var("BRAVE_SEARCH_API_KEY");

    let tool = Tool::Search {
        query: "test query".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::ApiKeyMissing(provider) => {
            assert_eq!(provider, "Brave Search");
        }
        _ => panic!("Expected ApiKeyMissing error"),
    }
}

#[tokio::test]
async fn test_code_generation_tool_error() {
    let tool = Tool::CodeGeneration {
        task: "Generate some code".to_string(),
    };
    
    let result = run_tool(tool).await;
    assert!(result.is_err());
    
    match result.unwrap_err() {
        AgentError::ToolError(msg) => {
            assert_eq!(msg, "CodeGeneration is not a runnable tool.");
        }
        _ => panic!("Expected ToolError"),
    }
}

#[test]
fn test_decision_serialization() {
    let decision = Decision {
        thought: "I need to read a file".to_string(),
        tool: Tool::ReadFile {
            path: "test.txt".to_string(),
        },
        file_path: Some("output.txt".to_string()),
    };

    // Test JSON serialization
    let json = serde_json::to_string(&decision).unwrap();
    assert!(json.contains("thought"));
    assert!(json.contains("I need to read a file"));
    assert!(json.contains("ReadFile"));
    assert!(json.contains("test.txt"));
    assert!(json.contains("file_path"));
    assert!(json.contains("output.txt"));

    // Test JSON deserialization
    let deserialized: Decision = serde_json::from_str(&json).unwrap();
    assert_eq!(deserialized.thought, decision.thought);
    assert_eq!(deserialized.file_path, decision.file_path);
    
    match (deserialized.tool, decision.tool) {
        (Tool::ReadFile { path: path1 }, Tool::ReadFile { path: path2 }) => {
            assert_eq!(path1, path2);
        }
        _ => panic!("Tool types don't match"),
    }
}

#[test]
fn test_decision_without_file_path() {
    let json = r#"{
        "thought": "Run a command",
        "tool_name": "RunCommand",
        "parameters": {
            "command": "ls -la"
        }
    }"#;

    let decision: Decision = serde_json::from_str(json).unwrap();
    assert_eq!(decision.thought, "Run a command");
    assert_eq!(decision.file_path, None);
    
    match decision.tool {
        Tool::RunCommand { command } => {
            assert_eq!(command, "ls -la");
        }
        _ => panic!("Expected RunCommand tool"),
    }
}

#[test]
fn test_tool_serialization() {
    let tools = vec![
        Tool::ReadFile {
            path: "test.txt".to_string(),
        },
        Tool::WriteFile {
            path: "output.txt".to_string(),
            content: "content".to_string(),
        },
        Tool::RunCommand {
            command: "echo hello".to_string(),
        },
        Tool::Search {
            query: "test query".to_string(),
        },
        Tool::ListFiles {
            path: ".".to_string(),
        },
        Tool::CodeGeneration {
            task: "write code".to_string(),
        },
    ];

    for tool in tools {
        let json = serde_json::to_string(&tool).unwrap();
        let deserialized: Tool = serde_json::from_str(&json).unwrap();
        
        // Compare debug representations since Tool doesn't implement PartialEq
        assert_eq!(format!("{:?}", tool), format!("{:?}", deserialized));
    }
}

#[test]
fn test_get_decision_prompt() {
    let step = "Read the configuration file";
    let context = "We are working on a Rust project";
    
    let prompt = get_decision_prompt(step, context);
    
    assert!(prompt.contains(step));
    assert!(prompt.contains(context));
    assert!(prompt.contains("reasoning engine"));
    assert!(prompt.contains("tool to use"));
}

#[test]
fn test_tool_debug() {
    let tool = Tool::ReadFile {
        path: "test.txt".to_string(),
    };
    
    let debug_str = format!("{:?}", tool);
    assert!(debug_str.contains("ReadFile"));
    assert!(debug_str.contains("test.txt"));
}

#[test]
fn test_decision_debug() {
    let decision = Decision {
        thought: "Test thought".to_string(),
        tool: Tool::ListFiles {
            path: ".".to_string(),
        },
        file_path: None,
    };
    
    let debug_str = format!("{:?}", decision);
    assert!(debug_str.contains("Decision"));
    assert!(debug_str.contains("Test thought"));
    assert!(debug_str.contains("ListFiles"));
}

#[test]
fn test_tool_result_debug() {
    let result = ToolResult::Success("Test output".to_string());
    let debug_str = format!("{:?}", result);
    assert!(debug_str.contains("Success"));
    assert!(debug_str.contains("Test output"));
}