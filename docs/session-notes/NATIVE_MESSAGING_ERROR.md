# Native Messaging Error - Diagnosis

## The Error You're Seeing

**Location:** `background.js:20`

**Code:**
```javascript
if (message.status === "error") {
  console.error("Native host error:", message.message);
}
```

## What This Means

The error at line 20 is **NOT breaking the extension**. It's just a diagnostic message from the native messaging connection attempt.

### Two Possible Causes:

#### 1. Extension ID Mismatch (Most Likely)

The native messaging manifest expects a specific extension ID, but your extension has a different ID.

**Current manifest:**
```json
{
  "allowed_origins": [
    "chrome-extension://ebcaiiihmjajgghojmdpfjlddjapbgpd/"
  ]
}
```

**To check your actual extension ID:**
1. Go to `brave://extensions`
2. Enable "Developer mode" (toggle in top-right)
3. Look at the "ID" field under "Agent Inbox Tracker"

If it's different from `ebcaiiihmjajgghojmdpfjlddjapbgpd`, the native messaging won't connect.

#### 2. Native Host Not Found

The manifest points to `/home/adlrocha/.local/bin/agent-bridge` but the binary might not be there or might not have correct permissions.

## Impact on Extension

**Good News:** The extension will still work!

Even without native messaging:
- ✅ Content scripts still run
- ✅ Conversation tracking still works
- ✅ State detection still works
- ✅ Debug logging still works

**What doesn't work:**
- ❌ Tasks won't be saved to CLI database
- ❌ `agent-inbox list` won't show web tasks

## Quick Fix

### Option 1: Fix Native Messaging (Recommended)

Run the helper script:
```bash
cd ~/workspace/personal/agent-notifications
./fix-native-messaging.sh
```

This will:
1. Ask for your extension ID
2. Update the native messaging manifest
3. Verify agent-bridge is installed
4. Test the connection

### Option 2: Ignore Error (For Testing)

If you just want to test the extension logic without CLI integration:

1. The error is harmless - extension still tracks state
2. You can see all tracking in the browser DevTools console
3. Just ignore the background.js error for now

## Verify It's Working

**Even with the error, check if tracking works:**

1. Open Claude.ai or Gemini
2. Open DevTools → Console
3. Start a conversation
4. Look for:
   ```
   [Agent Inbox DEBUG] NEW CONVERSATION DETECTED
   [Agent Inbox DEBUG]   Task ID: <uuid>
   Task update sent: running <uuid>
   ```

If you see these logs, **the extension IS working**. The only issue is that tasks aren't being saved to the CLI database.

## Fix Steps (Detailed)

### Step 1: Get Extension ID

```bash
# Open Brave
brave-browser brave://extensions

# Enable Developer Mode
# Copy the ID from "Agent Inbox Tracker"
```

### Step 2: Update Manifest

```bash
cd ~/workspace/personal/agent-notifications

# Edit the manifest (replace YOUR_EXTENSION_ID)
cat > ~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json << 'EOF'
{
  "name": "com.agent_tasks.bridge",
  "description": "Native messaging host for Agent Inbox extension",
  "path": "/usr/local/bin/agent-bridge",
  "type": "stdio",
  "allowed_origins": [
    "chrome-extension://YOUR_EXTENSION_ID/"
  ]
}
EOF
```

### Step 3: Verify Binary

```bash
# Check if agent-bridge exists
ls -la /usr/local/bin/agent-bridge

# If not, install it
cd ~/workspace/personal/agent-notifications
cargo build --release
sudo cp target/release/agent-bridge /usr/local/bin/
```

### Step 4: Test

```bash
# Test agent-bridge directly
echo '{"type":"test"}' | /usr/local/bin/agent-bridge

# Should output: {"status":"ok",...}
```

### Step 5: Reload Extension

1. Go to `brave://extensions`
2. Find "Agent Inbox Tracker"
3. Click reload icon
4. Check "Service Worker" → "Inspect views"
5. Look for: "Connected to native host: com.agent_tasks.bridge"

## Alternative: Pack Extension

To get a stable extension ID:

```bash
cd ~/workspace/personal/agent-notifications

# Pack the extension (creates .crx file with stable ID)
# Go to brave://extensions
# Click "Pack extension"
# Select: extension/ folder
# Note the new ID
# Update manifest with that ID
```

## Current Status

Based on your error, the extension is probably:
- ✅ Loading correctly
- ✅ Running content scripts
- ✅ Tracking conversations
- ❌ Not connecting to native messaging host

**Result:** You can see tracking in the browser console, but not in CLI.

## Quick Test Without Native Messaging

To verify the extension logic works:

```javascript
// In Claude.ai or Gemini console:
diagnoseClaude()  // or diagnoseGemini()

// Send a message and watch for:
[Agent Inbox DEBUG] NEW CONVERSATION DETECTED
[Agent Inbox DEBUG]   Task ID: abc-123-...
```

If you see these logs, the extension's core logic is working perfectly. The native messaging is just a separate connection issue.

## Summary

**The Error:** Line 20 in background.js is a native messaging connection error

**Impact:** Extension tracks state, but doesn't save to CLI database

**Fix:** Run `./fix-native-messaging.sh` to update extension ID in manifest

**Workaround:** Extension still works for testing even with error

Need help? Check the Service Worker console at:
`brave://extensions` → Agent Inbox Tracker → "Inspect views: service worker"
