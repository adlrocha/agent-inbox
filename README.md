# Agent Inbox

A CLI-first notification system that tracks tasks across multiple LLM/coding agents (Claude Web, Gemini Web, Claude Code, OpenCode, etc.) and provides a simple inbox-style dashboard to view tasks requiring attention.

## Features

- **Unified Task Tracking**: Track tasks across different AI agents in one place
- **Transparent Wrappers**: Auto-track CLI agents without changing your workflow
- **Parallel Tracking**: Monitor multiple instances of the same agent simultaneously
- **Simple CLI Interface**: View, manage, and monitor tasks from the command line
- **Automatic Cleanup**: Completed tasks auto-delete after 1 hour (configurable)
- **Background Monitoring**: Detect when tasks need attention (stdin, stalls)
- **SQLite Backend**: Fast, reliable, and concurrent-safe storage
- **Extensible**: Easy template for wrapping any CLI coding agent

## Current Status

✅ **Phase 1: Core CLI Dashboard** - COMPLETE
- Working CLI inbox for viewing and managing tasks
- Manual task reporting via `agent-inbox report` commands
- SQLite database with automatic cleanup
- All unit tests passing

✅ **Phase 2: CLI Tool Wrappers** - COMPLETE
- Transparent wrapper scripts for Claude Code and OpenCode
- Automatic task registration and completion detection
- Background monitoring with pluggable detector architecture
- Setup script for easy installation
- Template for wrapping additional agents
- **Tested with parallel agents** ✓

✅ **Phase 3: Browser Extension** - COMPLETE
- Chrome/Firefox extension for web LLMs
- Content scripts for Claude.ai and Gemini
- Native messaging bridge (agent-bridge binary)
- Automatic conversation tracking
- Generation start/complete detection
- One-command installation

## Installation

### Prerequisites

- Rust 1.70+ (for building)
- Linux (tested on Arch Linux)

### Build and Install

```bash
# Clone the repository
cd /path/to/agent-notifications

# Install
./install.sh
```

This will:
1. Build the release binary
2. Install `agent-inbox` to `/usr/local/bin/` (or `~/.local/bin/` if no sudo)
3. Create `~/.agent-tasks/` directory

### Phase 2: Set Up Wrappers (Automatic Tracking)

After installing the core CLI, set up wrappers to automatically track your coding agents:

```bash
# Set up wrappers for detected agents
./setup-wrappers.sh

# Reload your shell
source ~/.bashrc  # or ~/.zshrc

# Test it
claude --help  # Should now be tracked
agent-inbox list --all
```

The setup script will:
1. Detect which agents are installed (claude, opencode, etc.)
2. Install wrapper scripts to `~/.agent-tasks/wrappers/`
3. Add transparent aliases to your shell RC file
4. Create backups of original binaries

**Supported agents out of the box:**
- Claude Code (`claude`)
- OpenCode (`opencode`)

**Want to wrap other agents?** See [WRAPPING_AGENTS.md](WRAPPING_AGENTS.md) for a complete guide on wrapping Cursor, Aider, Windsurf, or any other CLI agent.

### Phase 3: Install Browser Extension (Optional - For Web LLMs)

To track Claude.ai and Gemini conversations:

```bash
# Install extension and native messaging host
./install-extension.sh

# Follow prompts to:
# 1. Load extension in browser (chrome://extensions)
# 2. Copy extension ID
# 3. Update native messaging manifest
# 4. Reload extension
```

See [EXTENSION.md](EXTENSION.md) for detailed installation guide and troubleshooting.

## Usage

### Basic Commands

```bash
# Show tasks needing attention (default)
agent-inbox

# List all tasks
agent-inbox list --all

# List tasks by status
agent-inbox list --status running
agent-inbox list --status needs_attention
agent-inbox list --status completed
agent-inbox list --status failed

# Show detailed task information
agent-inbox show <task-id>

# Clear a specific task
agent-inbox clear <task-id>

# Clear all completed and failed tasks
agent-inbox clear-all

# Watch tasks in real-time (refreshes every 2s)
agent-inbox watch

# Manual cleanup of old completed tasks
agent-inbox cleanup --retention-secs 3600
```

### Manual Task Reporting (Phase 1)

You can manually report task status using the `report` subcommand:

```bash
# Start a task
TASK_ID=$(uuidgen)
agent-inbox report start "$TASK_ID" "claude_code" "$PWD" "My task description" --pid $$ --ppid $PPID

# Mark task as needing attention
agent-inbox report needs-attention "$TASK_ID" "Waiting for user input"

# Complete a task
agent-inbox report complete "$TASK_ID" --exit-code 0

# Report task failure
agent-inbox report failed "$TASK_ID" 1
```

## Architecture

```
┌─────────────────────────────────┐
│     User CLI Commands           │
└───────────┬─────────────────────┘
            │
    ┌───────▼────────┐
    │  agent-inbox   │  (Rust binary)
    └───────┬────────┘
            │
    ┌───────▼────────┐
    │  SQLite DB     │  (~/.agent-tasks/tasks.db)
    └────────────────┘
```

### Database Schema

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY AUTOINCREMENT,
    task_id TEXT UNIQUE NOT NULL,          -- UUID
    agent_type TEXT NOT NULL,               -- 'claude_web', 'gemini_web', 'claude_code', etc.
    title TEXT NOT NULL,                    -- First 100 chars of prompt
    status TEXT NOT NULL,                   -- 'running', 'completed', 'needs_attention', 'failed'
    created_at INTEGER NOT NULL,            -- Unix timestamp
    updated_at INTEGER NOT NULL,
    completed_at INTEGER,                   -- Timestamp when finished
    pid INTEGER,                            -- Process ID for CLI tools
    ppid INTEGER,                           -- Parent process ID
    monitor_pid INTEGER,                    -- Background monitor process ID
    attention_reason TEXT,                  -- Why it needs attention
    exit_code INTEGER,                      -- Exit code when completed/failed
    context TEXT,                           -- JSON: {url, project_path, session_id}
    metadata TEXT                           -- JSON: agent-specific data
);
```

### Task States

- **running**: Task in progress
- **completed**: Finished successfully (auto-cleared after 1 hour)
- **needs_attention**: Waiting for user input/approval
- **failed**: Errored out

## Development

### Run Tests

```bash
cargo test
```

All tests should pass:
- Unit tests for task model
- Unit tests for database operations
- Integration tests for CLI commands

### Build for Development

```bash
cargo build
./target/debug/agent-inbox --help
```

## Configuration

Default configuration (future):
- **Database**: `~/.agent-tasks/tasks.db`
- **Cleanup retention**: 3600 seconds (1 hour) for completed tasks
- **Wrappers directory**: `~/.agent-tasks/wrappers/`

## Roadmap

### Phase 2: CLI Tool Wrappers (Next)
- [ ] Create wrapper shell scripts for Claude Code and OpenCode
- [ ] Implement transparent aliasing
- [ ] Add background monitoring for task state changes
- [ ] Implement "needs attention" detectors (stdin, output patterns, stall detection)

### Phase 3: Browser Extension
- [ ] Build Chrome/Firefox extension structure
- [ ] Implement Claude.ai content script
- [ ] Implement Gemini content script
- [ ] Create native messaging host
- [ ] Set up native messaging manifest

### Future Enhancements
- Desktop notifications (libnotify)
- More agent integrations (Cursor, Windsurf, etc.)
- Web dashboard
- Task analytics
- Daemon mode with Unix socket API

## Contributing

This is a personal project, but suggestions and improvements are welcome!

## License

MIT (or your preferred license)

## Troubleshooting

### Database Issues

If you encounter database corruption:
```bash
rm ~/.agent-tasks/tasks.db
# Database will be recreated on next run
```

### Permission Issues

If `agent-inbox` command not found:
```bash
# Ensure /usr/local/bin is in your PATH
echo $PATH | grep /usr/local/bin

# If not, add to ~/.bashrc or ~/.zshrc:
export PATH="/usr/local/bin:$PATH"
```

## Testing Phase 1

Follow the verification steps from the plan:

```bash
# 1. Create a test task
TASK_ID=$(uuidgen)
agent-inbox report start "$TASK_ID" "claude_code" "$PWD" "test task" --pid $$

# 2. List tasks
agent-inbox list --all
# Should show 1 running task

# 3. Mark as needs attention
agent-inbox report needs-attention "$TASK_ID" "Waiting for input"

# 4. Check default view
agent-inbox
# Should show task needing attention

# 5. Complete the task
agent-inbox report complete "$TASK_ID" --exit-code 0

# 6. List all tasks
agent-inbox list --all
# Should show 1 completed task

# 7. Clean up
agent-inbox cleanup --retention-secs 0

# 8. Verify cleanup
agent-inbox list --all
# Should show no tasks
```

---

## Quick Reference

### Common Workflows

**Daily use (Phase 2):**
```bash
# Just use your agents normally - they're automatically tracked!
claude "implement new feature"
opencode "write tests"

# Check what needs attention
agent-inbox

# Watch in real-time
agent-inbox watch
```

**Manual tracking (Phase 1):**
```bash
TASK_ID=$(uuidgen)
agent-inbox report start "$TASK_ID" "my_agent" "$PWD" "task description" --pid $$
# ... do work ...
agent-inbox report complete "$TASK_ID" --exit-code 0
```

**Wrapping new agents:**
```bash
# 1. Copy template
cp wrappers/TEMPLATE-wrapper wrappers/myagent-wrapper

# 2. Edit AGENT_NAME and AGENT_TYPE
vim wrappers/myagent-wrapper

# 3. Install
chmod +x wrappers/myagent-wrapper
./setup-wrappers.sh
source ~/.bashrc
```

### Files and Directories

```
~/.agent-tasks/
├── tasks.db                    # SQLite database
├── wrappers/                   # Wrapper scripts
│   ├── claude-wrapper
│   ├── opencode-wrapper
│   └── *.original             # Original binaries

/usr/local/bin/
└── agent-inbox                 # Main CLI binary

~/.bashrc or ~/.zshrc
└── # Contains aliases for wrapped agents
```

---

**Status**: Phase 1 Complete ✓ | Phase 2 Complete ✓ | Phase 3 Complete ✓ | **ALL PHASES COMPLETE!** 🎉

## Force Reset Command

If tasks get stuck or you want to start fresh, use the reset command:

```bash
# Interactive reset (asks for confirmation)
agent-inbox reset

# Force reset (no confirmation)
agent-inbox reset --force
```

This will delete **ALL tasks** regardless of status (running, completed, needs_attention, failed).

**When to use:**
- Tasks are stuck in "running" state
- Want to start with a clean slate
- Database has corrupted entries
- Testing/debugging

**Safety:**
- Shows all tasks before deletion
- Requires typing "yes" to confirm (unless --force)
- Cannot be undone

