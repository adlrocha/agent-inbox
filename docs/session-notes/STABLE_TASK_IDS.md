# Stable Task IDs for Follow-up Messages

## The Problem

Previously, each follow-up message created a NEW task with a new UUID, leading to:
- Multiple tasks for same conversation
- No way to resume completed tasks
- Inbox spam

## The Solution: Stable Task IDs

**Use conversation ID as the basis for task ID**

### Extension Changes

```javascript
// OLD: Random UUID each time
const taskId = generateUUID();  // Different ID every time!

// NEW: Stable ID based on conversation
const taskId = `${this.agentType}-${conversationId}`;
// Example: "claude_web-abc-123-def"
```

**Benefits:**
- Same conversation = same task ID
- Follow-ups automatically reuse existing task
- Works across page reloads
- Database handles updates correctly

### Backend Changes (agent-bridge)

**OLD behavior:**
```rust
"running" => {
    db.insert_task(&task)?;  // Fails if task exists!
}
```

**NEW behavior (Upsert logic):**
```rust
"running" => {
    if let Some(mut existing_task) = db.get_task_by_id(&task_id)? {
        // Task exists - update to running (follow-up!)
        existing_task.status = TaskStatus::Running;
        existing_task.updated_at = Utc::now();
        existing_task.completed_at = None;
        db.update_task(&existing_task)?;
    } else {
        // Task doesn't exist - create new one
        db.insert_task(&task)?;
    }
}
```

## How It Works

### First Message

```
Extension:
  conversationId = "abc-123"
  taskId = "claude_web-abc-123"
  status = "running"
  → Send to backend

Backend:
  get_task_by_id("claude_web-abc-123") → None
  → insert_task() - creates new task
  → Task #claude_web-abc-123 created

CLI:
  $ agent-inbox list --all
  RUNNING:
    1. [claude.ai] "Hello" (5s ago)
```

### Message Completes

```
Extension:
  isGenerating = false
  status = "completed"
  → Send to backend

Backend:
  get_task_by_id("claude_web-abc-123") → Found
  → update_task() - mark as completed
  → Task #claude_web-abc-123 completed

CLI:
  $ agent-inbox list --all
  COMPLETED:
    1. [claude.ai] "Hello" (30s ago)
```

### Follow-up Message (KEY!)

```
Extension:
  conversationId = "abc-123" (SAME!)
  taskId = "claude_web-abc-123" (SAME!)
  status = "running"
  → Send to backend

Backend:
  get_task_by_id("claude_web-abc-123") → Found (completed)
  → update_task() - change to running, clear completed_at
  → Task #claude_web-abc-123 back to running!

CLI:
  $ agent-inbox list --all
  RUNNING:
    1. [claude.ai] "Hello" (2s ago)  ← SAME TASK!
```

### Follow-up Completes

```
Extension:
  isGenerating = false
  status = "completed"
  → Send to backend

Backend:
  get_task_by_id("claude_web-abc-123") → Found
  → update_task() - mark as completed again

CLI:
  $ agent-inbox list --all
  COMPLETED:
    1. [claude.ai] "Hello" (10s ago)  ← SAME TASK!
```

## Task ID Format

**Pattern:** `{agent_type}-{conversation_id}`

**Examples:**
- `claude_web-abc-123-def-456`
- `gemini_web-chat-xyz-789`
- `claude_code-session-123` (future)

**Why this format:**
- Unique per conversation
- Includes agent type (no collisions)
- Human-readable for debugging
- Stable across page reloads

## Database Behavior

### Task Lifecycle

```sql
-- First message creates task
INSERT INTO tasks (task_id, status, ...)
VALUES ('claude_web-abc123', 'running', ...);

-- First message completes
UPDATE tasks
SET status = 'completed', completed_at = 1234567890
WHERE task_id = 'claude_web-abc123';

-- Follow-up message resumes
UPDATE tasks
SET status = 'running', completed_at = NULL, updated_at = NOW()
WHERE task_id = 'claude_web-abc123';

-- Follow-up completes
UPDATE tasks
SET status = 'completed', completed_at = 1234567899
WHERE task_id = 'claude_web-abc123';
```

**Key:** Same `task_id` throughout!

## Comparison

### Before (Random UUIDs)

```
User: "Hello"
→ Task #uuid-1 created
→ Task #uuid-1 completed

User: "Tell me more"
→ Task #uuid-2 created (NEW!)
→ Task #uuid-2 completed

User: "Thanks"
→ Task #uuid-3 created (NEW!)
→ Task #uuid-3 completed

Result: 3 tasks in inbox for 1 conversation
```

### After (Stable IDs)

```
User: "Hello"
→ Task #claude_web-abc123 created
→ Task #claude_web-abc123 completed

User: "Tell me more"
→ Task #claude_web-abc123 updated to running (SAME!)
→ Task #claude_web-abc123 completed

User: "Thanks"
→ Task #claude_web-abc123 updated to running (SAME!)
→ Task #claude_web-abc123 completed

Result: 1 task in inbox for 1 conversation
```

## Edge Cases Handled

### Page Reload

```
1. User starts conversation
   → Task #claude_web-abc123 created

2. User closes tab (task stays in DB as "completed")

3. User reopens conversation tomorrow
   → Extension creates activeConversation with same taskId
   → Sends "running" update
   → Backend finds existing task and updates it
   → Same task resumes!
```

### Multiple Tabs

```
Tab 1: Conversation A (task #claude_web-aaa)
Tab 2: Conversation B (task #claude_web-bbb)

Each tab has unique conversationId
→ Different task IDs
→ No collision
```

### URL Navigation

```
1. User in conversation A (task #claude_web-aaa)
2. User navigates to conversation B
   → conversationId changes
   → activeConversation reset
   → New taskId generated (#claude_web-bbb)
   → Different tasks, no collision
```

## Benefits

1. **Deduplication** - One task per conversation
2. **Persistence** - Works across page reloads
3. **Resumability** - Completed tasks can return to running
4. **Simplicity** - No complex state tracking needed
5. **Debuggability** - Task ID shows conversation ID

## Testing

### Test Follow-up Resumption

```bash
# 1. Start conversation in Claude.ai
# Wait for completion

$ agent-inbox list --all
COMPLETED:
  1. [claude.ai] "Hello" (30s ago)

# Note the task stays completed

# 2. Send follow-up message
# Watch it return to running

$ agent-inbox list --all
RUNNING:
  1. [claude.ai] "Hello" (2s ago)  ← SAME TASK!

# 3. Wait for completion

$ agent-inbox list --all
COMPLETED:
  1. [claude.ai] "Hello" (10s ago)  ← STILL SAME TASK!
```

### Test Page Reload

```bash
# 1. Start conversation, complete it
$ agent-inbox list --all
COMPLETED:
  1. [claude.ai] "Test" (1m ago)

# 2. Close and reopen browser tab with same conversation

# 3. Send message
# Should reuse same task!

$ agent-inbox list --all
RUNNING:
  1. [claude.ai] "Test" (3s ago)  ← SAME TASK RESUMED!
```

## Installation

```bash
cd ~/workspace/personal/agent-notifications

# Build binaries
cargo build --release

# Install (requires sudo)
sudo cp target/release/{agent-inbox,agent-bridge} /usr/local/bin/

# Reload extension
brave://extensions → Agent Inbox Tracker → Reload
```

## Summary

**Key Change:** Task IDs are now stable, based on conversation ID.

**Result:**
- ✅ One task per conversation (deduplication)
- ✅ Completed tasks can return to running (resumability)
- ✅ Works across page reloads (persistence)
- ✅ Simple logic (no complex state machine)

**The right behavior is now implemented!**
