# Gemini Tracking Troubleshooting Guide

## Quick Diagnosis

When Gemini chats aren't being tracked, follow these steps:

### Step 1: Check if Extension is Loaded

1. **Open Gemini** (gemini.google.com)
2. **Open DevTools** (F12 or Right-click → Inspect)
3. **Go to Console tab**
4. **Look for these messages:**
   ```
   Agent Inbox: Gemini content script loaded
   Agent Inbox: URL = https://gemini.google.com/...
   Agent Inbox: Full location = { href: "...", pathname: "...", ... }
   ```

**If you DON'T see these messages:**
- Extension is not loading on Gemini
- Check manifest.json has correct URL pattern: `https://gemini.google.com/*`
- Check extension is enabled: `brave://extensions`
- Try reloading the extension
- Try hard-refreshing Gemini page (Ctrl+Shift+R)

**If you DO see these messages:**
- Extension is loading correctly
- Continue to Step 2

### Step 2: Run Diagnostic Command

In the Gemini page console, run:
```javascript
diagnoseGemini()
```

This will show:
- Current URL and conversation ID
- Whether generation is detected
- Conversation title
- Active conversations being tracked
- DOM element counts
- Manual check trigger

**Example output:**
```
=== Gemini Tracking Diagnostics ===
URL: https://gemini.google.com/app/abc123
Pathname: /app/abc123
Search:
Conversation ID: app_abc123
Is generating: false
Conversation title: My first Gemini chat
Active conversations: 0
=== DOM Elements Check ===
Buttons: 42
Textareas: 1
Messages: 0
=== Try triggering check manually ===
[Agent Inbox DEBUG] Checking conversation state...
[Agent Inbox DEBUG] Conversation ID: app_abc123
...
```

### Step 3: Check Conversation ID Detection

Look at the diagnostic output:

**If Conversation ID is `null`:**
- Gemini URL pattern doesn't match our patterns
- Need to update URL pattern matching
- **Action:** Share the URL structure with me so I can add the pattern

**Example Gemini URL structures:**
- `https://gemini.google.com/app` (home page - no conversation)
- `https://gemini.google.com/app/abc123` (conversation with ID)
- `https://gemini.google.com/chat/xyz789` (alternative pattern)
- `https://gemini.google.com/?q=search` (search query)

### Step 4: Check Generation Detection

If conversation ID is found but "Is generating" is always `false`:

1. **Start typing a message in Gemini**
2. **Send the message**
3. **While Gemini is responding, run:** `diagnoseGemini()`
4. **Check if "Is generating" changes to `true`**

**If still `false` while generating:**
- DOM selectors for stop button / loading indicators don't match
- **Action:** Run `inspectButtons()` while Gemini is generating
- Look for buttons with text "Stop" or aria-labels containing "Stop"
- Share the button attributes with me

### Step 5: Check Background Connection

1. **Open extension background console:**
   - Go to `brave://extensions`
   - Click "background page" on Agent Inbox Tracker

2. **Look for:**
   ```
   Connected to native host: com.agent_tasks.bridge
   ```

**If you see disconnection errors:**
- Native messaging bridge is not working
- Check: `which agent-bridge`
- Check: `~/.config/BraveSoftware/Brave-Browser/NativeMessagingHosts/com.agent_tasks.bridge.json`
- See `EXTENSION_DEBUGGING.md` for native messaging setup

### Step 6: Manually Trigger Tracking

If everything looks good but tracking doesn't start automatically, try:

```javascript
// In Gemini console:
checkConversationState()
```

This manually triggers the tracking check. If this works but automatic tracking doesn't:
- The polling interval might not be running
- Check for JavaScript errors in console
- Extension might need to be reloaded

## Common Issues and Solutions

### Issue 1: Extension Loads but No Conversation ID

**Symptom:** Console shows "Agent Inbox: Gemini content script loaded" but conversation ID is always `null`

**Cause:** Gemini uses a different URL structure than expected

**Solution:**
1. Open Gemini and start a conversation
2. Note the exact URL structure
3. Share it with me so I can add the pattern
4. Or manually add pattern to `getConversationId()` function

### Issue 2: Conversation Detected but Never Marks as Generating

**Symptom:** Conversation ID found, but "Is generating" always `false` even when Gemini is typing

**Cause:** DOM selectors for stop button / loading indicators don't match Gemini's current UI

**Solution:**
1. While Gemini is generating, run: `inspectButtons()`
2. Find the stop button (look for aria-label with "Stop")
3. Note the exact selector
4. Update `isGenerating()` function in `gemini.js` with the correct selector

### Issue 3: Multiple Conversations Not Tracked Separately

**Symptom:** Opening new Gemini tabs doesn't create separate tasks

**Cause:** Conversation ID extraction might be using a static value

**Solution:**
1. Open multiple Gemini conversations in different tabs
2. In each tab, run: `diagnoseGemini()`
3. Check if conversation IDs are different
4. If they're the same, the URL pattern needs adjustment

### Issue 4: Extension Works in Claude but Not Gemini

**Symptom:** Claude.ai tracking works perfectly, but Gemini doesn't track at all

**Possible Causes:**
1. **Different URL pattern:** Gemini might be at `ai.google.com/gemini` instead of `gemini.google.com`
2. **Permissions:** Extension might not have permission for the actual Gemini URL
3. **CSP (Content Security Policy):** Gemini might block extension scripts

**Solution:**
1. Check exact Gemini URL you're using
2. Update manifest.json if URL is different:
   ```json
   "host_permissions": [
     "https://gemini.google.com/*",
     "https://ai.google.com/*"
   ],
   "content_scripts": [
     {
       "matches": ["https://gemini.google.com/*", "https://ai.google.com/*"],
       "js": ["content-scripts/gemini.js"]
     }
   ]
   ```

## Debug Helpers Reference

### `diagnoseGemini()`
Complete diagnostic output showing:
- URL and conversation ID
- Generation state
- Title detection
- Active conversations
- DOM element counts
- Manual check trigger

### `inspectButtons()`
Lists all buttons on the page with:
- Text content
- aria-label
- CSS classes
- data-testid
- Visibility
- Disabled state

Use this to find the correct selector for the stop button.

### Manual Functions

```javascript
// Check conversation state
checkConversationState()

// Get conversation ID
getConversationId()

// Check if generating
isGenerating()

// Get conversation title
getConversationTitle()

// Check active conversations
console.log(activeConversations)
```

## Reporting Issues

When reporting Gemini tracking issues, please provide:

1. **Exact Gemini URL:**
   ```
   Example: https://gemini.google.com/app/abc123
   ```

2. **Console output from load:**
   ```
   Agent Inbox: Gemini content script loaded
   Agent Inbox: URL = ...
   Agent Inbox: Full location = ...
   ```

3. **Output from `diagnoseGemini()`**

4. **Screenshots of:**
   - Gemini page while generating
   - Browser console showing logs
   - Extension background page (if errors)

5. **Steps to reproduce:**
   - What you did
   - What you expected
   - What actually happened

## Quick Fixes to Try

### Fix 1: Reload Extension
```
brave://extensions → Reload button on Agent Inbox Tracker
```

### Fix 2: Hard Refresh Gemini
```
Ctrl+Shift+R on Gemini page
```

### Fix 3: Clear Extension Storage
```javascript
// In extension background console:
chrome.storage.local.clear(() => console.log('Cleared'))
```

### Fix 4: Reinstall Extension
1. Remove extension
2. Rebuild: `cargo build --release`
3. Copy bridge: `cp target/release/agent-bridge ~/.local/bin/`
4. Load extension again
5. Update native messaging manifest with new extension ID

## Next Steps

If none of the above works:

1. **Share diagnostic output** from `diagnoseGemini()`
2. **Share exact Gemini URL** you're using
3. **Share button output** from `inspectButtons()` while generating
4. **Check if Gemini changed their UI** (Google updates frequently)

I can then update the selectors and URL patterns to match the current Gemini implementation.
