#!/bin/bash

# Get the directory where this script is located
SCRIPT_DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" &> /dev/null && pwd )"

# Construct the full path to the executable
AGENT_EXECUTABLE="$SCRIPT_DIR/target/release/cli_coding_agent"
LINK_NAME="/usr/local/bin/cli_coding_agent"

echo "Creating symbolic link for '$AGENT_EXECUTABLE' to '$LINK_NAME'..."

# Check if the executable exists
if [ ! -f "$AGENT_EXECUTABLE" ]; then
    echo "Error: Executable not found at '$AGENT_EXECUTABLE'."
    echo "Please ensure you have run 'cargo build --release' in the project root."
    exit 1
fi

# Create the symbolic link
sudo ln -sf "$AGENT_EXECUTABLE" "$LINK_NAME"

if [ $? -eq 0 ]; then
    echo "Symbolic link created successfully."
    echo "You can now run 'cli_coding_agent' from any directory."
    echo "To verify, type: cli_coding_agent --version"
else
    echo "Error: Failed to create symbolic link."
    echo "Please ensure you have appropriate permissions (e.g., run with sudo if necessary)."
fi
