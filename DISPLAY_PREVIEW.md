# CLI Display Preview

## New Pretty Display Features

### Colors & Icons
- ⚠️  **Yellow** for tasks needing attention
- ▶️  **Blue** for running tasks
- ✓ **Green** for completed tasks
- ✗ **Red** for failed tasks

### Agent Badges
- **Purple** [claude.ai] - Claude web conversations
- **Blue** [gemini] - Gemini web conversations
- **Cyan** [claude-code] - Claude Code CLI sessions
- **Green** [opencode] - OpenCode sessions

### Layout Improvements
- Box-drawn header with title
- Colored status indicators (● bullets)
- Truncated titles (max 60 chars)
- Dimmed timestamps
- Separator lines between sections
- Helpful footer with next commands

---

## Example Output

### Empty State
```
No active tasks
Start a conversation in Claude.ai or Gemini to create tasks
```

### With Tasks
```
╭─────────────────────────────────────────────╮
│  Agent Inbox                              │
╰─────────────────────────────────────────────╯

2 need attention  •  3 running  •  1 completed

⚠️  NEEDS ATTENTION
──────────────────────────────────────────────────
   1. ● [claude.ai] "Refactor authentication system" (5m ago)
      → Waiting for approval to proceed

   2. ● [claude-code:4521] "Fix type errors in user service" (12m ago)
      → Tests failing, needs review

▶️  RUNNING
──────────────────────────────────────────────────
   3. ● [gemini] "Analyze performance bottlenecks" (3m ago)

   4. ● [claude-code:4893] "Add authentication flow" (2m ago)

   5. ● [claude.ai] "Update API documentation" (1m ago)

✓ COMPLETED
──────────────────────────────────────────────────
   6. ● [gemini] "Generate test cases" (30m ago)

 Completed tasks auto-clear after 1 hour
 Run agent-inbox show <id> for details
```

### Task Details View
```
╭─────────────────────────────────────────────╮
│  Task Details                            │
╰─────────────────────────────────────────────╯

Status: RUNNING

ID: abc-123-def-456
Agent: claude_web
Title: Refactor authentication system

Timestamps:
  Created:  2026-01-13 10:30:45 UTC
  Updated:  2026-01-13 10:35:20 UTC

Context:
  URL:        https://claude.ai/chat/abc-123
  Session ID: session-xyz
```

---

## Color Scheme

| Element | Color | ANSI Code |
|---------|-------|-----------|
| Header box | Cyan (bold) | `\x1b[1m\x1b[36m` |
| Needs Attention | Bright Yellow | `\x1b[93m` |
| Running | Bright Blue | `\x1b[94m` |
| Completed | Green | `\x1b[32m` |
| Failed | Bright Red | `\x1b[91m` |
| Agent: claude.ai | Magenta | `\x1b[35m` |
| Agent: gemini | Blue | `\x1b[34m` |
| Agent: claude-code | Cyan | `\x1b[36m` |
| Timestamps/hints | Gray (dim) | `\x1b[2m\x1b[90m` |
| Separator lines | Gray | `\x1b[90m` |

---

## Comparison

### Before
```
Agent Tasks (2 need attention, 3 running, 1 completed)

NEEDS ATTENTION:
  1. [claude_web] "Refactor authentication system" (5m ago)
     → Waiting for approval to proceed

RUNNING:
  2. [gemini_web] "Analyze performance bottlenecks" (3m ago)
  3. [claude_code:4893] "Add authentication flow" (2m ago)

COMPLETED:
  4. [gemini_web] "Generate test cases" (30m ago)

Completed tasks auto-clear after 1 hour
Run 'agent-inbox show <task_id>' for details
```

### After
```
╭─────────────────────────────────────────────╮
│  Agent Inbox                              │
╰─────────────────────────────────────────────╯

2 need attention  •  3 running  •  1 completed

⚠️  NEEDS ATTENTION
──────────────────────────────────────────────────
   1. ● [claude.ai] "Refactor authentication system" (5m ago)
      → Waiting for approval to proceed

▶️  RUNNING
──────────────────────────────────────────────────
   2. ● [gemini] "Analyze performance bottlenecks" (3m ago)

   3. ● [claude-code:4893] "Add authentication flow" (2m ago)

✓ COMPLETED
──────────────────────────────────────────────────
   4. ● [gemini] "Generate test cases" (30m ago)

 Completed tasks auto-clear after 1 hour
 Run agent-inbox show <id> for details
```

---

## Features Added

1. **Visual Hierarchy**
   - Box-drawn header
   - Section headers with icons
   - Separator lines
   - Indentation

2. **Color Coding**
   - Status-based colors
   - Agent-based colors
   - Dim/bright for importance

3. **Icons**
   - ⚠️  for attention
   - ▶️  for running
   - ✓ for completed
   - ✗ for failed
   - ● status bullets
   - → for sub-info

4. **Better Labels**
   - Friendly agent names (claude.ai, gemini)
   - Concise status badges
   - Truncated long titles

5. **Improved Spacing**
   - Blank lines between sections
   - Consistent indentation
   - Aligned elements

---

## Installation

To use the new display:

```bash
cd ~/workspace/personal/agent-notifications
cargo build --release
sudo cp target/release/agent-inbox /usr/local/bin/

# Test it
agent-inbox list --all
```

---

## Notes

- All colors use standard ANSI codes (widely supported)
- Emojis work in most modern terminals
- Box drawing characters (╭╮╰╯─) are Unicode
- If emojis don't display, they gracefully degrade to text
- Colors can be disabled with `NO_COLOR` env var (not implemented yet)

---

## Future Enhancements

Could add:
- `--no-color` flag for plain text
- `--json` format for scripting
- More agent icons/colors
- Time-based color coding (older = dimmer)
- Interactive mode (arrow keys to navigate)
