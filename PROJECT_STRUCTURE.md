# Project Structure

## Directory Organization

```
agent-notifications/
├── src/                          # Rust CLI source code
│   ├── bin/                      # Binary targets (agent-inbox, agent-bridge)
│   ├── cli/                      # CLI command implementations
│   └── db/                       # Database operations
│
├── extension/                    # Browser extension
│   ├── background.js             # Service worker (native messaging)
│   ├── manifest.json             # Extension manifest
│   ├── content-scripts/
│   │   ├── shared.js             # Shared tracking logic (ConversationTracker)
│   │   ├── claude.js             # Claude.ai content script
│   │   └── gemini.js             # Gemini content script
│   └── icons/                    # Extension icons
│
├── wrappers/                     # CLI tool wrappers
│   ├── claude-wrapper            # Wrapper for Claude Code
│   └── opencode-wrapper          # Wrapper for OpenCode (future)
│
├── docs/                         # Documentation
│   ├── session-notes/            # Session work logs and fixes
│   │   ├── CLAUDE_DETECTION_TEST.md
│   │   ├── FINAL_FIXES.md
│   │   ├── LATEST_FIXES.md
│   │   ├── NATIVE_MESSAGING_ERROR.md
│   │   ├── SESSION_SUMMARY.md
│   │   └── STATE_MACHINE.md
│   │
│   └── archive/                  # Old/obsolete documentation
│       ├── ATTENTION_DETECTION.md
│       ├── BRAVE_SETUP.md
│       ├── EXTENSION.md
│       ├── EXTENSION_DEBUGGING.md
│       ├── FUTURE_PROMPTS.md
│       ├── PROJECT_COMPLETE.md
│       └── WRAPPING_AGENTS.md
│
├── old-files/                    # Backup of old code
│   ├── claude-old.js             # Pre-refactor Claude script
│   └── gemini-old.js             # Pre-refactor Gemini script
│
├── README.md                     # Main project documentation
├── FIXES.md                      # Changelog of all fixes
├── REFACTOR.md                   # Architecture and refactor details
├── CLAUDE_HOOKS.md               # Claude Code hooks setup
├── GEMINI_TROUBLESHOOTING.md    # Gemini debugging guide
├── TESTING_GUIDE.md              # Comprehensive testing procedures
├── UPDATE_INSTRUCTIONS.md        # How to update and deploy
│
├── install.sh                    # Install script (main)
├── install-extension.sh          # Install browser extension
├── setup-wrappers.sh             # Setup CLI wrappers
└── fix-native-messaging.sh       # Fix native messaging connection
```

---

## Key Files by Category

### Active Documentation (Root)

**User-facing guides:**
- `README.md` - Project overview and quick start
- `TESTING_GUIDE.md` - How to test the system
- `UPDATE_INSTRUCTIONS.md` - How to update after pulling changes

**Implementation details:**
- `REFACTOR.md` - Architecture, deduplication design
- `CLAUDE_HOOKS.md` - Claude Code hooks integration
- `GEMINI_TROUBLESHOOTING.md` - Gemini-specific debugging
- `FIXES.md` - Complete changelog of fixes

### Session Documentation (docs/session-notes/)

Work logs and detailed analysis from development sessions:
- `STATE_MACHINE.md` - Complete state transition documentation
- `SESSION_SUMMARY.md` - Full session overview
- `LATEST_FIXES.md` - Most recent fixes explained
- `FINAL_FIXES.md` - Claude/Gemini issue fixes
- `NATIVE_MESSAGING_ERROR.md` - Native messaging troubleshooting
- `CLAUDE_DETECTION_TEST.md` - Claude detection testing script

### Archived Documentation (docs/archive/)

Historical or superseded documentation:
- Previous implementation attempts
- Old setup guides
- Obsolete architecture notes

### Installation Scripts

- `install.sh` - Main installation (builds binaries, installs wrappers, extension)
- `install-extension.sh` - Browser extension only
- `setup-wrappers.sh` - CLI wrappers only
- `fix-native-messaging.sh` - Fix extension ↔ CLI connection

---

## Core Components

### 1. CLI Tools (Rust)

**Location:** `src/`

**Binaries:**
- `agent-inbox` - Main CLI dashboard
- `agent-bridge` - Native messaging host

**Key Modules:**
- `src/cli/mod.rs` - CLI commands
- `src/db/mod.rs` - SQLite database operations
- `src/main.rs` - Entry point

### 2. Browser Extension (JavaScript)

**Location:** `extension/`

**Architecture:**
```
┌─────────────────┐
│  Content Script │ (claude.js / gemini.js)
│  - Detect state │
│  - Get conv ID  │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│   shared.js     │ (ConversationTracker)
│  - State machine│
│  - Deduplication│
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  background.js  │ (Service Worker)
│  - Forward msgs │
└────────┬────────┘
         │
         ▼
┌─────────────────┐
│  agent-bridge   │ (Native Host)
│  - Save to DB   │
└─────────────────┘
```

**Key Files:**
- `shared.js` - ConversationTracker class (shared logic)
- `claude.js` - Claude.ai-specific detection
- `gemini.js` - Gemini-specific detection
- `background.js` - Native messaging bridge
- `manifest.json` - Extension configuration

### 3. CLI Wrappers (Bash)

**Location:** `wrappers/`

Transparent wrappers that intercept CLI tool invocations:
- `claude-wrapper` - Wraps Claude Code
- Exports `AGENT_TASK_ID` for hooks
- Starts background monitor process
- Reports start/complete to database

---

## State Machine

The extension uses a 4-state machine (documented in `docs/session-notes/STATE_MACHINE.md`):

1. **State 1:** `wasGen=false, isGen=true` → Start/Resume (NEW or FOLLOW-UP)
2. **State 2:** `wasGen=true, isGen=false` → Complete
3. **State 3:** `wasGen=true, isGen=true` → Continue (ongoing)
4. **State 4:** `wasGen=false, isGen=false` → Idle

**Key Insight:** Follow-up messages go through State 1 (reuse task ID), not State 3.

---

## Database Schema

**Location:** `~/.agent-tasks/tasks.db`

```sql
CREATE TABLE tasks (
    id INTEGER PRIMARY KEY,
    task_id TEXT UNIQUE NOT NULL,
    agent_type TEXT NOT NULL,
    title TEXT NOT NULL,
    status TEXT NOT NULL,
    created_at INTEGER NOT NULL,
    updated_at INTEGER NOT NULL,
    completed_at INTEGER,
    context TEXT,
    -- ... more fields
);
```

**Task States:**
- `running` - In progress
- `completed` - Finished (auto-deleted after 1 hour)
- `needs_attention` - Waiting for user input
- `failed` - Errored

---

## Common Workflows

### 1. Update Extension After Code Changes

```bash
# Rebuild binaries
cargo build --release
sudo cp target/release/{agent-inbox,agent-bridge} /usr/local/bin/

# Reload extension
# Go to brave://extensions → Reload "Agent Inbox Tracker"
```

### 2. Debug Extension

```bash
# Open DevTools in Claude.ai/Gemini
# Watch console for: [Agent Inbox DEBUG] messages

# Run diagnostics
diagnoseClaude()    # In Claude.ai console
diagnoseGemini()    # In Gemini console
```

### 3. Test End-to-End

```bash
# Start conversation in browser
# Check tracking
agent-inbox list --all

# Watch in real-time
watch -n 1 'agent-inbox list --all'
```

### 4. Fix Native Messaging

```bash
./fix-native-messaging.sh
# Enter extension ID when prompted
# Reload extension
```

---

## Testing

See `TESTING_GUIDE.md` for comprehensive test procedures.

**Quick Tests:**
1. New conversation → Creates 1 task
2. Follow-up message → Reuses same task
3. Generation completes → Marks as completed
4. No duplicates throughout

---

## Development

### Adding a New Agent

To add support for a new web agent (e.g., ChatGPT):

1. **Create content script:** `extension/content-scripts/chatgpt.js`
2. **Implement 3 functions:**
   ```javascript
   function getConversationId() { /* Extract from URL */ }
   function getConversationTitle() { /* Get first message */ }
   function isGenerating() { /* Detect state */ }
   ```
3. **Use tracker:**
   ```javascript
   const tracker = new ConversationTracker("chatgpt_web");
   tracker.checkState(conversationId, isGenerating, title);
   ```
4. **Update manifest.json:**
   ```json
   {
     "matches": ["https://chatgpt.com/*"],
     "js": ["content-scripts/shared.js", "content-scripts/chatgpt.js"]
   }
   ```

All deduplication logic is automatic via `ConversationTracker`!

---

## Troubleshooting

**Extension not tracking?**
→ See `GEMINI_TROUBLESHOOTING.md`

**Native messaging error?**
→ See `docs/session-notes/NATIVE_MESSAGING_ERROR.md`

**Claude not detecting completion?**
→ See `docs/session-notes/CLAUDE_DETECTION_TEST.md`

**State machine confusion?**
→ See `docs/session-notes/STATE_MACHINE.md`

---

## Reference Documentation

**For Users:**
- Start here: `README.md`
- Testing: `TESTING_GUIDE.md`
- Updates: `UPDATE_INSTRUCTIONS.md`

**For Developers:**
- Architecture: `REFACTOR.md`
- State machine: `docs/session-notes/STATE_MACHINE.md`
- Session notes: `docs/session-notes/`

**For Debugging:**
- Fixes log: `FIXES.md`
- Gemini issues: `GEMINI_TROUBLESHOOTING.md`
- Claude hooks: `CLAUDE_HOOKS.md`
- Native messaging: `docs/session-notes/NATIVE_MESSAGING_ERROR.md`

---

## File Cleanup Summary

**Moved to `docs/session-notes/`:**
- Session work logs (STATE_MACHINE.md, SESSION_SUMMARY.md, etc.)
- Detailed fix analysis (FINAL_FIXES.md, LATEST_FIXES.md)
- Debug guides (NATIVE_MESSAGING_ERROR.md, CLAUDE_DETECTION_TEST.md)

**Moved to `docs/archive/`:**
- Obsolete documentation (PROJECT_COMPLETE.md, EXTENSION.md)
- Old setup guides (BRAVE_SETUP.md, setup-brave-extension.sh)
- Historical notes (ATTENTION_DETECTION.md, WRAPPING_AGENTS.md)

**Moved to `old-files/`:**
- Pre-refactor code backups (claude-old.js, gemini-old.js)

**Kept in root:**
- Active documentation (README.md, TESTING_GUIDE.md, REFACTOR.md)
- Installation scripts (install.sh, fix-native-messaging.sh)
- Implementation guides (CLAUDE_HOOKS.md, GEMINI_TROUBLESHOOTING.md)
