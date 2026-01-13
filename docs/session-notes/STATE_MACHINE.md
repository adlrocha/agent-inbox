# Conversation State Machine - Complete Logic

## State Transitions

The `ConversationTracker.checkState()` function handles 4 possible state transitions:

### State 1: Start/Resume Generation
**Condition:** `wasGenerating=false` AND `isGenerating=true`

**Triggers:**
- New conversation starts (first message)
- Follow-up message in existing conversation (task was completed, now reactivated)

**Actions:**
- **If no activeConversation:** Create new task with new UUID
- **If activeConversation exists:** Reuse existing task ID, send "running" status update

**Code Path:**
```javascript
if (!wasGenerating && isGenerating) {
  this.isTransitioning = true;

  if (!this.activeConversation) {
    // NEW: Create task
    taskId = generateUUID();
    sendTaskUpdate(agentType, taskId, "running", ...);
  } else {
    // FOLLOW-UP: Reuse task
    sendTaskUpdate(agentType, this.activeConversation.taskId, "running", ...);
  }

  this.activeConversation.isGenerating = true;
}
```

**Example:**
```
User sends "Hello"
  wasGenerating: false
  isGenerating:  true
  → NEW CONVERSATION: Create task #abc123, status=running

Agent responds, user sends "Tell me more"
  wasGenerating: false (was completed)
  isGenerating:  true
  → FOLLOW-UP: Reuse task #abc123, status=running
```

---

### State 2: Complete Generation
**Condition:** `wasGenerating=true` AND `isGenerating=false`

**Triggers:**
- AI finishes generating response
- Generation stopped by user
- Generation completed naturally

**Actions:**
- Send "completed" status update
- Set `activeConversation.isGenerating = false`
- Keep conversation in memory for potential follow-ups

**Code Path:**
```javascript
} else if (wasGenerating && !isGenerating) {
  // Check minimum duration (1 second) to avoid false positives
  if (timeSinceLastUpdate < 1000) {
    return; // Too soon, wait
  }

  this.isTransitioning = true;
  sendTaskUpdate(agentType, taskId, "completed", ...);
  this.activeConversation.isGenerating = false;
}
```

**Example:**
```
Claude finishes responding
  wasGenerating: true
  isGenerating:  false
  → COMPLETED: Update task #abc123, status=completed
```

---

### State 3: Continue Generating
**Condition:** `wasGenerating=true` AND `isGenerating=true`

**Triggers:**
- Ongoing generation (multiple polling cycles during single response)
- Long-running generation still in progress

**Actions:**
- Update `lastUpdate` timestamp (keep status fresh)
- **No status update sent** (already "running")
- Just maintain current state

**Code Path:**
```javascript
} else if (isGenerating && wasGenerating) {
  // Still generating - update timestamp to keep status fresh
  debugLog("Generation in progress (already tracked)");
  this.activeConversation.lastUpdate = Date.now();
}
```

**Example:**
```
Poll #1: wasGenerating=true, isGenerating=true → Update timestamp
Poll #2: wasGenerating=true, isGenerating=true → Update timestamp
Poll #3: wasGenerating=true, isGenerating=true → Update timestamp
  (No status updates sent, task stays "running")
```

**Why This Matters:**
The `lastUpdate` timestamp is checked before marking as completed (State 2). Without updating it here, long-running generations would be marked complete prematurely.

---

### State 4: Idle
**Condition:** `wasGenerating=false` AND `isGenerating=false`

**Triggers:**
- User viewing completed conversation (no activity)
- Browsing conversation history
- Between messages (waiting for user input)

**Actions:**
- No action taken
- Just log for debugging

**Code Path:**
```javascript
} else {
  debugLog("No generation activity");
}
```

**Example:**
```
User reading previous conversation
  wasGenerating: false
  isGenerating:  false
  → IDLE: No action
```

---

## Critical Edge Cases Handled

### Edge Case 1: Race Condition (Duplicate Detection)
**Problem:** Polling interval (2 seconds) could trigger multiple state changes before first completes.

**Solution:** `isTransitioning` flag
```javascript
if (this.isTransitioning) {
  debugLog("Already transitioning, skipping check");
  return;
}
```

Prevents duplicate task creation during the 500ms transition window.

---

### Edge Case 2: False Positive Completion
**Problem:** Brief moments where `isGenerating=false` detected during state updates.

**Solution:** Minimum duration check
```javascript
const timeSinceLastUpdate = Date.now() - this.activeConversation.lastUpdate;
if (timeSinceLastUpdate < 1000) {
  debugLog("Too soon after generation start, waiting...");
  return;
}
```

Requires at least 1 second of generation before allowing completion.

---

### Edge Case 3: Conversation Navigation
**Problem:** User navigates to different conversation (URL changes).

**Solution:** Reset tracking
```javascript
if (this.activeConversation && this.activeConversation.conversationId !== conversationId) {
  debugLog("Different conversation detected - resetting");
  this.activeConversation = null;
}
```

Next generation in new conversation creates new task.

---

### Edge Case 4: Tab Close During Generation
**Problem:** User closes tab while agent is still generating.

**Solution:** `beforeunload` handler
```javascript
window.addEventListener("beforeunload", () => {
  tracker.handleUnload();
});
```

Marks task as completed with `reason: "tab_closed"`.

---

## State Machine Diagram

```
┌─────────────┐
│   IDLE      │ wasGen=F, isGen=F
│ (no action) │
└─────┬───────┘
      │
      │ User sends message
      ▼
┌─────────────┐
│   START     │ wasGen=F, isGen=T
│ (create or  │───────┐
│  resume)    │       │
└─────┬───────┘       │
      │               │
      │ Generating... │ Multiple polls
      ▼               │
┌─────────────┐       │
│ GENERATING  │ wasGen=T, isGen=T
│ (continue)  │◄──────┘
└─────┬───────┘
      │
      │ Generation finishes
      ▼
┌─────────────┐
│ COMPLETED   │ wasGen=T, isGen=F
│ (mark done) │
└─────┬───────┘
      │
      │ Wait for follow-up...
      ▼
┌─────────────┐
│   IDLE      │ wasGen=F, isGen=F
│             │
└─────────────┘
      │
      │ User sends follow-up
      └──────► (back to START, reuse task)
```

---

## Polling Behavior

**Intervals:**
- State check: Every 2000ms (`checkConversationState`)
- URL check: Every 1000ms (`checkUrlChange`)

**Example Timeline:**
```
T=0ms    User sends message
T=0ms    State: wasGen=F → isGen=T (START, task created)
T=500ms  Transition flag cleared
T=2000ms Poll: wasGen=T, isGen=T (CONTINUE, update timestamp)
T=4000ms Poll: wasGen=T, isGen=T (CONTINUE, update timestamp)
T=6000ms Poll: wasGen=T, isGen=T (CONTINUE, update timestamp)
T=8000ms Generation finishes
T=8000ms Poll: wasGen=T, isGen=F (COMPLETED, mark done)
T=8500ms Transition flag cleared
T=10000ms Poll: wasGen=F, isGen=F (IDLE)
```

---

## Debug Output

**State 1 (Start/Resume):**
```
[Agent Inbox DEBUG] Checking conversation state...
[Agent Inbox DEBUG]   Is generating NOW: true
[Agent Inbox DEBUG]   Was generating BEFORE: false
[Agent Inbox DEBUG] FOLLOW-UP MESSAGE - reusing task
[Agent Inbox DEBUG]   Task ID: abc-123
[Agent Inbox DEBUG]   Previous state: isGenerating = false
Task update sent: running abc-123
```

**State 2 (Complete):**
```
[Agent Inbox DEBUG] Checking conversation state...
[Agent Inbox DEBUG]   Is generating NOW: false
[Agent Inbox DEBUG]   Was generating BEFORE: true
[Agent Inbox DEBUG] GENERATION COMPLETED
[Agent Inbox DEBUG]   Task ID: abc-123
Task update sent: completed abc-123
```

**State 3 (Continue):**
```
[Agent Inbox DEBUG] Checking conversation state...
[Agent Inbox DEBUG]   Is generating NOW: true
[Agent Inbox DEBUG]   Was generating BEFORE: true
[Agent Inbox DEBUG] Generation in progress (already tracked)
```

**State 4 (Idle):**
```
[Agent Inbox DEBUG] Checking conversation state...
[Agent Inbox DEBUG]   Is generating NOW: false
[Agent Inbox DEBUG]   Was generating BEFORE: false
[Agent Inbox DEBUG] No generation activity
```

---

## Testing Each State

### Test State 1: Start/Resume
```bash
# New conversation
1. Open Gemini/Claude
2. Send "Hello"
3. Watch console: "NEW CONVERSATION DETECTED"
4. Check: agent-inbox list --all → 1 running task

# Follow-up
1. Wait for completion
2. Send "Tell me more"
3. Watch console: "FOLLOW-UP MESSAGE - reusing task"
4. Check: agent-inbox list --all → same task, running again
```

### Test State 2: Complete
```bash
1. Start conversation
2. Wait for generation to finish
3. Watch console: "GENERATION COMPLETED"
4. Check: agent-inbox list --all → 1 completed task
```

### Test State 3: Continue
```bash
1. Start conversation (long response)
2. Watch console during generation
3. Should see: "Generation in progress (already tracked)" (multiple times)
4. Task stays "running" throughout
```

### Test State 4: Idle
```bash
1. Open completed conversation
2. Watch console (every 2 seconds)
3. Should see: "No generation activity"
4. No status updates sent
```

---

## Summary

The state machine correctly handles:
✅ New conversations (create task)
✅ Follow-up messages (reuse task, update to running)
✅ Ongoing generation (maintain running status)
✅ Completion detection (mark as completed)
✅ Idle state (no action)
✅ Race conditions (transition flag)
✅ False positives (minimum duration)
✅ Navigation (reset on URL change)
✅ Tab close (cleanup handler)

All logic is shared between Claude and Gemini via `shared.js`!
