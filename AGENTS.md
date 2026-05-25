<!-- nibble-sandbox:begin -->
# nibble Sandbox Agent Instructions

You are running inside an isolated Podman sandbox managed by **nibble** for the **nibble** project. This file contains all instructions for how to operate inside this environment. Read it fully before starting any task.

## Environment

- **Working directory**: `/nibble (the project repo, mounted read-write)
- **Full sudo access**: install any system package with `apt-get install`
- **Ports forwarded** to the host: services on `localhost:3000`, `:8080`, etc. are reachable from outside
- **Internet access** is available
- **Git** is configured with the host user's identity and SSH keys
- `claude` is available if you need a nested agent session

## Toolchain Setup

Project dependencies are installed automatically at sandbox spawn via `.nibble/setup.sh` if that script exists. By the time you receive a task, dependencies should already be built and ready.

- If `.nibble/setup.sh` **exists**: it was already run at spawn — do not re-run it unless something is broken. If you need a new system dependency or build step, update the script and run it manually once, then commit the change.
- If `.nibble/setup.sh` **does not exist**: dependencies won't be pre-installed. Check for manifest files below and install them yourself. Create (or ask to create) `.nibble/setup.sh` so future spawns are automatic.

The following dependency manifests were detected:

| Manifest | Install command | Run/test |
|----------|----------------|----------|
| Rust | `cargo build  # rustup + cargo pre-installed by .nibble/setup.sh; binary at ~/.cargo/bin/cargo` | `cargo run / cargo test` |

If a command fails due to missing system tools, install them with `sudo apt-get install <package>`.

## General Working Principles

- Make small, focused changes and run tests after each one
- The container persists between sessions — installed packages and build artifacts are retained
- When you finish a task, summarise what you did clearly so the notification sent to the user is informative
- Ask before making changes outside the project's stated scope

## Skills & Lessons

Factory pipeline skills are stored on the **host** at `~/.claude/skills/` and bind-mounted into every sandbox at `/home/node/.claude/skills/`. This means:

- Skills and lessons updates made inside a sandbox are immediately visible on the host and in all other sandboxes — they share the same directory.
- To persist a lessons-learned update, edit the skill file directly (e.g. `~/.claude/skills/factory-lessons/SKILL.md`). No restart or re-injection needed.
- The host `install.sh` re-installs skills from the nibble repo to `~/.claude/skills/` whenever you update them in source. Run it after editing skills in the repo.

<!-- nibble:global:begin -->
## AI Factory Pipeline

When factory is enabled, every non-trivial coding task follows the AI Factory pipeline.

Load skill `factory-pipeline` to classify the task and determine which tier to run:

- **Quick** (≤3 functions, no security/API change): Spec → Implement → Verify
- **Standard** (4–15 functions): Spec → Implement → Verify → Audit
- **Full** (16+ functions, security-sensitive, API changes): Full pipeline with QA Gate

**QA Gate fires for ANY tier** when unfixed Critical or High findings are discovered. For Full tier, QA Gate always fires.

Skills: `factory-pipeline` · `factory-spec` · `factory-verify` · `factory-qa-gate` · `factory-lessons`

```
.nibble/factory/blueprints/    # Feature specs (committed)
.nibble/factory/reports/audit/ # Adversarial + risk findings (gitignored)
.nibble/factory/reports/qa/    # QA gate decisions (committed)
```
<!-- nibble:global:end -->

<!-- nibble-sandbox:end -->
