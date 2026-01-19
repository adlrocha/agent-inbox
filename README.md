# Agent Inbox

A CLI-first notification system that tracks tasks across multiple LLM/coding agents (Claude Web, Gemini Web, Claude Code, OpenCode, etc.) and provides a simple inbox-style dashboard to view tasks requiring attention.

## Features

- **Unified Task Tracking**: Track tasks across different AI agents in one place
- **3-State Model**: Simple, reliable status tracking (Running → Completed → Exited)
- **Desktop Notifications**: Get notified when agents finish generating
- **Transparent Wrappers**: Auto-track CLI agents without changing your workflow
- **Browser Extension**: Track Claude.ai and Gemini conversations
- **Repo-Aware**: Shows git repo and branch in task titles for CLI agents
- **Auto-Reset on Restart**: Clears stale tasks automatically on login
- **SQLite Backend**: Fast, reliable, and concurrent-safe storage

## Task States

- **Running**: Agent is actively generating output
- **Completed**: Agent finished generating, waiting for user input
- **Exited**: Agent/tab closed or process terminated

## Installation

### Prerequisites

- Rust 1.70+ (for building)
- Linux (tested on Arch Linux)

### Build and Install

```bash
cd /path/to/agent-notifications

# Build
cargo build --release

# Install binaries
cp target/release/agent-inbox ~/.local/bin/
cp target/release/agent-bridge ~/.local/bin/
```

## Setup Scripts

### 1. Claude Code Wrapper

The wrapper enables automatic task tracking for Claude Code CLI:

```bash
# Copy wrapper to your path
mkdir -p ~/.agent-tasks/wrappers
cp wrappers/claude-wrapper ~/.agent-tasks/wrappers/

# Add alias to your shell RC (~/.bashrc or ~/.zshrc)
alias claude='~/.agent-tasks/wrappers/claude-wrapper'

# Reload shell
source ~/.bashrc
```

The wrapper:
- Creates a task when Claude starts
- Exports `AGENT_TASK_ID` for hooks to use
- Shows `[repo:branch]` in task title when in a git repo

### 2. Claude Code Hooks

Run the setup script to install hooks globally:

```bash
./scripts/setup-claude-hooks.sh
```

This installs hooks to `~/.claude/settings.json` that:
- **UserPromptSubmit**: Mark task as "running" when you send a prompt
- **Stop**: Mark task as "completed" + desktop notification when Claude finishes
- **SessionEnd**: Mark task as "exited" when you exit Claude Code

**Note**: Restart Claude Code after installing hooks for them to take effect.

### 3. Auto-Reset on Login/Restart

Automatically clear stale tasks when you log in:

```bash
./scripts/setup-auto-reset.sh
```

This creates a systemd user service that runs `agent-inbox reset --force` on each login.

To disable: `systemctl --user disable agent-inbox-reset.service`

### 4. Browser Extension (Claude.ai & Gemini)

To track web-based agents:

```bash
# 1. Load extension in browser
#    - Go to chrome://extensions (or brave://extensions)
#    - Enable "Developer mode"
#    - Click "Load unpacked" and select the `extension/` directory

# 2. Get extension ID from the extensions page

# 3. Update native messaging manifest
mkdir -p ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/
# (or ~/.config/google-chrome/NativeMessagingHosts/ for Chrome)

cat > ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json << EOF
{
  "name": "com.agent_tasks.bridge",
  "description": "Native messaging host for Agent Inbox extension",
  "path": "$HOME/.local/bin/agent-bridge",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF

# 4. Reload the extension
```

## Usage

### Basic Commands

```bash
# Show running tasks (default)
agent-inbox

# List all tasks
agent-inbox list --all

# List tasks by status
agent-inbox list --status running
agent-inbox list --status completed
agent-inbox list --status exited

# Show detailed task information
agent-inbox show <task-id>

# Clear a specific task
agent-inbox clear <task-id>

# Clear all completed and exited tasks
agent-inbox clear-all

# Force clear ALL tasks (useful when stuck)
agent-inbox reset --force

# Watch tasks in real-time (refreshes every 2s)
agent-inbox watch

# Manual cleanup of old completed tasks
agent-inbox cleanup --retention-secs 3600
```

### Manual Task Reporting

```bash
# Start a task
TASK_ID=$(uuidgen)
agent-inbox report start "$TASK_ID" "claude_code" "$PWD" "My task description"

# Mark task as running (generating)
agent-inbox report running "$TASK_ID"

# Mark task as completed (finished generating)
agent-inbox report complete "$TASK_ID"

# Mark task as exited (process terminated)
agent-inbox report exited "$TASK_ID" --exit-code 0
```

## Scripts Reference

| Script | Purpose |
|--------|---------|
| `scripts/setup-claude-hooks.sh` | Install Claude Code hooks globally (~/.claude/settings.json) |
| `scripts/setup-auto-reset.sh` | Install systemd service for auto-reset on login |
| `wrappers/claude-wrapper` | Wrapper script for Claude Code CLI |
| `wrappers/opencode-wrapper` | Wrapper script for OpenCode CLI |

## Architecture

```
┌──────────────────────────────────────────────────────────────┐
│                        User                                   │
└──────────────────────────────────────────────────────────────┘
          │                    │                    │
          ▼                    ▼                    ▼
┌──────────────────┐  ┌──────────────────┐  ┌──────────────────┐
│  Claude Code     │  │  Claude.ai       │  │  Gemini          │
│  (wrapper+hooks) │  │  (extension)     │  │  (extension)     │
└────────┬─────────┘  └────────┬─────────┘  └────────┬─────────┘
         │                     │                     │
         │                     ▼                     │
         │            ┌──────────────────┐           │
         │            │  agent-bridge    │           │
         │            │  (native msg)    │           │
         │            └────────┬─────────┘           │
         │                     │                     │
         ▼                     ▼                     ▼
┌──────────────────────────────────────────────────────────────┐
│                      agent-inbox CLI                          │
└──────────────────────────────────────────────────────────────┘
                              │
                              ▼
                    ┌──────────────────┐
                    │    SQLite DB     │
                    │ ~/.agent-tasks/  │
                    └──────────────────┘
```

## Development

```bash
# Run tests
cargo test

# Build for development
cargo build

# Build release
cargo build --release
```

## License

Apache 2.0
