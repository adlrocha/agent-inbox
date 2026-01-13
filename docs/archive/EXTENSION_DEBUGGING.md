# Browser Extension Debugging Guide

## Issue: Extension Not Tracking Conversations

If the extension is loaded but not creating inbox entries, follow these debugging steps:

### Step 1: Verify Extension is Loaded and Connected

1. **Open Brave Extensions Page:**
   ```
   brave://extensions
   ```

2. **Find "Agent Inbox Tracker"** - should show as enabled

3. **Click "background page"** to open the service worker console

4. **Check for connection message:**
   ```
   ✅ Connected to native host: com.agent_tasks.bridge
   ```

   If you see an error instead:
   - Check that extension ID matches manifest (see BRAVE_SETUP.md Step 4)
   - Verify agent-bridge is installed: `which agent-bridge`
   - Check manifest file: `cat ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json`

### Step 2: Check Content Script Loading

1. **Open Claude.ai** in a new tab: https://claude.ai

2. **Open DevTools** (F12 or Right-click → Inspect)

3. **Go to Console tab**

4. **Look for content script message:**
   ```
   Agent Inbox: Claude.ai content script loaded
   Agent Inbox: Initializing Claude.ai monitoring
   ```

   If you DON'T see these messages:
   - The content script is not loading
   - Check if URL matches manifest: `https://claude.ai/*`
   - Try reloading the extension (brave://extensions → reload)
   - Try hard-refreshing the page (Ctrl+Shift+R)

### Step 3: Test Conversation Detection

1. **With DevTools console still open on Claude.ai**

2. **Start a new conversation** or **send a message**

3. **Watch for log messages:**
   ```
   URL changed from ... to ...
   Started tracking conversation: <conv-id> <task-id>
   Task update sent: running <task-id>
   ```

4. **Check background console** (from Step 1) for:
   ```
   Received from content script: {type: "task_update", ...}
   Sent to native host: {type: "task_update", ...}
   ```

### Step 4: Verify Database Entry

After starting a conversation, check if task was saved:

```bash
agent-inbox list --all
```

You should see an entry like:
```
[claude_web] "Your conversation title..." (just now)
```

If you don't see it, the native messaging bridge might not be writing to DB.

### Step 5: Test Native Messaging Manually

Test agent-bridge manually to ensure it works:

```bash
echo '{"type":"task_update","task_id":"test-123","agent_type":"claude_web","status":"running","title":"Test conversation","context":{"url":"https://claude.ai/chat/test","timestamp":1234567890}}' | ~/.local/bin/agent-bridge
```

Check if task appears:
```bash
agent-inbox list --all
```

If this works but extension doesn't, the issue is in the extension-to-native communication.

## Common Issues and Fixes

### Issue: Content Script Loads but No Messages Sent

**Symptom:** You see "content script loaded" but no "Started tracking conversation" messages.

**Possible Causes:**
1. DOM selectors in `claude.js` don't match Claude's current UI
2. Claude's UI has changed and detection logic is outdated

**Fix:** Update DOM selectors in `extension/content-scripts/claude.js`

### Issue: Messages Sent but Native Host Not Receiving

**Symptom:** Background console shows "Sent to native host" but agent-inbox doesn't show tasks.

**Possible Causes:**
1. Native messaging format is incorrect
2. agent-bridge is crashing silently
3. Database path is wrong

**Fix:**
```bash
# Check agent-bridge logs
# agent-bridge writes to stderr
echo '{"type":"task_update","task_id":"test","agent_type":"test","status":"running","title":"test","context":{}}' | ~/.local/bin/agent-bridge 2>&1
```

### Issue: Extension Disconnects Immediately

**Symptom:** Background console shows "Disconnected from native host" right after connecting.

**Possible Causes:**
1. agent-bridge binary exits immediately (likely crash or missing dependency)
2. Permission issues with binary

**Fix:**
```bash
# Check if binary is executable
ls -la ~/.local/bin/agent-bridge

# Should show: -rwxr-xr-x

# If not:
chmod +x ~/.local/bin/agent-bridge

# Test manually
echo '{"test": true}' | ~/.local/bin/agent-bridge 2>&1
```

## Debugging Tools

### Enable Verbose Logging in Content Script

Edit `extension/content-scripts/claude.js` and add more console.log statements:

```javascript
function checkConversationState() {
  console.log("[DEBUG] Checking conversation state...");
  const conversationId = getConversationId();
  console.log("[DEBUG] Conversation ID:", conversationId);

  const isActive = isGenerating();
  console.log("[DEBUG] Is generating:", isActive);

  // ... rest of function
}
```

### Monitor Native Messaging Traffic

In background.js, log all messages:

```javascript
function sendToNativeHost(message) {
  console.log("[SEND]", JSON.stringify(message, null, 2));
  // ... rest of function
}

nativePort.onMessage.addListener((message) => {
  console.log("[RECV]", JSON.stringify(message, null, 2));
  // ... rest of handler
});
```

### Check Process Tree

When extension should be tracking, verify processes:

```bash
# Find agent-bridge processes
ps aux | grep agent-bridge

# Should be running when extension communicates
```

## Next Steps

If none of the above works:

1. **Share console logs** from both:
   - Background console (brave://extensions → background page)
   - Page console (F12 on claude.ai)

2. **Test with a simple message:**
   - Open background console
   - Run this JavaScript:
   ```javascript
   chrome.runtime.sendMessage({
     type: "task_update",
     task_id: "manual-test-123",
     agent_type: "claude_web",
     status: "running",
     title: "Manual test",
     context: {url: "https://claude.ai/test"}
   });
   ```
   - Check if it appears in `agent-inbox list --all`

3. **Verify extension permissions:**
   - brave://extensions → Agent Inbox Tracker → Details
   - Check "Site access" shows: "On specific sites: claude.ai, gemini.google.com"
   - If not, click and grant permissions

## Expected Flow

Here's what SHOULD happen when working correctly:

1. **User opens Claude.ai** → Content script loads → Logs "content script loaded"
2. **User starts conversation** → Content script detects generation → Logs "Started tracking"
3. **Content script sends message** → Background receives → Logs "Received from content script"
4. **Background forwards to native host** → Logs "Sent to native host"
5. **agent-bridge receives** → Writes to DB → Logs to stderr "Processed task_update"
6. **User runs agent-inbox** → Shows task

Any break in this chain means that step is failing.
