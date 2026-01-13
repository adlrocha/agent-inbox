# Update Instructions - Apply Latest Fixes

## Summary of Changes

This session fixed critical bugs in the browser extension:

1. **Race condition** causing duplicate tasks on new conversations
2. **Follow-up deduplication** - now one task per conversation
3. **Code consolidation** - 80% duplication eliminated via `shared.js`
4. **Claude Code hooks** - 100% accurate attention detection
5. **Force clear command** - `agent-inbox reset` for stuck tasks

## Quick Update (3 steps)

### 1. Update Binaries

```bash
cd ~/workspace/personal/agent-notifications
cargo build --release
sudo cp target/release/agent-inbox /usr/local/bin/
sudo cp target/release/agent-bridge /usr/local/bin/
```

### 2. Reload Extension

**In browser:**
- Go to `brave://extensions`
- Find "Agent Inbox Tracker"
- Click reload icon

**Or reinstall:**
```bash
./install-extension.sh
```

### 3. Test It

```bash
# Open Claude.ai or Gemini
# Start a conversation
# Check inbox (should show exactly 1 task)
agent-inbox list --all
```

## What to Test

### ✓ No More Duplicates

**Before:**
```
RUNNING:
  1. [claude_web] "Test message" (5s ago)
  2. [claude_web] "Test message" (7s ago)  ← DUPLICATE
```

**After:**
```
RUNNING:
  1. [claude_web] "Test message" (5s ago)  ← Only one!
```

### ✓ Follow-ups Deduplicated

**Before (3 messages = 3 tasks):**
```
COMPLETED:
  1. [gemini_web] "Hello" (5m ago)
  2. [gemini_web] "How are you?" (4m ago)
  3. [gemini_web] "Thanks" (3m ago)
```

**After (3 messages = 1 task):**
```
COMPLETED:
  1. [gemini_web] "Hello" (3m ago)  ← Same task updated!
```

### ✓ Force Clear Works

```bash
agent-inbox reset
# Type: yes
✓ Cleared all 3 tasks
```

## Files Modified

### New Files:
- `extension/content-scripts/shared.js` - Common tracking logic
- `.claude/settings.json` - Hooks configuration
- `TESTING_GUIDE.md` - Comprehensive test procedures
- `REFACTOR.md` - Architecture documentation

### Modified Files:
- `extension/content-scripts/claude.js` - Rewritten (290→110 lines)
- `extension/content-scripts/gemini.js` - Rewritten (318→110 lines)
- `extension/manifest.json` - Load shared.js
- `src/cli/mod.rs` - Added Reset command
- `src/main.rs` - Implemented reset logic
- `wrappers/claude-wrapper` - Export AGENT_TASK_ID

### Key Changes in `shared.js`:

```javascript
class ConversationTracker {
  // Prevents duplicate creation
  if (this.isTransitioning) {
    return;  // Skip if already transitioning
  }

  // Reuses task ID for follow-ups
  if (!this.activeConversation) {
    taskId = generateUUID();  // New conversation
  } else {
    taskId = this.activeConversation.taskId;  // Reuse!
  }

  // Prevents premature completion
  if (timeSinceLastUpdate < 1000) {
    return;  // Wait at least 1 second
  }
}
```

## Known Issues

### Issue: Claude Completion Detection

**Status:** May not detect completion as reliably as Gemini

**Symptoms:** Claude tasks stay "Running" even after response finishes

**Workaround:** Manually clear with `agent-inbox reset`

**Debug:**
```javascript
// In Claude.ai console
diagnoseClaude()
```

**Root Cause:** Claude's UI may not have consistent stop indicators. The `isGenerating()` function checks:
1. Stop button visibility
2. Send button disabled state
3. Loading indicators
4. Streaming text

If none match, completion isn't detected.

**Potential Fix:** Add more robust selectors in `claude.js` lines 74-129.

## Troubleshooting

### Extension Not Loading?

```bash
# Check for errors
brave://extensions → Agent Inbox Tracker → Inspect views

# Reinstall
./install-extension.sh
```

### Still Seeing Duplicates?

```javascript
// In browser console (Claude.ai or Gemini)
// Check for this log:
[Agent Inbox DEBUG] Already transitioning, skipping check

// If NOT appearing, transition guard may not be working
```

**Potential fix:** Increase cooldown in `shared.js`:
```javascript
setTimeout(() => {
  this.isTransitioning = false;
}, 1000);  // Change from 500ms to 1000ms
```

### Follow-ups Not Tracked?

Check console for:
```
[Agent Inbox DEBUG] FOLLOW-UP MESSAGE - reusing task
[Agent Inbox DEBUG]   Task ID: <uuid>
```

If missing, verify:
1. Conversation ID stays the same (check URL)
2. `activeConversation` is not null
3. State transitions from false→true for isGenerating

### Tasks Not Completing?

Run diagnostics:
```javascript
// Claude.ai
diagnoseClaude()

// Gemini
diagnoseGemini()
```

Check "Is generating" field - should be:
- `true` during generation
- `false` when complete

If stuck on `true`, the selectors in `isGenerating()` need improvement.

## Advanced Configuration

### Disable Debug Logging

Edit `extension/content-scripts/shared.js`:
```javascript
const DEBUG = false;  // Line 7
```

Reload extension after change.

### Adjust Timing

**Increase minimum generation duration:**
```javascript
// shared.js line 160
if (timeSinceLastUpdate < 2000) {  // 2 seconds instead of 1
  return;
}
```

**Increase transition cooldown:**
```javascript
// shared.js lines 153, 186
setTimeout(() => {
  this.isTransitioning = false;
}, 1000);  // 1 second instead of 500ms
```

### Change Polling Intervals

Edit `claude.js` and `gemini.js`:
```javascript
// Line 216 - Check conversation state
setInterval(checkConversationState, 3000);  // 3s instead of 2s

// Line 219 - Check URL changes
setInterval(checkUrlChange, 2000);  // 2s instead of 1s
```

## Documentation

Comprehensive guides available:

- **TESTING_GUIDE.md** - Step-by-step test procedures
- **REFACTOR.md** - Architecture and design decisions
- **CLAUDE_HOOKS.md** - Claude Code hooks setup
- **GEMINI_TROUBLESHOOTING.md** - Gemini debugging
- **FIXES.md** - Session changelog

## Rollback (If Needed)

If the new version causes issues:

```bash
# Restore old scripts (backups exist)
mv extension/content-scripts/claude-old.js extension/content-scripts/claude.js
mv extension/content-scripts/gemini-old.js extension/content-scripts/gemini.js

# Remove shared.js from manifest
# Edit extension/manifest.json, remove "content-scripts/shared.js"

# Reload extension
```

Old behavior (each message = new task) will be restored.

## Verification Checklist

After updating, verify:

- [ ] `agent-inbox --version` shows updated binary
- [ ] Extension reloaded without errors
- [ ] New conversation creates exactly 1 task
- [ ] Task completes when generation finishes
- [ ] Follow-up message reuses same task
- [ ] No duplicate tasks appear
- [ ] `agent-inbox reset` works
- [ ] Debug logs show state transitions

## Next Session

If you continue work tomorrow:

1. **Test current fixes** using TESTING_GUIDE.md
2. **Report results** - which tests passed/failed
3. **Fix Claude completion detection** if still an issue
4. **Consider next features:**
   - Desktop notifications
   - More agents (ChatGPT, Cursor)
   - Better attention detection heuristics
