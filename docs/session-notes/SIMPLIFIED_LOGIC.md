# Simplified State Logic

## The Problem with the Old Approach

The previous implementation tracked `wasGenerating` and `isGenerating` to detect state transitions, which led to:
- Over-complicated logic with 4 different states
- Race conditions with transition flags
- Timing issues (minimum 1-second checks)
- Confusion about when to send updates

## New Simplified Approach

**Simple rule: Just check the current state every poll.**

```javascript
checkState(conversationId, isGenerating, title) {
  // Create task if needed
  if (!this.activeConversation) {
    this.activeConversation = {
      taskId: generateUUID(),
      lastStatus: null
    };
  }

  // Determine current status
  const currentStatus = isGenerating ? "running" : "completed";

  // Only send update if status changed
  if (this.activeConversation.lastStatus !== currentStatus) {
    sendTaskUpdate(taskId, currentStatus, ...);
    this.activeConversation.lastStatus = currentStatus;
  }
}
```

## How It Works

### First Message

```
Poll #1: isGenerating = true
  → No activeConversation exists
  → Create task with taskId
  → lastStatus = null
  → currentStatus = "running"
  → Send update: "running"
  → Set lastStatus = "running"

Poll #2: isGenerating = true
  → lastStatus = "running"
  → currentStatus = "running"
  → No change, no update sent

Poll #3: isGenerating = false
  → lastStatus = "running"
  → currentStatus = "completed"
  → Send update: "completed"
  → Set lastStatus = "completed"
```

### Follow-up Message

```
Poll #4: isGenerating = true (user sent follow-up)
  → activeConversation exists (same task!)
  → lastStatus = "completed"
  → currentStatus = "running"
  → Send update: "running" (reactivates task)
  → Set lastStatus = "running"

Poll #5: isGenerating = false (generation finishes)
  → lastStatus = "running"
  → currentStatus = "completed"
  → Send update: "completed"
  → Set lastStatus = "completed"
```

## Benefits

1. **No race conditions** - No transition flags needed
2. **No timing issues** - No minimum duration checks
3. **Simpler logic** - Just check current state
4. **Easier to debug** - Clear "status changed" logs
5. **Fewer edge cases** - Only 2 states: running or completed

## Comparison

### Old Logic (Complicated)

```javascript
const wasGenerating = this.activeConversation?.isGenerating || false;

if (!wasGenerating && isGenerating) {
  // Handle start
  if (!this.activeConversation) {
    // NEW
  } else {
    // FOLLOW-UP
  }
} else if (wasGenerating && !isGenerating) {
  // Handle stop (with timing check)
  if (timeSinceLastUpdate < 1000) return;
  // ...
} else if (wasGenerating && isGenerating) {
  // Still generating
  this.activeConversation.lastUpdate = Date.now();
} else {
  // Idle
}
```

**Problems:**
- 4 different branches
- Timing checks
- Transition flags
- Complex state tracking

### New Logic (Simple)

```javascript
if (!this.activeConversation) {
  this.activeConversation = { taskId, lastStatus: null };
}

const currentStatus = isGenerating ? "running" : "completed";

if (this.activeConversation.lastStatus !== currentStatus) {
  sendTaskUpdate(taskId, currentStatus, ...);
  this.activeConversation.lastStatus = currentStatus;
}
```

**Benefits:**
- 1 simple check
- No timing needed
- No transition flags
- Clear logic

## Deduplication

The key insight: **We don't need to track transitions, just the current state.**

- Task ID stays the same throughout conversation
- Status changes from "running" ↔ "completed" as needed
- Only send updates when status actually changes

## What Changed

**Removed:**
- `isTransitioning` flag
- `wasGenerating` comparison
- `lastUpdate` timestamp
- `startTime` tracking
- Minimum duration checks (1000ms)
- Transition cooldowns (500ms)

**Kept:**
- Task ID reuse (deduplication)
- Conversation ID tracking
- URL change detection
- Status change detection

**Added:**
- `lastStatus` field (simpler than tracking previous `isGenerating` state)

## Expected Behavior

### Console Output

**First message:**
```
[Agent Inbox DEBUG] NEW CONVERSATION - creating task
[Agent Inbox DEBUG]   Task ID: abc-123
[Agent Inbox DEBUG] Status changed: null → running
Task abc-123: running

[Agent Inbox DEBUG] Status unchanged: running
[Agent Inbox DEBUG] Status unchanged: running

[Agent Inbox DEBUG] Status changed: running → completed
Task abc-123: completed
```

**Follow-up message:**
```
[Agent Inbox DEBUG] Status changed: completed → running
Task abc-123: running

[Agent Inbox DEBUG] Status unchanged: running

[Agent Inbox DEBUG] Status changed: running → completed
Task abc-123: completed
```

### CLI Output

```bash
# First message starts
$ agent-inbox list --all
RUNNING:
  1. [claude.ai] "Hello" (2s ago)

# First message completes
$ agent-inbox list --all
COMPLETED:
  1. [claude.ai] "Hello" (5s ago)

# Follow-up starts (same task!)
$ agent-inbox list --all
RUNNING:
  1. [claude.ai] "Hello" (1s ago)

# Follow-up completes (same task!)
$ agent-inbox list --all
COMPLETED:
  1. [claude.ai] "Hello" (3s ago)
```

## Testing

Test the new logic:

1. **Reload extension:**
   ```
   brave://extensions → Agent Inbox Tracker → Reload
   ```

2. **Open Claude.ai with DevTools**

3. **Send message, watch logs:**
   - Should see: "Status changed: null → running"
   - Then: "Status unchanged: running" (multiple times)
   - Finally: "Status changed: running → completed"

4. **Send follow-up:**
   - Should see: "Status changed: completed → running"
   - Same task ID reused!

5. **Verify CLI:**
   ```bash
   watch -n 1 'agent-inbox list --all'
   ```

## Summary

**Old approach:** Track state transitions with complex logic
**New approach:** Just check current state and update if changed

This is **much simpler** and **more reliable**!
