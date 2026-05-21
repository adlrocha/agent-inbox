---
name: nibble-pr-review
description: Track and review GitHub PRs where you are requested as a reviewer.
  Uses gh CLI to list pending reviews, inspect diffs, analyze changes, and submit
  review comments through an interactive agent-driven flow.
---

# PR Review Skill

Track and review GitHub PRs where you are requested as a reviewer, using the
`gh` CLI and your coding agent capabilities.

## Prerequisites

- `gh` CLI installed and authenticated (`gh auth status`)
- Git repository with a GitHub remote (or explicit `--repo owner/name`)

### Authentication in nibble sandboxes

`gh` credentials are **not** automatically inherited from the host. Nibble's sandbox
spawner forwards `GITHUB_TOKEN` (and only `GITHUB_TOKEN`) into the container at
`podman run` time. If `GITHUB_TOKEN` is not set in the host shell when the sandbox
is spawned, `gh` will be unauthenticated inside.

**To fix:** On the host, run:
```bash
export GITHUB_TOKEN=$(gh auth token)
```
then spawn or attach with `--fresh`:
```bash
nibble sandbox attach /path/to/repo --fresh
```

If `GITHUB_TOKEN` is already set but `gh auth status` still fails, the token may
have expired — re-export it from the host and restart the sandbox.

## Flow

### Phase 1 — Dashboard

Gather all PRs awaiting your review and present a prioritized dashboard.

Use the **cross-repo search API** as the default — it covers all repos the user has
access to and does not require being inside a specific repo's working directory:

```bash
# Primary: search across all repos (recommended)
gh search prs --review-requested=@me --state open --json repository,number,title,url,author,updatedAt
```

The legacy `gh pr list --search` command requires a `--repo` flag and its `--json`
fields differ (no `repository` field). Only use it when reviewing PRs in the
current repo specifically:

```bash
# Fallback: current repo only
gh pr list --search "review-requested:@me state:open" --json number,title,author,updatedAt,url,labels
```

Present results as a numbered table:

```
📋 Pending PR Reviews

  #  Repo                PR     Title                                    Author      Updated
  1  owner/repo-a        #42    Refactor auth middleware                 @alice      2h ago
  2  owner/repo-b        #108   Add caching layer to API                @bob        1d ago
  3  ...

  Type a number to start reviewing, or "skip" to exit.
```

**Prioritization hints** (mention if applicable):
- PRs labeled `urgent`, `security`, `hotfix` → review first
- Older PRs (stale) → call out
- Large PRs (>500 lines changed) → flag for the user

### Phase 2 — PR Inspection

When the user picks a PR, gather full context:

```bash
# Core PR info (include url so we can link it)
gh pr view <NUMBER> --repo <owner/repo> --json title,body,author,baseRefName,headRefName,additions,deletions,changedFiles,labels,reviews,comments,url,reviewDecision

# The actual diff
gh pr diff <NUMBER> --repo <owner/repo>

# Existing review comments (critical for peer reviews — see Phase 2b)
gh api repos/{owner}/{repo}/pulls/{number}/reviews
gh api repos/{owner}/{repo}/pulls/{number}/comments

# CI status
gh pr checks <NUMBER> --repo <owner/repo>

# Conversation thread
gh pr view <NUMBER> --repo <owner/repo> --comments
```

Present a summary to the user. **Always include the direct PR URL** so the user
can navigate to it easily:

```
🔍 Reviewing: #42 — Refactor auth middleware
   Repo: owner/repo-a
   PR URL: https://github.com/owner/repo-a/pull/42
   Author: @alice
   Branch: feature/auth-refactor → main
   Size: +324 / -89 (12 files)
   Labels: enhancement
   CI: ✅ all checks passed
   Existing reviews: none / 1 comment by @bob

   Changes overview:
   - auth/middleware.rs: Replaced session-based auth with JWT validation
   - auth/session_store.rs: Removed (deleted file)
   - routes/protected.rs: Updated guards to use new auth extractor
   - tests/auth_tests.rs: New test suite for JWT flow
```

### Phase 2b — Detect Self-Review vs. Peer Review

After fetching the PR author, determine whether this is the **user's own PR** or
**someone else's**:

```bash
# Get the authenticated user's login
gh api user --jq '.login'
```

Compare it to `pr.author.login`. Then branch into the appropriate review mode.

#### Self-Review mode (`author.login == current_user`)

- The user just pushed this PR and wants a final sanity check before asking others.
- Focus on: completeness, CI status, test coverage, documentation, no embarrassing
  typos or debug logs left behind.
- There are typically **no existing reviews** from humans (maybe bot comments).
- Be constructive but direct — treat it like a pre-flight checklist.
- You may still deliver findings as a structured text summary; the user can fix
  issues locally and force-push.

#### Peer-Review mode (`author.login != current_user`)

- You are reviewing someone else's work.
- **Mandatory:** fetch all existing reviews and inline comments (see commands above).
- Before surfacing any finding, check whether it was already flagged by another
  reviewer (e.g. @asmarques, @gemini-code-assist).
- **If already flagged:** skip it or briefly note "already raised by @X" — do NOT
  re-post the same issue.
- **If new:** surface it with the same severity and actionable format.
- Build upon existing review threads rather than rehashing them. If you agree
  with an existing reviewer and have additional context, add a reply to their
  thread instead of opening a duplicate.
- The final deliverable can be either:
  - A top-level review comment (approve / request changes / comment)
  - Individual **inline comments** posted via the API (see Phase 5b)
  - Both

### Phase 2c — Architecture Review ("Does this design make sense?")

Step back before reading individual lines. Assess whether the high-level design
is sound — especially for new features, API additions, or cross-subsystem
refactorings. This is about **subsystem-level decisions**, not code-level patterns
(those come in Phase 3).

**Questions to answer:**

1. **Feature placement — does this belong here?**
   - Is the new module/service in the right layer?
   - Does it respect existing boundaries (e.g. repo vs. service vs. handler)?

2. **Data flow — is the path from input to output sound?**
   - Any unnecessary hops or round-trips?
   - Is caching applied consistently?

3. **Abstraction level — too much or too little?**
   - Premature abstraction (indirection with no benefit)?
   - Logic leaked into the wrong layer (e.g. DB queries in a route handler)?

4. **Coupling — are boundaries respected?**
   - New circular dependencies?
   - Internal details leaking across modules?
   - Can the new code be tested in isolation?

5. **API design (if applicable)**
   - Paths, parameters, and responses consistent with existing conventions?
   - Versioning strategy?
   - Error shapes uniform?

6. **Operational concerns**
   - Expected load — will it scale?
   - New failure modes (external dependency going down)?
   - Observability (logs, metrics) adequate?

7. **Migration / backward compatibility**
   - Breaking changes for existing clients?
   - Rollout strategy?

> **If the architecture is flawed, stop here.** Flag the design concern and
> suggest an alternative. Don't review variable names when the module shouldn't
> exist in its current form.
>
> **If the architecture is sound, proceed** to Phase 2d (local checkout) or
> Phase 3 (detailed code review).

### Phase 2d — Deep Dive: Local Checkout & Test Execution

The diff and GitHub file browser are often insufficient for a thorough review.
If you need to understand context (surrounding files, imports, test setup,
dependency relationships), **ask the user for permission to clone the repo and
check out the PR branch locally.**

**When to ask for local checkout:**
- The PR touches files you're unfamiliar with and you need to see the broader
  module structure.
- You suspect a bug that would be confirmed by running the test suite.
- The PR introduces a new dependency or build step you want to validate.
- You need to trace types/imports across multiple packages in a monorepo.
- The diff is large (>500 lines) and context is hard to follow in the browser.

**How to check out the PR branch:**

```bash
# Clone the repo (or navigate to it if already in /workspace)
gh repo clone <owner/repo> /tmp/pr-review-<number>
cd /tmp/pr-review-<number>

# Fetch and check out the PR branch
gh pr checkout <NUMBER>

# Or manually:
git fetch origin pull/<NUMBER>/head:pr-<NUMBER>
git checkout pr-<NUMBER>
```

**What to do once checked out:**

1. **Explore context** — Read surrounding files the PR imports from or depends
   on (`rg`, `find`, `read`). Understand the test helpers, service layer, and
   module boundaries.

2. **Run relevant tests** — Execute only the test suites touching changed code:
   ```bash
   cargo test                    # Rust
   pnpm test --filter=api        # JS/TS monorepo
   pytest tests/catalog/         # Python
   go test ./...                 # Go
   make test                     # Makefile-driven
   ```
   Report pass/fail and whether new behavior is actually exercised.

3. **Run lint / type check** — Catch errors CI might miss:
   ```bash
   cargo clippy; pnpm lint; mypy .; golangci-lint run
   ```

4. **Check for side effects** — After tests, run `git status`. Generated files
   (lockfiles, SDKs) that changed indicate missing regenerated artifacts in the PR.

5. **Clean up** — Remove the temp clone when done:
   ```bash
   rm -rf /tmp/pr-review-<number>
   ```

**Always ask the user first:** "This PR touches code I'm not deeply familiar
with. Can I check out the branch locally and run the tests to give you a more
thorough review?"

If the user says yes, proceed. If no, do your best with the diff and GitHub
file browser.

### Phase 3 — Structured Review

Perform the review systematically. Use these lenses:

#### A. Correctness
- Does the code do what the PR description says?
- Are edge cases handled? (empty inputs, null values, off-by-one, concurrent access)
- Are error paths covered? Do errors propagate correctly?

#### B. Design & Architecture
- Does the change fit the existing codebase patterns?
- Are abstractions at the right level?
- Is there unnecessary coupling or circular dependencies?
- Are new types/functions well-named and in the right module?

#### C. Security
- Input validation and sanitization
- Auth/authz changes — are they safe?
- Secrets or credentials exposed?
- Injection vectors (SQL, command, path traversal)?

#### D. Performance
- N+1 queries or unnecessary loops?
- Memory allocation patterns (large clones, missing iterators)?
- Async/blocking mismatches?

#### E. Testing
- Are new behaviors tested?
- Do tests cover error paths, not just happy paths?
- Are mocks/stubs appropriate?
- Are there missing test cases for the acceptance criteria?

#### F. Readability & Maintainability
- Clear naming, no cryptic abbreviations
- Comments explain *why*, not *what*
- No dead code or TODOs without tracking issues
- Consistent style with the rest of the codebase

**Peer-Review de-duplication rule:** Before writing up a finding, scan the
existing review comments (from Phase 2b). If the same file/line/issue was already
flagged, skip it. Only present genuinely new findings to the user.

### Phase 4 — Deliver the Review

Organize findings into a clear review comment.

#### Finding format

For each issue, classify severity and provide actionable feedback:

```
### 🔴 Critical / 🟡 Major / 🔵 Minor / 💡 Suggestion

**File:** `path/to/file.rs:45-52`

The issue description and why it matters.

```suggestion
// Concrete code suggestion if applicable
```
```

#### Severity guide

| Severity | When to use |
|----------|------------|
| 🔴 Critical | Bugs, security vulnerabilities, data loss risk |
| 🟡 Major | Design issues, missing error handling, incorrect logic |
| 🔵 Minor | Style, naming, small improvements |
| 💡 Suggestion | Optional improvements, questions, alternative approaches |

#### Drafting the review

1. **Summarize** your overall impression (approve, request changes, or comment)
2. **List findings** grouped by severity (Critical → Suggestion)
3. **Highlight positives** — call out good patterns, clever solutions, clean tests

**Self-Review variation:** Frame findings as a pre-flight checklist:
- "Before requesting review from the team, consider fixing..."
- "CI is green — nice. One thing to double-check..."

### Phase 5 — Submit the Review

Give the user options for how to submit:

```bash
# Approve with comments
gh pr review <NUMBER> --repo <owner/repo> --approve --body "<review body>"

# Request changes
gh pr review <NUMBER> --repo <owner/repo> --request-changes --body "<review body>"

# Comment only (no explicit approve/reject)
gh pr review <NUMBER> --repo <owner/repo> --comment --body "<review body>"
```

**Always confirm with the user before submitting.** Show the full review text and
let them edit or approve. Never submit without explicit confirmation.

### Phase 5b — Inline Comments via API (Peer-Review only, Human-in-the-Loop)

When reviewing someone else's PR, you can post **individual inline comments**
directly on specific lines using the GitHub API. This is useful for precise,
actionable feedback that appears in the diff view.

**Step 1 — Draft comments.** For each new finding (not already flagged by others),
draft:
- `path`: file path (e.g. `apps/api/src/modules/catalog/catalog.module.ts`)
- `line`: line number on the RIGHT side of the diff (the PR's version)
- `side`: always `"RIGHT"` for comments on the PR's changes
- `body`: the comment text (include severity emoji and suggestion if applicable)
- `commit_id`: the latest commit SHA on the PR branch (needed for API)

Fetch the latest commit SHA:
```bash
gh pr view <NUMBER> --repo <owner/repo> --json headRefOid --jq '.headRefOid'
```

**Step 2 — Present to user for approval.** Show each draft comment like this:

```
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
Draft inline comment 1 of 3

File: apps/api/src/modules/catalog/catalog.module.ts:17-21
Side: RIGHT

🟡 Major — `mapDataset` crashes if index document lacks `creator`

The `creator` field is destructured unconditionally. If any indexed document
has a missing or null `creator`, `creator.id` throws at runtime.

```suggestion
creator: creator ? {
  id: creator.id,
  username: creator.username,
  displayName: creator.displayName,
} : null,
```

Post this comment? [yes / skip / edit]
━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
```

Wait for the user's explicit response (`yes`, `skip`, or an edited version).
Do NOT batch-post; go one by one so the user can filter.

**Step 3 — Post approved comments.** For each approved comment:

```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  -f body="<comment text>" \
  -f path="<file>" \
  -f line=<line> \
  -f side="RIGHT" \
  -f commit_id="<headRefOid>"
```

**Important constraints:**
- Comments can only be posted on lines that appear in the diff (changed or context).
- If a line is outside the diff hunk, the API will reject it.
- For multi-line comments, also include `start_line` and `start_side`.
- Always use the latest `commit_id` (head of the PR branch), not the base.

After all inline comments are handled, optionally post a top-level review
(Phase 5) to summarize the overall verdict.

### Phase 6 — Loop

After submitting (or skipping), return to the dashboard:

```
✅ Review submitted for #42 (Approved / Changes Requested / Commented)

Return to dashboard? [Y/n]
```

If yes, re-run Phase 1 (the reviewed PR should now be gone from the list).

## Quick Commands Reference

```bash
# Dashboard — all pending reviews (cross-repo)
gh search prs --review-requested=@me --state open --json repository,number,title,url,author,updatedAt

# PR details + diff
gh pr view <NUMBER> --repo <owner/repo> --json title,body,author,additions,deletions,changedFiles,labels,url
gh pr diff <NUMBER> --repo <owner/repo>

# Current user login (for self-review detection)
gh api user --jq '.login'

# Existing reviews and inline comments (peer-review dedup)
gh api repos/{owner}/{repo}/pulls/{number}/reviews
gh api repos/{owner}/{repo}/pulls/{number}/comments

# CI checks
gh pr checks <NUMBER> --repo <owner/repo>

# Latest commit SHA (for inline comment posting)
gh pr view <NUMBER> --repo <owner/repo> --json headRefOid --jq '.headRefOid'

# Local checkout for deep validation
gh repo clone <owner/repo> /tmp/pr-review-<number>
cd /tmp/pr-review-<number> && gh pr checkout <NUMBER>

# Submit top-level review
gh pr review <NUMBER> --repo <owner/repo> --approve --body "<body>"
gh pr review <NUMBER> --repo <owner/repo> --request-changes --body "<body>"
gh pr review <NUMBER> --repo <owner/repo> --comment --body "<body>"

# Inline comment on specific line (peer-review, HITL approved only)
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  -f body="<comment>" -f path="<file>" -f line=<line> \
  -f side="RIGHT" -f commit_id="<sha>"
```

## Tips

- **Large PRs** (>500 lines): suggest the author split into smaller PRs. Offer to
  help identify logical split points.
- **Unfamiliar codebase**: spend extra time on Phase 2. Read surrounding code,
  check recent commits for context, look at linked issues.
- **Time-boxed reviews**: if the user is in a hurry, focus on Critical and Major
  findings only. Flag that the review was not comprehensive.
- **Draft PRs**: check if the PR is marked as draft. Ask the user if they still
  want to review (drafts may not be ready for full review).
- **Auto-merge**: if CI passes and you approved, remind the user they can enable
  auto-merge: `gh pr merge <NUMBER> --auto`
- **Peer-review etiquette:** When existing reviewers (e.g. @asmarques) have
  already flagged issues, do not re-post the same comment. Acknowledge their
  review, add only net-new findings, and feel free to reply to their threads
  if you have additional context.
