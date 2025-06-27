@echo off
setlocal

:: Get the directory where this script is located
set "SCRIPT_DIR=%~dp0"

:: Construct the full path to the target/release directory
set "AGENT_PATH=%SCRIPT_DIR%target\release"

echo Adding "%AGENT_PATH%" to your user PATH environment variable...

:: Add to user PATH
setx PATH "%PATH%;%AGENT_PATH%"

echo.
echo PATH update initiated.
echo You may need to restart your command prompt or PowerShell for the changes to take effect.
echo To verify, open a NEW terminal and type: cli_coding_agent --version
echo.

endlocal

