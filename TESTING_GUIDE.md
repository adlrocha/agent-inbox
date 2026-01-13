# Testing Guide - Agent Inbox Extension Fixes

## What Was Fixed

### 1. Race Condition Prevention
- **Problem**: Duplicate tasks created when starting new conversation
- **Fix**: Added `isTransitioning` flag to prevent multiple state changes within 500ms
- **Fix**: Added minimum 1-second duration check before marking as completed

### 2. Follow-up Message Deduplication
- **Problem**: Each follow-up message created a new task
- **Fix**: ConversationTracker now reuses same task ID across conversation

### 3. Code Consolidation
- **Before**: 608 lines with 80% duplication
- **After**: 400 lines with shared ConversationTracker class

## Installation Steps

### 1. Update Binaries

```bash
cd ~/workspace/personal/agent-notifications

# Build release binaries
cargo build --release

# Install (requires sudo)
sudo cp target/release/agent-inbox /usr/local/bin/
sudo cp target/release/agent-bridge /usr/local/bin/

# Verify installation
agent-inbox --version
agent-bridge --version
```

### 2. Reload Browser Extension

**In Brave/Chrome:**
1. Navigate to `brave://extensions` (or `chrome://extensions`)
2. Find "Agent Inbox Tracker"
3. Click the reload icon (circular arrow)
4. Verify no errors in console

**Alternative: Reinstall Extension**
```bash
cd ~/workspace/personal/agent-notifications
./install-extension.sh
```

### 3. Verify Claude Code Hooks

Check that hooks are configured:
```bash
cat ~/.claude/settings.json
```

Should show:
```json
{
  "hooks": {
    "Notification": [
      {
        "matcher": "idle_prompt",
        "hooks": [...]
      }
    ],
    "Stop": [...]
  }
}
```

## Test Cases

### Test 1: New Conversation (No Duplicates)

**Steps:**
1. Open DevTools in Claude.ai or Gemini
2. Go to Console tab
3. Start a new conversation with a message
4. Watch for debug logs: `[Agent Inbox DEBUG]`

**Expected Output:**
```
[Agent Inbox DEBUG] NEW CONVERSATION DETECTED
[Agent Inbox DEBUG]   Task ID: <uuid>
Started tracking conversation: <conv_id> <task_id>
```

**Verify in CLI:**
```bash
agent-inbox list --all
```

Should show **ONE** running task, not duplicates.

**Success Criteria:**
- ✓ Only ONE task appears
- ✓ Status shows "Running"
- ✓ No duplicate entries

### Test 2: Generation Completion

**Steps:**
1. Wait for AI response to complete
2. Watch debug logs for "GENERATION COMPLETED"
3. Check inbox

**Expected Output:**
```
[Agent Inbox DEBUG] GENERATION COMPLETED
[Agent Inbox DEBUG]   Task ID: <uuid>
Completed generation: <conv_id> <task_id>
```

**Verify in CLI:**
```bash
agent-inbox list --all
```

Should show task as "Completed" (within 1 second of generation ending).

**Success Criteria:**
- ✓ Task transitions from Running → Completed
- ✓ Same task ID (not a new task)
- ✓ Timing is reasonable (not too fast, not delayed)

### Test 3: Follow-up Messages (Deduplication)

**Steps:**
1. After first message completes, send a follow-up message
2. Watch debug logs for "FOLLOW-UP MESSAGE - reusing task"
3. Check inbox during and after response

**Expected Output:**
```
[Agent Inbox DEBUG] FOLLOW-UP MESSAGE - reusing task
[Agent Inbox DEBUG]   Task ID: <same-uuid-as-before>
Follow-up message in conversation: <conv_id> <task_id>
```

**Verify in CLI:**
```bash
# While generating
agent-inbox list --all
# Should show: Running (same task as before)

# After completion
agent-inbox list --all
# Should show: Completed (same task, NOT a new entry)
```

**Success Criteria:**
- ✓ Task returns to "Running" state
- ✓ **Same task ID** reused (critical!)
- ✓ Only **ONE** task in inbox for entire conversation
- ✓ Completes successfully after response

### Test 4: Multiple Conversations in Parallel

**Steps:**
1. Open Claude.ai in one tab
2. Open Gemini in another tab
3. Start conversations in both simultaneously
4. Send follow-ups in both

**Verify in CLI:**
```bash
agent-inbox list --all
```

**Expected Output:**
```
RUNNING:
  1. [claude_web] "First conversation" (10s ago)
  2. [gemini_web] "Second conversation" (5s ago)
```

**Success Criteria:**
- ✓ Two separate tasks (different conversations)
- ✓ Each tracked independently
- ✓ No cross-contamination
- ✓ Follow-ups in each reuse their own task ID

### Test 5: Race Condition Prevention

**Steps:**
1. Start a new conversation
2. Immediately check inbox multiple times:
   ```bash
   watch -n 0.5 'agent-inbox list --all'
   ```
3. Monitor for duplicate creation

**Expected Behavior:**
- First check: Shows 1 Running task
- Subsequent checks: Same 1 Running task (no duplicates appear)

**Success Criteria:**
- ✓ Never see duplicate running tasks
- ✓ `isTransitioning` flag prevents double-creation
- ✓ Timing window (1 second minimum) prevents premature completion

### Test 6: Claude Completion Detection (Known Issue)

**Steps:**
1. Start conversation in Claude.ai (not Gemini)
2. Watch when Claude finishes generating
3. Check if task transitions to "Completed"

**Known Issue:**
Claude may not detect completion as reliably as Gemini.

**Debug:**
Open DevTools console and run:
```javascript
diagnoseClaude()
```

Look for:
- `Is generating: false` when Claude is done
- Which selectors matched (stop button, disabled send, etc.)

**If completion not detected:**
The `isGenerating()` function in `claude.js` may need better selectors. Check:
- Stop button disappears when done
- Send button re-enabled
- No streaming indicators visible

### Test 7: Force Clear (Stuck Tasks)

**Steps:**
1. If tasks get stuck, use force clear:
   ```bash
   agent-inbox reset
   ```

2. Type `yes` when prompted

**Expected Output:**
```
This will delete ALL 3 tasks:
  - [claude_web] "Task 1"
  - [gemini_web] "Task 2"
  - [claude_code] "Task 3"

Are you sure you want to delete ALL tasks? (yes/no): yes
✓ Cleared all 3 tasks
```

**Success Criteria:**
- ✓ Shows tasks before deletion
- ✓ Requires "yes" confirmation
- ✓ All tasks removed from database

## Debugging Tools

### Console Diagnostics

**For Claude.ai:**
```javascript
diagnoseClaude()
```

**For Gemini:**
```javascript
diagnoseGemini()
```

**For Button Inspection:**
```javascript
inspectButtons()
```

### Enable/Disable Debug Logging

Edit `extension/content-scripts/shared.js`:
```javascript
// Line 7
const DEBUG = true;  // Set to false to disable verbose logging
```

### Check Extension Errors

1. Go to `brave://extensions`
2. Click "Details" on Agent Inbox Tracker
3. Click "Inspect views: service worker"
4. Check Console for errors

### Verify Database State

```bash
# Show all tasks including completed
agent-inbox list --all

# Show specific task details
agent-inbox show <task-id>

# Clear stuck tasks
agent-inbox reset
```

### Monitor Background Process

Check if native messaging host is running:
```bash
ps aux | grep agent-bridge
```

Check logs:
```bash
# Chrome/Brave native messaging logs
journalctl -f | grep agent-bridge
```

## Common Issues

### Issue 1: Duplicates Still Appearing

**Symptoms:** Two identical tasks when starting conversation

**Debug Steps:**
1. Check console for "Already transitioning, skipping check" messages
2. Verify `isTransitioning` flag is working
3. Check polling interval (should be 2000ms)

**Potential Fix:** Increase transition cooldown:
```javascript
// In shared.js, line 153 and 186
setTimeout(() => {
  this.isTransitioning = false;
}, 1000);  // Increased from 500ms to 1000ms
```

### Issue 2: Claude Not Detecting Completion

**Symptoms:** Claude tasks stay "Running" even after response finishes

**Debug Steps:**
1. Run `diagnoseClaude()` in console
2. Check "Is generating: false" when done
3. Inspect which selectors are matched

**Potential Fix:** Improve selectors in `claude.js`:
```javascript
// Add more robust completion detection
const completionIndicators = [
  '[data-testid="message-complete"]',
  '.response-complete',
  // Add selectors based on diagnoseClaude() output
];
```

### Issue 3: Follow-ups Not Tracked

**Symptoms:** First message tracked, follow-ups ignored

**Debug Steps:**
1. Check console for "FOLLOW-UP MESSAGE - reusing task"
2. Verify `activeConversation` is not null
3. Check `wasGenerating` flag transitions

**Verify Fix:** Should see in console:
```
Was generating: false
Is generating: true
→ FOLLOW-UP MESSAGE - reusing task
```

### Issue 4: Extension Not Loading

**Symptoms:** No debug logs in console

**Debug Steps:**
1. Check `brave://extensions` for errors
2. Verify manifest.json is valid
3. Check content script matches URL pattern

**Reload Extension:**
```bash
cd ~/workspace/personal/agent-notifications
./install-extension.sh
```

## Performance Monitoring

### Check Polling Impact

Monitor CPU usage with:
```bash
top -p $(pgrep brave)
```

Extension polls every 2 seconds:
- `checkConversationState()` - Every 2000ms
- `checkUrlChange()` - Every 1000ms

This should have minimal impact (<1% CPU).

### Database Size

Check task database size:
```bash
du -h ~/.agent-tasks/tasks.db
```

Old tasks auto-clean after 1 hour (configurable in `~/.agent-tasks/config.toml`).

## Success Checklist

After completing all tests, verify:

- [ ] New conversations create single task (no duplicates)
- [ ] Tasks complete within 1-2 seconds of generation ending
- [ ] Follow-up messages reuse same task ID
- [ ] Multiple parallel conversations tracked independently
- [ ] Race condition prevented (no duplicates even during quick polls)
- [ ] Force clear command works with confirmation
- [ ] Claude.ai completion detection works (or issue documented)
- [ ] Gemini completion detection works
- [ ] Extension loads without errors
- [ ] Debug logs show expected state transitions
- [ ] Binaries installed and version matches

## Next Steps

### If Tests Pass:
1. Disable debug logging in production:
   ```javascript
   const DEBUG = false;  // In shared.js
   ```

2. Document any remaining issues in GitHub/FIXES.md

3. Consider additional agents (ChatGPT, Cursor, etc.)

### If Tests Fail:

1. **Capture debug output:**
   ```bash
   # In browser console, save logs
   # Right-click console → "Save as..."
   ```

2. **Check timing issues:**
   - Increase minimum duration check (currently 1000ms)
   - Increase transition cooldown (currently 500ms)

3. **Report findings:**
   - Which test failed
   - Debug logs showing state transitions
   - Expected vs actual behavior

## Reference Documentation

- Full refactor details: `REFACTOR.md`
- Claude hooks setup: `CLAUDE_HOOKS.md`
- Gemini troubleshooting: `GEMINI_TROUBLESHOOTING.md`
- All session fixes: `FIXES.md`
