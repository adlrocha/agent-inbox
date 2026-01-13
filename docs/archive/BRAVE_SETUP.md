# Setting Up Agent Inbox Extension for Brave

## Quick Setup

```bash
cd /path/to/agent-notifications
./setup-brave-extension.sh
```

Then follow the steps below to configure your extension ID.

---

## What You're Setting Up

```
┌─────────────────────────────────────────────────┐
│  Brave Browser                                  │
│  ┌───────────────────────────────┐              │
│  │  Agent Inbox Extension        │              │
│  │  (needs permission to talk    │              │
│  │   to programs on your system) │              │
│  └───────────┬───────────────────┘              │
└──────────────┼──────────────────────────────────┘
               │
               │ "Is this extension allowed?"
               ▼
    ┌──────────────────────────────┐
    │  NativeMessagingHosts/       │
    │  com.agent_tasks.bridge.json │  ← Configuration file
    │                              │
    │  Says: "Yes, extension ID    │
    │   xyz123 can talk to         │
    │   /usr/local/bin/agent-bridge"│
    └──────────┬───────────────────┘
               │
               │ "OK, allowed!"
               ▼
    ┌──────────────────────────────┐
    │  agent-bridge                │  ← Your Rust program
    │  (writes to database)        │
    └──────────────────────────────┘
```

## Step-by-Step Guide

### Step 1: Run Setup Script

```bash
cd /path/to/agent-notifications
./setup-brave-extension.sh
```

This creates:
- `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json`
- Installs `agent-bridge` to `/usr/local/bin/`

### Step 2: Load Extension in Brave

1. Open Brave and go to: `brave://extensions`

2. Enable **Developer mode** (toggle switch in top-right corner)

3. Click **"Load unpacked"** button

4. Navigate to and select:
   ```
   /home/adlrocha/workspace/personal/agent-notifications/extension
   ```

5. The extension should now appear in your list!

### Step 3: Copy Extension ID

After loading, you'll see something like this:

```
┌─────────────────────────────────────────┐
│ Agent Inbox Tracker                     │
│ ID: abcdefghijklmnopqrstuvwxyz123456   │ ← Copy this!
│ [Remove] [Reload] [Errors]             │
└─────────────────────────────────────────┘
```

**Copy the ID** (that long string of letters/numbers)

### Step 4: Configure the ID in Manifest

Now you need to tell the manifest file which extension is allowed to connect.

**Option A: Using sed (quick):**
```bash
# Replace YOUR_ID_HERE with the ID you copied
sed -i 's/EXTENSION_ID_PLACEHOLDER/abcdefghijklmnopqrstuvwxyz123456/' \
  ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json
```

**Option B: Edit manually:**
```bash
# Open the file
nano ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json

# Find this line:
  "allowed_origins": ["chrome-extension://EXTENSION_ID_PLACEHOLDER/"]

# Change to (with YOUR actual ID):
  "allowed_origins": ["chrome-extension://abcdefghijklmnopqrstuvwxyz123456/"]

# Save (Ctrl+O, Enter, Ctrl+X)
```

### Step 5: Reload Extension

1. Go back to `brave://extensions`
2. Find "Agent Inbox Tracker"
3. Click the **circular arrow** (reload button)

### Step 6: Verify It's Working

1. On the extension card, click **"background page"**
   - This opens the extension's console

2. Look for this message:
   ```
   ✅ Agent Inbox background service worker initialized
   ✅ Connected to native host: com.agent_tasks.bridge
   ```

3. If you see **"Specified native messaging host not found"**:
   - Go back to Step 4 and make sure you updated the Extension ID
   - The ID in the manifest MUST match the ID shown in brave://extensions

### Step 7: Test It!

1. **Open Claude.ai** in Brave and start a conversation

2. **Check agent-inbox:**
   ```bash
   agent-inbox list --all
   ```

3. You should see your conversation listed!
   ```
   [claude_web] "Your conversation title..." (2m ago)
   ```

---

## Troubleshooting

### Issue: "Specified native messaging host not found"

**Cause:** Extension ID not configured in manifest

**Fix:**
```bash
# 1. Check what ID is in the manifest:
cat ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json | grep EXTENSION_ID

# 2. If you see "EXTENSION_ID_PLACEHOLDER", you need to update it
# 3. Get your actual ID from brave://extensions
# 4. Run the sed command from Step 4 above
```

### Issue: "Native host has exited"

**Cause:** agent-bridge binary not found or not executable

**Fix:**
```bash
# Check if installed
which agent-bridge
# Should show: /usr/local/bin/agent-bridge

# If not found, reinstall:
cd /path/to/agent-notifications
cargo build --release --bin agent-bridge
sudo cp target/release/agent-bridge /usr/local/bin/
```

### Issue: Extension loads but conversations aren't tracked

**Cause:** Content script not loading or native messaging not connected

**Fix:**
1. Open Claude.ai
2. Press **F12** to open console
3. Look for: `Agent Inbox: Claude.ai content script loaded`
4. If missing, refresh the page
5. Check background console for native messaging connection

### Issue: Can't find extension directory

The extension is in your cloned repo:
```bash
cd /home/adlrocha/workspace/personal/agent-notifications/extension
pwd  # This is the path to select when loading unpacked
```

---

## File Locations Reference

| Item | Location |
|------|----------|
| **Extension source** | `~/workspace/personal/agent-notifications/extension/` |
| **Native messaging manifest** | `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json` |
| **agent-bridge binary** | `/usr/local/bin/agent-bridge` |
| **Database** | `~/.agent-tasks/tasks.db` |

---

## Understanding the Manifest File

Here's what's in the manifest file:

```json
{
  "name": "com.agent_tasks.bridge",           ← Name of the native messaging host
  "description": "...",                       ← Description
  "path": "/usr/local/bin/agent-bridge",     ← Location of your program
  "type": "stdio",                           ← Communication method (stdin/stdout)
  "allowed_origins": [
    "chrome-extension://YOUR_ID_HERE/"       ← Only this extension can connect
  ]
}
```

**Security Note:** The `allowed_origins` is a security feature. Only extensions with the exact ID you specify can talk to agent-bridge. This prevents malicious extensions from accessing your data.

---

## Why This Setup?

You might wonder why all this configuration is needed:

1. **Security:** Browsers don't let extensions run arbitrary programs on your system
2. **Permission System:** You explicitly grant permission via the manifest file
3. **Extension Identity:** The Extension ID ensures only YOUR extension can connect
4. **Isolation:** Each extension and native app must be explicitly paired

This is by design to keep your system safe!

---

## Quick Commands

```bash
# Check if manifest exists
ls -la ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/

# View manifest content
cat ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json

# Test agent-bridge manually
echo '{"type":"task_update","task_id":"test","agent_type":"test","status":"running","title":"test","context":{}}' | agent-bridge

# View all tracked tasks
agent-inbox list --all

# Open extension console
# brave://extensions → Agent Inbox → "background page"
```

---

## Next Steps

Once the extension is working:

1. Use Claude.ai or Gemini normally
2. Check `agent-inbox` to see your conversations
3. Tasks auto-track and auto-clean after 1 hour
4. Enjoy unified tracking across CLI and web agents!
