# Agent Inbox Sandboxing Implementation Plan

## Overview

Add rootless Podman sandboxing to agent-inbox for secure, autonomous Claude Code execution with Telegram-based monitoring and control.

## Architecture

```
┌─────────────────────────────────────────────────────────────────────────────┐
│                              HOST SYSTEM                                    │
│  ┌─────────────┐  ┌─────────────┐  ┌─────────────────────────────────────┐  │
│  │ agent-inbox │  │  SQLite DB  │  │  Telegram Listener (daemon)         │  │
│  │   (CLI)     │  │  ~/.agent-  │  │                                     │  │
│  │             │  │  tasks/     │  │  • Polls Telegram for replies       │  │
│  │ Commands:   │  │             │  │  • Routes to container via named    │  │
│  │ - spawn     │  │  Tasks with │  │    pipe                             │  │
│  │ - status    │  │  container  │  │  • Sends notifications              │  │
│  │ - kill      │  │  references │  │                                     │  │
│  │ - list      │  │             │  │                                     │  │
│  │ - resume    │  └─────────────┘  └─────────────────────────────────────┘  │
│  └──────┬──────┘                                                           │
│         │                                                                   │
│         │  podman run                                                       │
│         ▼                                                                   │
│  ┌─────────────────────────────────────────────────────────────────────┐   │
│  │                    ROOTLESS PODMAN CONTAINERS                        │   │
│  │                                                                       │   │
│  │  ┌─────────────────────────────────────────────────────────────┐     │   │
│  │  │  Container: agent-inbox-<task_id>                           │     │   │
│  │  │                                                             │     │   │
│  │  │  ┌─────────────┐    ┌─────────────┐    ┌─────────────────┐  │     │   │
│  │  │  │ Claude Code │◄──►│    PTY      │◄──►│ input-forwarder │  │     │   │
│  │  │  │   (node)    │    │  (/dev/pts) │    │   (bash script) │  │     │   │
│  │  │  └─────────────┘    └──────┬──────┘    └────────┬────────┘  │     │   │
│  │  │                            │                    │           │     │   │
│  │  │                            └────────────────────┘           │     │   │
│  │  │                                                 reads from  │     │   │
│  │  │                                                 /tmp/agent- │     │   │
│  │  │                                                 input.pipe  │     │   │
│  │  └─────────────────────────────────────────────────────────────┘     │   │
│  │                                                                       │   │
│  │  Mounts:                                                              │   │
│  │  - Repo code: /workspace (RW)                                         │   │
│  │  - Agent hooks: /workspace/.claude/settings.json                      │   │
│  │                                                                       │   │
│  │  Network: host (ports 3000-3100, 8000-8100 available)                 │   │
│  │                                                                       │   │
│  │  Environment:                                                         │   │
│  │  - ANTHROPIC_API_KEY, etc. from host                                  │   │
│  │  - AGENT_TASK_ID, AGENT_INBOX_VERSION                                 │   │
│  │                                                                       │   │
│  └─────────────────────────────────────────────────────────────────────┘   │
│                                                                             │
└─────────────────────────────────────────────────────────────────────────────┘
                                    │
                                    ▼
                          ┌─────────────────┐
                          │   TELEGRAM      │
                          │   (your phone)  │
                          └─────────────────┘
```

## Input Flow (Reply from Telegram)

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Telegram  │────►│   Listener  │────►│ agent-inbox │────►│   podman    │
│   (reply)   │     │   (daemon)  │     │  inject cmd │     │    exec     │
└─────────────┘     └─────────────┘     └─────────────┘     └──────┬──────┘
                                                                   │
                                                                   ▼
                                                          ┌─────────────────┐
                                                          │  echo "msg" >   │
                                                          │ /tmp/agent-     │
                                                          │ input.pipe      │
                                                          └────────┬────────┘
                                                                   │
                                                                   ▼
                                                          ┌─────────────────┐
                                                          │ input-forwarder │
                                                          │  (in container) │
                                                          └────────┬────────┘
                                                                   │
                                                                   ▼
                                                          ┌─────────────────┐
                                                          │   Claude Code   │
                                                          │   (receives     │
                                                          │    input)       │
                                                          └─────────────────┘
```

## Database Schema Changes (v3)

```sql
-- Add container fields to tasks table
ALTER TABLE tasks ADD COLUMN container_id TEXT;
ALTER TABLE tasks ADD COLUMN sandbox_type TEXT DEFAULT 'none'; -- 'none', 'podman'
ALTER TABLE tasks ADD COLUMN sandbox_config TEXT; -- JSON: ports, image, etc.

-- New table for container state (for resume)
CREATE TABLE container_state (
    task_id TEXT PRIMARY KEY,
    container_name TEXT NOT NULL,
    repo_path TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    FOREIGN KEY (task_id) REFERENCES tasks(task_id) ON DELETE CASCADE
);

-- Index for container lookups
CREATE INDEX idx_container_id ON tasks(container_id);
```

## Module Structure

```
src/
├── main.rs                    # CLI entry, command dispatch
├── lib.rs                     # Public exports
├── cli/
│   └── mod.rs                 # Add Spawn, Kill, Resume commands
├── models/
│   ├── mod.rs
│   └── task.rs                # Add container fields
├── db/
│   └── mod.rs                 # Schema v3 migration, container ops
├── sandbox/
│   ├── mod.rs                 # Sandbox trait, factory
│   ├── podman.rs              # Podman implementation
│   └── input_forwarder.sh     # In-container input helper
├── notifications/
│   ├── mod.rs
│   ├── telegram.rs            # Minor updates
│   └── telegram_listener.rs   # Route to containers
└── agent_input.rs             # Update for container-aware injection
```

## Implementation Phases

### Phase 1: Core Sandbox Module

**Files:** `src/sandbox/mod.rs`, `src/sandbox/podman.rs`, `src/sandbox/input_forwarder.sh`

1. Create `Sandbox` trait with methods:
   - `spawn(repo_path, task_id, env_vars) -> Result<ContainerId>`
   - `kill(container_id) -> Result<()>`
   - `status(container_id) -> Result<ContainerStatus>`
   - `inject_input(container_id, message) -> Result<()>`
   - `list() -> Result<Vec<ContainerInfo>>`

2. Implement `PodmanSandbox`:
   - Check podman availability, install if missing
   - Build/pull base image
   - Create container with proper mounts, network, env
   - Start input-forwarder alongside claude-code

3. Create `input-forwarder.sh`:
   - Creates named pipe `/tmp/agent-input.pipe`
   - Reads lines from pipe
   - Writes to PTY (using `screen` or direct TTY)
   - Handles special commands (e.g., "\x03" for Ctrl+C)

### Phase 2: Database & Models

**Files:** `src/models/task.rs`, `src/db/mod.rs`

1. Update `Task` struct:

   ```rust
   pub struct Task {
       // ... existing fields ...
       pub container_id: Option<String>,
       pub sandbox_type: SandboxType, // enum: None, Podman
       pub sandbox_config: Option<SandboxConfig>,
   }
   ```

2. Schema migration v2 -> v3:
   - Add columns
   - Create container_state table
   - Migrate existing tasks (sandbox_type = 'none')

3. Add container operations:
   - `get_task_by_container_id()`
   - `list_running_containers()`
   - `update_container_state()`

### Phase 3: CLI Commands

**Files:** `src/cli/mod.rs`, `src/main.rs`, `wrappers/claude-wrapper`

1. Add CLI commands:

   ```rust
   Spawn {
       repo_path: PathBuf,
       #[arg(short, long)]
       sandbox: bool,  // Use podman sandbox
       #[arg(short, long)]
       task: Option<String>,  // Task description
   },
   Kill {
       task_id: String,
   },
   Resume {
       #[arg(short, long)]
       all: bool,  // Resume all stopped containers
   },
   ```

2. Update `ReportAction::Start` to include sandbox flag

3. Update wrapper script to support `--sandbox` flag

### Phase 4: Telegram Integration

**Files:** `src/notifications/telegram_listener.rs`, `src/agent_input.rs`

1. Update `agent_input.rs`:
   - Check if task has container_id
   - If yes, use `podman exec` to write to named pipe
   - If no, use existing PTY injection (backward compat)

2. Update telegram listener:
   - No major changes needed - uses existing `agent_input::inject()`

### Phase 5: Auto-Resume

**Files:** `src/main.rs`, `scripts/agent-inbox.service`

1. Add `Resume` command logic:
   - Query container_state table
   - Check if containers still exist via podman
   - Re-register tasks for running containers
   - Optionally re-attach to them

2. Create systemd user service for auto-start:
   - Runs `agent-inbox resume --all` on boot
   - Starts telegram listener

### Phase 6: Install Script & Hardening

**Files:** `install.sh`, `scripts/setup-podman.sh`

1. Update `install.sh`:
   - Check for podman, install if missing
   - Build sandbox base image
   - Setup systemd service (optional)

2. Security hardening:
   - Validate all container names (prevent injection)
   - Limit container resources (CPU, memory)
   - Audit all `podman exec` calls
   - Ensure rootless mode

## Input Protocol

Simple text-based protocol via named pipe:

```
# Normal text (sent as-is to Claude)
Hello, please continue with the implementation

# Special control sequences (prefix with \x00)
\x00\x03     # Ctrl+C (interrupt)
\x00\x04     # Ctrl+D (EOF)
\x00\x1a     # Ctrl+Z (suspend)
\x00\x03\x03 # Double Ctrl+C (force quit)
```

For now, only normal text is implemented. Control sequences can be added later.

## Security Considerations

1. **Rootless Podman**: Containers run as user, not root
2. **No new privileges**: Container cannot gain additional capabilities
3. **Seccomp**: Default seccomp profile blocks dangerous syscalls
4. **Resource limits**: CPU/memory limits prevent DoS
5. **Network**: Host mode is acceptable for local dev, but containers cannot bind privileged ports (<1024)
6. **Volume mounts**: Repo is mounted RW, but system directories are not
7. **Input validation**: All messages sanitized before pipe write
8. **Container naming**: Strict pattern `agent-inbox-{uuid}` prevents collisions

## Testing Plan

1. Unit tests:
   - Sandbox trait implementations
   - Database migrations
   - Input protocol parsing

2. Integration tests:
   - Spawn/kill container lifecycle
   - Input injection end-to-end
   - Telegram notification routing
   - Resume after restart

3. Security tests:
   - Verify rootless mode
   - Attempt container escape
   - Test input injection boundaries

## Rollback Plan

If sandboxing fails:

1. `sandbox_type = 'none'` allows traditional mode
2. Database migration is reversible
3. Existing PTY injection still works for non-sandboxed tasks
