# Wrapping Additional Coding Agents

This guide explains how to create wrappers for any CLI-based coding agent to track tasks in agent-inbox.

## Quick Start

1. Copy the template: `cp wrappers/TEMPLATE-wrapper wrappers/myagent-wrapper`
2. Edit the wrapper to customize agent name and type
3. Make it executable: `chmod +x wrappers/myagent-wrapper`
4. Run setup: `./setup-wrappers.sh`

## Wrapper Architecture

Each wrapper script does three things:

1. **Register Task**: Creates a task entry when the agent starts
2. **Monitor Process**: Starts a background monitor to detect when attention is needed
3. **Report Completion**: Updates the task status when the agent exits

```
┌──────────────────────────────────────────────┐
│   User runs: myagent "do something"          │
└──────────────┬───────────────────────────────┘
               │
        ┌──────▼──────────┐
        │  myagent-wrapper│  (intercepts command)
        └──────┬──────────┘
               │
    ┌──────────┼──────────┐
    │          │          │
    ▼          ▼          ▼
[Register] [Monitor] [Execute]
    │          │          │
    │          │     ┌────▼────┐
    │          │     │ Real    │
    │          │     │ myagent │
    │          │     │ binary  │
    │          │     └────┬────┘
    │          │          │
    │          │     [Completes]
    │          │          │
    │          ▼          ▼
    │    [Detects]  [Cleanup]
    │    [Needs     [Report
    │     Attention] Complete]
    │          │          │
    └──────────┴──────────┘
               │
               ▼
       ┌──────────────┐
       │ agent-inbox  │
       │   database   │
       └──────────────┘
```

## Step-by-Step Guide

### 1. Identify Your Agent

First, determine the details of your coding agent:

- **Binary name**: The command you type (e.g., `cursor`, `aider`, `windsurf`)
- **Agent type**: A unique identifier for display (can be same as binary name)
- **Installation**: Where the agent is installed (check with `which <agent>`)

### 2. Create Wrapper from Template

```bash
cd /path/to/agent-notifications
cp wrappers/TEMPLATE-wrapper wrappers/myagent-wrapper
```

### 3. Customize the Wrapper

Edit `wrappers/myagent-wrapper`:

```bash
#!/bin/bash
# Wrapper for MyAgent CLI to track tasks in agent-inbox

# CUSTOMIZE THESE TWO LINES:
AGENT_NAME="myagent"        # The binary name (e.g., "cursor", "aider")
AGENT_TYPE="myagent"        # Display name in agent-inbox (e.g., "cursor", "aider")

# The rest of the template should work as-is for most agents
# ...
```

**Important variables to customize:**

| Variable | Description | Example |
|----------|-------------|---------|
| `AGENT_NAME` | Binary command name | `"cursor"`, `"aider"`, `"windsurf"` |
| `AGENT_TYPE` | Display identifier | `"cursor"`, `"aider"`, `"windsurf"` |

### 4. Make It Executable

```bash
chmod +x wrappers/myagent-wrapper
```

### 5. Install the Wrapper

Run the setup script:

```bash
./setup-wrappers.sh
```

This will:
- Copy the wrapper to `~/.agent-tasks/wrappers/`
- Add an alias to your shell RC file
- Create a symlink to the original binary

### 6. Reload Your Shell

```bash
source ~/.bashrc  # or ~/.zshrc
```

### 7. Test It

```bash
# Run your agent
myagent "help me with something"

# Check agent-inbox
agent-inbox list --all

# You should see your task!
```

## Real-World Examples

### Example 1: Wrapping Cursor

```bash
#!/bin/bash
# Wrapper for Cursor CLI

AGENT_NAME="cursor"
AGENT_TYPE="cursor"

# Find original binary
AGENT_BIN=$(which -a "$AGENT_NAME" | grep -v "$(readlink -f "$0")" | head -n 1)

if [ -z "$AGENT_BIN" ]; then
    echo "Error: Could not find original $AGENT_NAME binary" >&2
    exit 1
fi

# Generate task ID
TASK_ID=$(uuidgen)

# Create task title
if [ $# -eq 0 ]; then
    TASK_TITLE="$AGENT_NAME (interactive)"
else
    TASK_TITLE="$AGENT_NAME $*"
    TASK_TITLE="${TASK_TITLE:0:100}"
fi

# Register task
agent-inbox report start "$TASK_ID" "$AGENT_TYPE" "$PWD" "$TASK_TITLE" --pid $$ --ppid $PPID 2>/dev/null || {
    echo "Warning: Failed to register task with agent-inbox" >&2
}

# Start monitor
agent-inbox monitor "$TASK_ID" $$ &
MONITOR_PID=$!

# Cleanup function
cleanup() {
    EXIT_CODE=$?
    kill $MONITOR_PID 2>/dev/null
    agent-inbox report complete "$TASK_ID" --exit-code "$EXIT_CODE" 2>/dev/null || true
    exit $EXIT_CODE
}

trap cleanup EXIT INT TERM

# Execute
exec "$AGENT_BIN" "$@"
```

### Example 2: Wrapping Aider

```bash
#!/bin/bash
AGENT_NAME="aider"
AGENT_TYPE="aider"
# ... rest is identical to template ...
```

### Example 3: Wrapping Windsurf

```bash
#!/bin/bash
AGENT_NAME="windsurf"
AGENT_TYPE="windsurf"
# ... rest is identical to template ...
```

## Advanced Customization

### Custom Title Generation

If your agent has specific argument patterns, customize the title:

```bash
# Example: Extract filename for editor-based agents
if [ $# -eq 0 ]; then
    TASK_TITLE="$AGENT_NAME (interactive)"
elif [ -f "$1" ]; then
    # If first arg is a file, use it in title
    TASK_TITLE="$AGENT_NAME: $(basename "$1")"
else
    TASK_TITLE="$AGENT_NAME $*"
fi
TASK_TITLE="${TASK_TITLE:0:100}"
```

### Agent-Specific Metadata

Add custom context to the task:

```bash
# After registering the task, you can update it with custom metadata
# (This requires additional agent-inbox CLI support)

# Example: Add git branch to context
GIT_BRANCH=$(git branch --show-current 2>/dev/null || echo "unknown")
# Store in environment variable for monitor to use
export AGENT_INBOX_GIT_BRANCH="$GIT_BRANCH"
```

### Skip Monitoring for Fast Commands

For agents that run quick commands, you might want to skip monitoring:

```bash
# Add this before starting the monitor
if [[ "$*" == "--help" ]] || [[ "$*" == "--version" ]]; then
    # Don't monitor help/version commands
    exec "$AGENT_BIN" "$@"
fi

# Otherwise, proceed with normal monitoring
agent-inbox monitor "$TASK_ID" $$ &
# ...
```

## Troubleshooting

### Problem: "Could not find original binary"

**Cause**: The wrapper can't locate the original agent binary.

**Solution**:
1. Check that the agent is in your PATH: `which myagent`
2. Verify the `AGENT_NAME` variable matches exactly
3. If the binary is in an unusual location, specify it directly:
   ```bash
   AGENT_BIN="/opt/myagent/bin/myagent"
   ```

### Problem: Tasks not appearing in agent-inbox

**Cause**: The wrapper isn't being called.

**Solution**:
1. Verify the alias is active: `alias myagent`
2. Reload shell: `source ~/.bashrc`
3. Check wrapper permissions: `ls -la ~/.agent-tasks/wrappers/myagent-wrapper`
4. Test wrapper directly: `~/.agent-tasks/wrappers/myagent-wrapper --help`

### Problem: Wrapper creates task but doesn't track completion

**Cause**: Monitor process or cleanup trap isn't working.

**Solution**:
1. Check if monitor is running: `ps aux | grep "agent-inbox monitor"`
2. Test monitor directly:
   ```bash
   # In one terminal
   TASK_ID=$(uuidgen)
   agent-inbox report start "$TASK_ID" "test" "$PWD" "test" --pid $$
   agent-inbox monitor "$TASK_ID" $$ &

   # Wait, then exit terminal - task should complete
   ```
3. Check for bash trap issues (some shells handle traps differently)

### Problem: Alias conflicts with existing setup

**Cause**: You already have a custom alias or function for the agent.

**Solution**:
1. Manually edit your `~/.bashrc` or `~/.zshrc`
2. Comment out or rename your existing alias
3. Add the agent-inbox wrapper alias
4. Or, use a different command name:
   ```bash
   alias myagent-tracked='~/.agent-tasks/wrappers/myagent-wrapper'
   ```

## Best Practices

### 1. Test Before Committing

Always test a new wrapper with simple commands first:

```bash
myagent --help
agent-inbox list --all
```

### 2. Handle Edge Cases

Consider these scenarios:
- No arguments (interactive mode)
- Multiple parallel instances
- Quick-running commands (--help, --version)
- Commands that read from stdin
- Commands with special characters in args

### 3. Document Agent-Specific Quirks

If your agent has special behavior, document it:

```bash
# Note: MyAgent forks background processes - monitor may exit early
# Workaround: Monitor uses parent process detection
```

### 4. Version Your Wrapper

Add a version comment at the top:

```bash
#!/bin/bash
# MyAgent wrapper for agent-inbox
# Version: 1.0
# Date: 2026-01-12
# Author: Your Name
```

## Integration Checklist

When adding a new agent wrapper, verify:

- [ ] Wrapper script created and executable
- [ ] `AGENT_NAME` and `AGENT_TYPE` customized
- [ ] Tested with `--help` command
- [ ] Tested with actual work command
- [ ] Verified task appears in `agent-inbox list --all`
- [ ] Verified task completes when command exits
- [ ] Tested with Ctrl+C interruption
- [ ] Tested with multiple parallel instances
- [ ] Alias added to shell RC file
- [ ] Shell reloaded (`source ~/.bashrc`)

## Supported Agents

Current wrappers included:

- ✅ Claude Code (`claude`)
- ✅ OpenCode (`opencode`)

Community-contributed wrappers:

- ⏳ Cursor (pending)
- ⏳ Aider (pending)
- ⏳ Windsurf (pending)

Want to contribute a wrapper? Create a PR with:
1. The wrapper script in `wrappers/`
2. Test results
3. Any agent-specific notes

## Technical Details

### How the Wrapper Works

1. **Binary Discovery**: Uses `which -a` to find the original binary, excluding the wrapper itself
2. **Task ID**: Generates a UUID for unique task identification
3. **Process Tracking**: Captures `$$` (current PID) and `$PPID` (parent PID)
4. **Background Monitor**: Spawns `agent-inbox monitor` as a background job
5. **Trap Cleanup**: Registers bash trap to catch EXIT, INT, TERM signals
6. **Exec**: Uses `exec` to replace wrapper process with agent (preserves PID)

### Why Use `exec`?

The wrapper uses `exec "$AGENT_BIN" "$@"` instead of just calling the binary:

```bash
# ❌ Don't do this:
"$AGENT_BIN" "$@"
EXIT_CODE=$?

# ✅ Do this:
exec "$AGENT_BIN" "$@"
```

**Reason**: `exec` replaces the wrapper process with the agent, which means:
- The PID stays the same (monitor can track it)
- No extra process in the tree
- Signals (Ctrl+C) work correctly
- No resource overhead

### Failure Modes

The wrapper is designed to fail gracefully:

- If `agent-inbox` isn't installed → Warning printed, agent still runs
- If database is locked → Task not tracked, agent still runs
- If monitor crashes → Cleanup trap still reports completion
- If original binary not found → Error, wrapper exits

This ensures the wrapper never breaks your workflow.

## FAQ

**Q: Will the wrapper slow down my agent?**
A: No. The wrapper adds ~10ms overhead for task registration. The monitor runs in the background and polls every 5 seconds. The agent itself runs at full speed.

**Q: Can I wrap GUI applications?**
A: These wrappers are designed for CLI tools. For GUI apps, you'd need a different approach (possibly a system tray integration or desktop file modification).

**Q: Can I disable wrapping temporarily?**
A: Yes, several ways:
```bash
# Option 1: Call original binary directly
~/.agent-tasks/wrappers/myagent.original

# Option 2: Unalias temporarily
unalias myagent
myagent --help
alias myagent='~/.agent-tasks/wrappers/myagent-wrapper'

# Option 3: Use full path
/usr/bin/myagent --help
```

**Q: What if my agent requires sudo?**
A: Wrappers work with sudo, but task tracking won't work correctly (PID tracking breaks). Consider running agent-inbox with sudo or using agent without wrapper for sudo operations.

**Q: Can I wrap aliases or shell functions?**
A: No. Wrappers only work with actual binary executables. If your "agent" is an alias or function, you'll need to modify it directly to call agent-inbox commands.

---

**Need help?** Open an issue with:
- Agent name and version
- Your wrapper script
- Output of `agent-inbox list --all`
- Any error messages
