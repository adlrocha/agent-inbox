# Quick Reference

## 📂 Where to Find Things

### Need to...

**Start using the system?**
→ `README.md`

**Test if it works?**
→ `TESTING_GUIDE.md`

**Update after pulling changes?**
→ `UPDATE_INSTRUCTIONS.md`

**Understand the architecture?**
→ `REFACTOR.md`

**Fix native messaging error?**
→ `docs/session-notes/NATIVE_MESSAGING_ERROR.md`

**Debug Claude/Gemini?**
→ `GEMINI_TROUBLESHOOTING.md` or `CLAUDE_HOOKS.md`

**Understand state machine?**
→ `docs/session-notes/STATE_MACHINE.md`

**See what was fixed?**
→ `FIXES.md` (summary) or `docs/session-notes/SESSION_SUMMARY.md` (detailed)

**Find old notes?**
→ `docs/archive/`

**Find old code?**
→ `old-files/`

---

## 🚀 Common Commands

### Install/Update
```bash
# Full install
./install.sh

# Extension only
./install-extension.sh

# Wrappers only
./setup-wrappers.sh

# Fix native messaging
./fix-native-messaging.sh
```

### Build
```bash
# Build release binaries
cargo build --release

# Install binaries
sudo cp target/release/{agent-inbox,agent-bridge} /usr/local/bin/
```

### CLI Usage
```bash
# Show tasks needing attention
agent-inbox

# Show all tasks
agent-inbox list --all

# Show specific task
agent-inbox show <task-id>

# Clear all tasks (with confirmation)
agent-inbox reset

# Watch in real-time
watch -n 1 'agent-inbox list --all'
```

### Extension
```bash
# Reload extension
brave://extensions → Agent Inbox Tracker → Reload

# Debug in console
diagnoseClaude()    # In Claude.ai
diagnoseGemini()    # In Gemini
inspectButtons()    # In either
```

---

## 🔍 Debug Checklist

### Extension Not Tracking?

1. **Check extension is loaded:**
   - Go to `brave://extensions`
   - "Agent Inbox Tracker" should be enabled

2. **Check console logs:**
   - Open Claude.ai or Gemini
   - Open DevTools → Console
   - Look for: `[Agent Inbox DEBUG]`

3. **Run diagnostics:**
   ```javascript
   diagnoseClaude()    // or diagnoseGemini()
   ```

4. **Verify state detection:**
   - Send message
   - Watch for: "NEW CONVERSATION DETECTED"
   - Should see task ID and "running" status

### Native Messaging Error?

1. **Check extension ID:**
   - Go to `brave://extensions`
   - Enable "Developer mode"
   - Copy ID from "Agent Inbox Tracker"

2. **Fix manifest:**
   ```bash
   ./fix-native-messaging.sh
   # Enter extension ID when prompted
   ```

3. **Reload extension:**
   - `brave://extensions` → Reload

### Tasks Not Completing?

1. **Check if generation detection works:**
   ```javascript
   // In browser console while generating
   diagnoseClaude()
   // Check "Is generating: true"
   ```

2. **After generation finishes:**
   ```javascript
   diagnoseClaude()
   // Check "Is generating: false"
   ```

3. **If stuck on "true":**
   - See `docs/session-notes/CLAUDE_DETECTION_TEST.md`
   - Need better selectors in `isGenerating()`

### Follow-ups Creating Duplicates?

1. **Check state transitions:**
   - Watch console for: "FOLLOW-UP MESSAGE - reusing task"
   - Should show same task ID

2. **Verify in CLI:**
   ```bash
   agent-inbox list --all
   # Should show: 1 task (not multiple)
   ```

3. **If duplicates appear:**
   - Check `isTransitioning` flag is working
   - See `docs/session-notes/STATE_MACHINE.md`

---

## 📊 Expected Behavior

### First Message
```
Console:
  [Agent Inbox DEBUG] NEW CONVERSATION DETECTED
  [Agent Inbox DEBUG]   Task ID: abc-123
  Task update sent: running abc-123

CLI:
  $ agent-inbox list --all
  RUNNING:
    1. [claude_web] "Hello" (5s ago)
```

### After Completion
```
Console:
  [Agent Inbox DEBUG] GENERATION COMPLETED
  [Agent Inbox DEBUG]   Task ID: abc-123
  Task update sent: completed abc-123

CLI:
  $ agent-inbox list --all
  COMPLETED:
    1. [claude_web] "Hello" (30s ago)
```

### Follow-up Message
```
Console:
  [Agent Inbox DEBUG] FOLLOW-UP MESSAGE - reusing task
  [Agent Inbox DEBUG]   Task ID: abc-123  ← SAME ID!
  Task update sent: running abc-123

CLI:
  $ agent-inbox list --all
  RUNNING:
    1. [claude_web] "Hello" (2s ago)  ← SAME TASK!
```

---

## 🗂️ Project Structure

```
agent-notifications/
├── README.md                     ⭐ Start here
├── TESTING_GUIDE.md              🧪 How to test
├── UPDATE_INSTRUCTIONS.md        🔄 How to update
├── REFACTOR.md                   🏗️ Architecture
├── FIXES.md                      📝 Changelog
│
├── docs/
│   ├── session-notes/            📔 Development logs
│   └── archive/                  📦 Old docs
│
├── extension/                    🔌 Browser extension
│   ├── content-scripts/
│   │   ├── shared.js            ⚙️ ConversationTracker
│   │   ├── claude.js            🤖 Claude.ai
│   │   └── gemini.js            💎 Gemini
│   └── background.js            🌉 Native messaging
│
├── src/                          🦀 Rust CLI
├── wrappers/                     📦 CLI wrappers
└── old-files/                    🗄️ Backups
```

---

## 🎯 Quick Start

```bash
# 1. Install everything
./install.sh

# 2. Reload extension
brave://extensions → Reload

# 3. Test in browser
# Open Claude.ai or Gemini
# Send a message
# Watch DevTools console

# 4. Check CLI
agent-inbox list --all
```

---

## 📚 Documentation Index

| Document | Purpose |
|----------|---------|
| `README.md` | Project overview |
| `TESTING_GUIDE.md` | Test procedures |
| `UPDATE_INSTRUCTIONS.md` | How to update |
| `REFACTOR.md` | Architecture details |
| `FIXES.md` | Changelog |
| `CLAUDE_HOOKS.md` | Claude Code integration |
| `GEMINI_TROUBLESHOOTING.md` | Gemini debugging |
| `PROJECT_STRUCTURE.md` | File organization |
| `docs/session-notes/STATE_MACHINE.md` | State transitions |
| `docs/session-notes/SESSION_SUMMARY.md` | Complete session log |
| `docs/session-notes/NATIVE_MESSAGING_ERROR.md` | Fix connection issues |

---

## 💡 Tips

**For daily use:**
- Just run `agent-inbox` to see what needs attention
- Use `watch -n 1 'agent-inbox list --all'` for monitoring

**For debugging:**
- Always check browser console first
- Use `diagnoseClaude()` / `diagnoseGemini()` liberally
- Read `docs/session-notes/STATE_MACHINE.md` if confused

**For development:**
- Test changes with: `cargo build --release && sudo cp target/release/agent-inbox /usr/local/bin/`
- Reload extension after ANY changes to JavaScript
- Check background worker console for native messaging errors

---

## 🆘 Getting Help

1. **Check documentation:**
   - Start with this QUICK_REFERENCE.md
   - Then check specific guide for your issue

2. **Run diagnostics:**
   ```javascript
   diagnoseClaude()    // or diagnoseGemini()
   ```

3. **Check logs:**
   - Browser console: `[Agent Inbox DEBUG]` messages
   - Background worker: `brave://extensions` → Inspect views
   - CLI: `agent-inbox list --all`

4. **Common issues:**
   - Native messaging → `docs/session-notes/NATIVE_MESSAGING_ERROR.md`
   - Not tracking → `GEMINI_TROUBLESHOOTING.md`
   - Duplicates → `docs/session-notes/STATE_MACHINE.md`
