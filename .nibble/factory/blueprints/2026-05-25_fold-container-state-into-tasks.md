# Blueprint: Fold container_state into tasks

**Date**: 2026-05-25
**Tier**: Full (database schema change, 20+ functions affected)
**Author**: Agent

## Problem

`container_state` is a separate table keyed by `task_id` with a FK to `tasks`. It duplicates data already in the `tasks` table (`container_name` ≈ `container_id`, `repo_path` ≈ `context.project_path`). The two-table design creates sync bugs:

1. **prune_stale_tasks `Dead` branch**: never deletes `container_state`, has no status guard → useless DB writes every cycle, was sending duplicate Telegram notifications every ~5 min.
2. **Inconsistent cleanup**: Some code paths delete `container_state` (`list`, `resume`, `kill_all`), others don't (`prune_stale_tasks`).
3. **Two sources of truth**: callers must query both tables and keep them in sync.

## Field Mapping

| `container_state`      | `tasks` equivalent              | Gap? |
|------------------------|---------------------------------|------|
| `container_name`       | `container_id` (Podman accepts both name and ID) | Need to store *name* too, or ensure we always use container_id with Podman |
| `repo_path`            | `context.project_path`          | Hermes: `"__hermes__"` vs `None` |
| `worktree_path`        | —                               | **Missing** — add to Task |
| `created_at`           | `tasks.created_at`              | Redundant |

## Design

### New fields on Task model

```rust
pub struct Task {
    // ... existing fields ...
    pub container_id: Option<String>,      // already exists — stores Podman container ID
    pub container_name: Option<String>,    // NEW — stores Podman container name (e.g. nibble-20260525-...)
    pub repo_path: Option<String>,         // NEW — canonical host repo path (replaces context.project_path for sandbox lookups)
    pub worktree_path: Option<String>,     // NEW — git worktree path if applicable
    pub sandbox_type: SandboxType,         // already exists
    pub sandbox_config: Option<SandboxConfig>, // already exists
}
```

- `container_name`: Separated from `container_id` because Podman's `inspect` returns both and some operations are name-based. Adding it as a dedicated field avoids ambiguity.
- `repo_path`: Lifted out of `context.project_path`. For Hermes sandboxes, stores `"__hermes__"` (previously in `container_state.repo_path`). For regular sandboxes, stores the canonical absolute host path. This makes sandbox-by-repo lookups a simple DB query without parsing JSON context.
- `worktree_path`: Direct lift from `container_state.worktree_path`.

### DB Migration (v10)

```sql
-- 1. Add new columns to tasks
ALTER TABLE tasks ADD COLUMN container_name TEXT;
ALTER TABLE tasks ADD COLUMN repo_path TEXT;
ALTER TABLE tasks ADD COLUMN worktree_path TEXT;

-- 2. Copy data from container_state
UPDATE tasks SET
    container_name = (SELECT container_name FROM container_state WHERE container_state.task_id = tasks.task_id),
    repo_path = (SELECT repo_path FROM container_state WHERE container_state.task_id = tasks.task_id),
    worktree_path = (SELECT worktree_path FROM container_state WHERE container_state.task_id = tasks.task_id);

-- 3. Create index for repo_path lookups (replaces container_state queries)
CREATE INDEX idx_tasks_repo_path ON tasks(repo_path) WHERE repo_path IS NOT NULL;
```

Note: We do NOT drop `container_state` in v10. That happens in v11 after a release cycle confirms no rollback is needed.

### Replaced DB methods

| Old method (container_state) | New method (tasks) |
|---|---|
| `upsert_container_state_with_worktree()` | Removed — data written via `insert_task`/`update_task` |
| `get_container_state()` | Removed — `get_task_by_id()` suffices |
| `delete_container_state()` | Removed — task deletion handles it |
| `get_container_state_by_repo_path()` | `get_task_by_repo_path()` — new method |
| `get_all_containers_by_repo_path()` | `get_tasks_by_repo_path()` — new method |
| `list_container_states()` | `list_sandbox_tasks()` — new method, filters for `sandbox_type != 'none'` |
| `get_worktree_path()` | Removed — read `task.worktree_path` directly |

### New DB methods

```rust
/// Find the most recent sandbox task for a given repo path.
fn get_task_by_repo_path(&self, repo_path: &str) -> Result<Option<Task>>;

/// Return all sandbox tasks for a given repo path, newest first.
fn get_tasks_by_repo_path(&self, repo_path: &str) -> Result<Vec<Task>>;

/// List all tasks that have an associated sandbox (sandbox_type != 'none').
fn list_sandbox_tasks(&self) -> Result<Vec<Task>>;
```

### Invariants

- **INV-1**: Every task with `sandbox_type != None` has `container_id` and `container_name` set.
- **INV-2**: `repo_path` is always the canonical absolute path (or `"__hermes__"` for Hermes sandboxes).
- **INV-3**: `worktree_path` is `Some(path)` only for git worktree sandboxes.
- **INV-4**: `container_state` table is no longer written to (v10 writes to both; v11 drops the old table).
- **INV-5**: `prune_stale_tasks` only transitions tasks that are `Running` → `Exited` (idempotency guard).

### Affected callers (~20 call sites)

1. **Spawn (3 sites)**: Remove `upsert_container_state_with_worktree()`, store data in Task fields before `insert_task()`.
2. **prune_stale_tasks**: Use `list_sandbox_tasks()` + task fields. Add status guard on Dead branch.
3. **cmd_sandbox_list**: Use `list_sandbox_tasks()`. On Dead, mark task exited + no separate cleanup.
4. **cmd_sandbox_kill_all**: Use `list_sandbox_tasks()`. Kill + set_exited in one pass.
5. **cmd_sandbox_resume**: Use `list_sandbox_tasks()`. Same logic, just from task fields.
6. **find_healthy_sandbox_for_repo**: Use `get_task_by_repo_path()`.
7. **resolve_sandbox_id**: Use `get_task_by_repo_path()`.
8. **find_hermes_sandbox**: Use `list_sandbox_tasks()` filtered by agent_type.
9. **Hermes singleton check (2 sites)**: Filter `list_sandbox_tasks()` by Hermes.
10. **telegram_listener handle_sandboxes_command**: Use `list_sandbox_tasks()`.
11. **telegram_listener find_or_spawn_for_cron**: Use `get_tasks_by_repo_path()`.
12. **inject_prompt / kill --worktree**: Read `task.worktree_path` directly.
13. **cmd_sandbox_kill (single)**: Already resolves by task_id, just use task fields.

### Rollback safety

- v10 keeps `container_state` intact and stops writing to it.
- If rollback is needed, v9 code can still read from `container_state`.
- v11 (future) drops the table.
