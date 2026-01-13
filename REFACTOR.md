# Extension Refactor - Deduplication & Shared Code

## Problem

The original implementation had two issues:

1. **Duplication Problem:** Each follow-up message in Gemini/Claude created a NEW task with a new task ID, leading to inbox spam
2. **Code Duplication:** Claude and Gemini scripts had 90% identical code, making maintenance difficult

## Solution

### 1. Conversation Deduplication

**Old Behavior:**
```
User: "Hello"
→ Creates Task #1 (running)
→ Task #1 completes

User: "How are you?"
→ Creates Task #2 (running)  ❌ NEW TASK
→ Task #2 completes

Result: 2 tasks in inbox for same conversation
```

**New Behavior:**
```
User: "Hello"
→ Creates Task #1 (running)
→ Task #1 completes

User: "How are you?"
→ REUSES Task #1 (back to running)  ✅ SAME TASK
→ Task #1 completes again

Result: 1 task in inbox, updated with latest activity
```

### 2. Shared Code Architecture

Created `shared.js` with common functionality:

**Before:**
- `claude.js`: 290 lines
- `gemini.js`: 318 lines
- Total: ~608 lines
- Duplication: ~80%

**After:**
- `shared.js`: 180 lines (common code)
- `claude.js`: 110 lines (Claude-specific)
- `gemini.js`: 110 lines (Gemini-specific)
- Total: ~400 lines
- Duplication: ~0%

## Architecture

### ConversationTracker Class

The key is the `ConversationTracker` class in `shared.js`:

```javascript
class ConversationTracker {
  constructor(agentType) {
    this.agentType = agentType;
    this.activeConversation = null;  // ONE conversation per tab
  }

  checkState(conversationId, isGenerating, title) {
    // Logic for deduplication:
    // - First message: Create new task
    // - Follow-up messages: Reuse existing task
    // - New conversation: Reset and create new task
  }
}
```

### Key Logic

**State Transitions:**

1. **New Conversation:**
   ```
   conversationId exists, no activeConversation
   → Create new task
   → Set activeConversation with task ID
   ```

2. **Follow-up Message:**
   ```
   conversationId matches activeConversation
   isGenerating true, was false
   → Update SAME task to "running"
   → Keep same task ID
   ```

3. **Generation Complete:**
   ```
   conversationId matches activeConversation
   isGenerating false, was true
   → Update task to "completed"
   → Keep conversation in memory for follow-ups
   ```

4. **Navigate to Different Conversation:**
   ```
   conversationId different from activeConversation
   → Reset activeConversation
   → Next generation creates NEW task
   ```

### Shared vs. Specific Code

**Shared (shared.js):**
- ConversationTracker class
- UUID generation
- Task update sending
- Debug logging
- URL change detection
- Page unload handling
- Button inspection helper

**Claude-Specific (claude.js):**
- getConversationId() - Claude URL patterns
- getConversationTitle() - Claude DOM selectors
- isGenerating() - Claude button/indicator selectors
- isWaitingForUserInput() - Claude-specific prompt detection

**Gemini-Specific (gemini.js):**
- getConversationId() - Gemini URL patterns (more complex)
- getConversationTitle() - Gemini DOM selectors
- isGenerating() - Gemini button/indicator selectors

## Benefits

### 1. No More Inbox Spam

**Before:**
```bash
$ agent-inbox list --all
[gemini_web] "Hello" - Completed (5 mins ago)
[gemini_web] "How are you?" - Completed (4 mins ago)
[gemini_web] "Tell me more" - Completed (3 mins ago)
[gemini_web] "Thanks" - Completed (2 mins ago)
# 4 tasks for 1 conversation!
```

**After:**
```bash
$ agent-inbox list --all
[gemini_web] "Hello" - Completed (2 mins ago)
# 1 task, updated throughout conversation
```

### 2. Consistent Behavior

Both Claude and Gemini now have identical tracking behavior:
- Same deduplication logic
- Same state transitions
- Same task lifecycle

### 3. Easier Maintenance

To add a new agent (e.g., ChatGPT):
1. Import `shared.js`
2. Implement 3 functions:
   - `getConversationId()`
   - `getConversationTitle()`
   - `isGenerating()`
3. Call `tracker.checkState()`

That's it! All deduplication logic is handled automatically.

### 4. Better Debug Tools

Unified diagnostic helpers:
- `diagnoseClaude()` for Claude.ai
- `diagnoseGemini()` for Gemini
- `inspectButtons()` for both

All use the same underlying tracker, so debugging is consistent.

## Implementation Details

### Task Lifecycle with Deduplication

```
1. User opens Gemini
   → activeConversation = null

2. User sends "Hello"
   → isGenerating: false → true
   → Create Task #abc123 (running)
   → activeConversation = { conversationId, taskId: "abc123", isGenerating: true }

3. Gemini responds
   → isGenerating: true → false
   → Update Task #abc123 (completed)
   → activeConversation = { conversationId, taskId: "abc123", isGenerating: false }

4. User sends "How are you?"
   → isGenerating: false → true
   → REUSE Task #abc123 (back to running)  ← KEY: Same task ID!
   → activeConversation = { conversationId, taskId: "abc123", isGenerating: true }

5. Gemini responds
   → isGenerating: true → false
   → Update Task #abc123 (completed)
   → activeConversation = { conversationId, taskId: "abc123", isGenerating: false }

6. User closes tab
   → beforeunload event
   → If isGenerating: Mark Task #abc123 as completed (reason: "tab_closed")
```

### Conversation ID Stability

Critical for deduplication to work:
- Claude: `/chat/abc-123` (stable, from URL)
- Gemini: `/app/xyz-789` or synthetic from pathname (stable, from URL)

As long as conversation ID doesn't change, we reuse the same task.

### URL Navigation Detection

When user navigates to a different conversation:
```javascript
checkUrlChange() {
  if (currentUrl !== lastUrl) {
    // Different conversation - reset tracking
    this.activeConversation = null;
    // Next generation will create NEW task
  }
}
```

This ensures:
- Opening new conversation → New task
- Navigating back to old conversation → New task (we don't keep history across navigations)

## Files Changed

### New Files:
- `extension/content-scripts/shared.js` - Common functionality

### Modified Files:
- `extension/manifest.json` - Include shared.js before claude.js/gemini.js
- `extension/content-scripts/claude.js` - Rewritten to use ConversationTracker
- `extension/content-scripts/gemini.js` - Rewritten to use ConversationTracker

### Backup Files:
- `extension/content-scripts/claude-old.js` - Original Claude implementation
- `extension/content-scripts/gemini-old.js` - Original Gemini implementation

## Testing

### Test Deduplication

1. Open Gemini with DevTools
2. Send 3 messages in same conversation
3. Check inbox:
   ```bash
   agent-inbox list --all
   ```
4. Should see **ONE task**, not three

### Test State Transitions

```bash
# After first message starts
agent-inbox list --all
# → Should show: [gemini_web] "..." - Running

# After first message completes
agent-inbox list --all
# → Should show: [gemini_web] "..." - Completed

# After second message starts
agent-inbox list --all
# → Should show: [gemini_web] "..." - Running (same task!)

# After second message completes
agent-inbox list --all
# → Should show: [gemini_web] "..." - Completed (same task!)
```

### Test Conversation Separation

1. Start conversation in Tab 1
2. Open new conversation in Tab 2
3. Should create 2 separate tasks (different conversations)

### Test Navigation

1. Start conversation A
2. Navigate to conversation B (different URL)
3. Should create NEW task (even though tab didn't close)

## Migration Notes

**Breaking Change:** Task IDs are now stable across follow-ups

**Before:** Each message had unique task ID
```
Task abc-123: "First message"
Task def-456: "Second message"
Task ghi-789: "Third message"
```

**After:** All messages share one task ID
```
Task abc-123: "First message" (updated 3 times)
```

**Impact:**
- Existing tasks in database: No impact (old tasks stay as-is)
- New conversations: Use new deduplication logic
- No database migration needed

**User Experience:**
- Less inbox clutter ✓
- Clearer conversation tracking ✓
- Easier to see active conversations ✓

## Rollback Plan

If deduplication causes issues:

```bash
# Restore old scripts
mv extension/content-scripts/claude-old.js extension/content-scripts/claude.js
mv extension/content-scripts/gemini-old.js extension/content-scripts/gemini.js

# Remove shared.js from manifest
# Edit extension/manifest.json, remove "content-scripts/shared.js" from js arrays

# Reload extension
```

Old behavior will be restored (each message creates new task).
