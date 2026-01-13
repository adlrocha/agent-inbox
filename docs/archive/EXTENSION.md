# Browser Extension for Agent Inbox

This guide covers installing and using the Agent Inbox browser extension to track web-based LLM conversations from Claude.ai and Gemini.

## Overview

The extension consists of three components:

1. **Content Scripts** - Monitor Claude.ai and Gemini web pages
2. **Background Service Worker** - Handles native messaging
3. **Native Messaging Host** (`agent-bridge`) - Writes to agent-inbox database

```
┌─────────────────────────────────────┐
│   Claude.ai or Gemini Web Page      │
│   ┌───────────────────────────┐     │
│   │   Content Script          │     │
│   │   (claude.js/gemini.js)   │     │
│   └───────────┬───────────────┘     │
└───────────────┼─────────────────────┘
                │ chrome.runtime.sendMessage()
        ┌───────▼────────┐
        │  Background    │
        │  Service Worker│
        │  (background.js)│
        └───────┬────────┘
                │ Native Messaging
        ┌───────▼────────┐
        │  agent-bridge  │ (Rust binary)
        │  (stdio host)  │
        └───────┬────────┘
                │ Direct DB access
        ┌───────▼────────┐
        │  SQLite DB     │
        │  tasks.db      │
        └────────────────┘
                │
        ┌───────▼────────┐
        │  agent-inbox   │ CLI
        └────────────────┘
```

## Installation

### Quick Install

```bash
cd /path/to/agent-notifications
./install-extension.sh
```

### Manual Installation

#### 1. Build the Native Messaging Host

```bash
cargo build --release --bin agent-bridge
sudo cp target/release/agent-bridge /usr/local/bin/
```

#### 2. Install Native Messaging Manifest

**For Chrome/Chromium:**
```bash
mkdir -p ~/.config/google-chrome/NativeMessagingHosts
cp extension/com.agent_tasks.bridge.json \
   ~/.config/google-chrome/NativeMessagingHosts/
```

**For Firefox:**
```bash
mkdir -p ~/.mozilla/native-messaging-hosts
cp extension/com.agent_tasks.bridge.json \
   ~/.mozilla/native-messaging-hosts/
```

#### 3. Load Extension in Browser

**Chrome/Chromium:**
1. Navigate to `chrome://extensions`
2. Enable "Developer mode" (toggle in top-right)
3. Click "Load unpacked"
4. Select the `extension/` directory

**Firefox:**
1. Navigate to `about:debugging#/runtime/this-firefox`
2. Click "Load Temporary Add-on"
3. Select `extension/manifest.json`

#### 4. Configure Extension ID

After loading the extension, you'll see an Extension ID (e.g., `abcdefghijklmnopqrstuvwxyz123456`).

Update the native messaging manifest:

```bash
# Edit the manifest file
nano ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json

# Replace this line:
  "allowed_origins": ["chrome-extension://EXTENSION_ID_PLACEHOLDER/"]

# With your actual ID:
  "allowed_origins": ["chrome-extension://abcdefghijklmnop/"]
```

#### 5. Reload Extension

Go back to `chrome://extensions` and click the reload button on the Agent Inbox extension.

## Usage

### Automatic Tracking

Once installed, the extension automatically tracks:

1. **New Conversations** - When you start a new chat on Claude.ai or Gemini
2. **Response Generation** - Marks task as "running" while AI is generating
3. **Completion** - Marks task as "completed" when response finishes

### Checking Tracked Tasks

```bash
# View all tasks (including web)
agent-inbox list --all

# You should see entries like:
# [claude_web] "How do I implement feature X?" (2m ago)
# [gemini_web] "Explain this code..." (5m ago)
```

### Task Details

```bash
agent-inbox show <task-id>

# Shows:
# - Conversation URL
# - Timestamp
# - Duration
# - Conversation ID
```

## Supported Sites

### Claude.ai
- ✅ Conversation tracking
- ✅ Generation detection
- ✅ Completion detection
- ✅ Title extraction

### Gemini (gemini.google.com)
- ✅ Conversation tracking
- ✅ Generation detection
- ✅ Completion detection
- ✅ Title extraction

## Troubleshooting

### Extension Not Connecting

**Symptom:** Tasks not appearing in agent-inbox

**Check 1: Extension Console**
```
1. Go to chrome://extensions
2. Find "Agent Inbox Tracker"
3. Click "background page" (under "Inspect views")
4. Check console for errors
```

Look for:
- `Connected to native host: com.agent_tasks.bridge` ✅ Good
- `Failed to connect to native host` ❌ Problem

**Check 2: Native Messaging Manifest**
```bash
# Verify file exists
ls -la ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json

# Verify extension ID is correct
cat ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json | grep allowed_origins
```

**Check 3: Agent-Bridge Binary**
```bash
# Verify binary exists and is executable
which agent-bridge
ls -la /usr/local/bin/agent-bridge

# Test it manually (should wait for input)
echo '{"type":"task_update","task_id":"test","agent_type":"test","status":"running","title":"test","context":{}}' | \
  /usr/local/bin/agent-bridge
```

### Tasks Not Detected

**Symptom:** Extension connected but conversations not tracked

**Check: Content Script Loading**

Open browser console on Claude.ai or Gemini:
1. Press F12
2. Look for console messages:
   - `Agent Inbox: Claude.ai content script loaded` ✅ Good
   - If missing, content script didn't load ❌

**Fix: Reload Page**
- Sometimes content scripts don't inject on first load
- Refresh the page (Ctrl+R or Cmd+R)

**Check: DOM Selectors**

Claude.ai and Gemini update their UI frequently. The content scripts use selectors to detect elements.

If detection is broken:
1. Open browser console (F12)
2. Check for warnings from content script
3. Selectors may need updating in `extension/content-scripts/claude.js` or `gemini.js`

### Permission Errors

**Symptom:** "Native messaging host has exited"

**Cause:** agent-bridge doesn't have permission to access database

**Fix:**
```bash
# Ensure database directory exists and is writable
ls -la ~/.agent-tasks/
chmod 755 ~/.agent-tasks/
```

### Extension ID Mismatch

**Symptom:** Extension loads but native messaging fails

**Cause:** Manifest has wrong extension ID

**Fix:**
1. Get extension ID from `chrome://extensions`
2. Update manifest:
   ```bash
   nano ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json
   ```
3. Replace `EXTENSION_ID_PLACEHOLDER` with actual ID
4. Reload extension

## Development

### Testing the Extension

**Manual Testing:**

1. Load extension in developer mode
2. Open Claude.ai or Gemini
3. Open browser console (F12)
4. Start a conversation
5. Watch console for messages:
   ```
   Started tracking conversation: abc123 task-uuid
   Completed conversation: abc123 task-uuid
   ```

**Check Native Messaging:**

Extension console shows native messaging status:
```
chrome://extensions -> Agent Inbox -> background page

Console should show:
✅ Connected to native host: com.agent_tasks.bridge
✅ Sent to native host: {type: "task_update", ...}
✅ Received from native host: {status: "ok"}
```

### Modifying Content Scripts

Content scripts are in `extension/content-scripts/`:

**To update Claude.ai detection:**
```javascript
// extension/content-scripts/claude.js

// Modify isGenerating() to detect new UI elements
function isGenerating() {
  const stopButton = document.querySelector('YOUR_NEW_SELECTOR');
  return stopButton !== null;
}
```

**After changes:**
1. Go to `chrome://extensions`
2. Click reload button for Agent Inbox
3. Refresh the Claude.ai/Gemini tab
4. Test again

### Adding New Sites

To track additional AI chat websites:

1. **Add content script:**
   ```bash
   cp extension/content-scripts/claude.js extension/content-scripts/newsite.js
   ```

2. **Update manifest.json:**
   ```json
   {
     "matches": ["https://newsite.com/*"],
     "js": ["content-scripts/newsite.js"],
     "run_at": "document_end"
   }
   ```

3. **Customize selectors in newsite.js:**
   - Update `isGenerating()`
   - Update `getConversationTitle()`
   - Update `getConversationId()`
   - Change `agent_type` to unique identifier

4. **Reload extension and test**

## Architecture Details

### Native Messaging Protocol

Chrome native messaging uses a simple binary protocol:

```
[4 bytes: message length (little-endian)] [JSON message]
```

**Example:**
```
Message: {"type":"task_update",...}
Length: 25 bytes
Wire format: \x19\x00\x00\x00{"type":"task_update",...}
```

The `agent-bridge` binary:
- Reads from stdin using this protocol
- Writes responses to stdout
- Logs to stderr (visible in extension console)

### Security

**Native Messaging Security:**
- Extension must declare `nativeMessaging` permission
- Host manifest must allowlist extension ID
- Host runs with user's full permissions
- No network access for host (just stdio)

**Data Privacy:**
- Conversation titles stored locally in `~/.agent-tasks/tasks.db`
- No data sent to external servers
- All processing happens locally

**Permissions:**
The extension requests:
- `nativeMessaging` - To communicate with agent-bridge
- `storage` - To persist extension state
- `host_permissions` - Access to Claude.ai and Gemini only

### Message Flow

```
1. User sends message in Claude.ai
   ↓
2. Content script detects generation start
   ↓
3. Content script sends message to background:
   chrome.runtime.sendMessage({type: "task_update", status: "running", ...})
   ↓
4. Background worker forwards to native host:
   nativePort.postMessage({type: "task_update", ...})
   ↓
5. agent-bridge receives via stdin
   ↓
6. agent-bridge writes to SQLite
   ↓
7. agent-inbox CLI reads from SQLite
```

## FAQ

**Q: Does the extension track all my conversations?**
A: Yes, it tracks all conversations on Claude.ai and Gemini. Tasks auto-delete after 1 hour.

**Q: Can I disable tracking temporarily?**
A: Yes, disable the extension in `chrome://extensions` or revoke host permissions.

**Q: Does it work in private/incognito mode?**
A: Only if you enable "Allow in Incognito" in extension settings.

**Q: What if Claude/Gemini changes their UI?**
A: Content scripts may need updating. Check console for errors and update selectors.

**Q: Can I use this with other browsers?**
A: Chrome, Chromium, Brave, Edge, and Firefox are supported (all use WebExtensions API).

**Q: Does it slow down the websites?**
A: No significant impact. Content scripts poll DOM every 2 seconds, which is minimal overhead.

**Q: Can I see the source code?**
A: Yes! Everything is in `extension/` directory. Pure JavaScript, no minification.

## Uninstallation

```bash
# Remove extension from browser
# chrome://extensions -> Remove

# Remove native messaging manifest
rm ~/.config/google-chrome/NativeMessagingHosts/com.agent_tasks.bridge.json

# Remove binary
sudo rm /usr/local/bin/agent-bridge

# Keep database (tasks)
# Or remove: rm -rf ~/.agent-tasks/
```

---

**Status**: Phase 3 Complete ✓

Extension is functional and ready for use. UI refinements and additional site support can be added incrementally.
