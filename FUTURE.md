# Deferred Features

Features that were considered and intentionally deferred. Each entry explains why
it was deferred and what would be needed to implement it.

---

## Pi Agent: Telegram Notifications & Injection — NOT IMPLEMENTED

> **Status**: Intentionally deferred. There is no ETA.

**What works today**:
- `nibble sandbox spawn --pi` and `nibble sandbox attach --pi` work fine for
  interactive TUI sessions.

**What does NOT work**:
- **Telegram notifications when Pi finishes a turn.** Claude Code achieves this
  via a `Stop` hook in `~/.claude/settings.json`. Pi has no equivalent lifecycle
  hooks, so there is no mechanism to call `nibble notify` when an assistant turn
  completes.
- **Telegram reply injection into Pi sandboxes.** `nibble inject` and the Telegram
  listener's "↩ Reply" button route to `agent_input::inject_returning_child`,
  which is hardcoded to run `claude --resume`. Pi would need `pi --print` with
  a post-exit epilogue that sends the completion notification.

**Why it's hard**:
1. Pi has no `--on-exit` hook or settings file where we can register a shell
   command to run after each turn.
2. Pi sessions are JSONL files with internal tree structures. Reading the "last
   assistant message" from the outside is possible (we already do this for the
   safety-net), but without a hook we only know the process exited — not whether
   it finished successfully, needs attention, or is still mid-conversation.
3. Building a wrapper script around `pi --print` would work for injection, but
   would not help for interactive `attach` sessions where the user is in the TUI.

**Possible paths forward**:
- **Pi adds hooks**: If Pi ever supports an `--on-exit` or `--hook` flag, we can
  wire it up exactly like Claude Code.
- **Extension approach**: A Pi TypeScript extension could listen for turn-end
  events and call `nibble notify` via `child_process.spawn`. This requires
  installing an extension inside every sandbox.
- **File-watcher approach**: A background process watches the Pi session JSONL
  for new `"type":"message"` entries with `role:"assistant"` and calls
  `nibble notify`. This is fragile and may fire on compaction or branching.

**Current behaviour**: Pi sandboxes are invisible to Telegram. You will not
receive completion notifications, and you cannot reply to Pi tasks from your
phone. Use `nibble sandbox attach --pi` for all interaction.
