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
- **If already flagged:** do NOT mention it again. Not even to say "already raised
  by @X". Simply skip it. The user does not need a catalog of what they already
  read.
- **If new:** surface it with the same severity and actionable format.
- Build upon existing review threads rather than rehashing them. If you agree
  with an existing reviewer and have additional context, add a reply to their
  thread instead of opening a duplicate.
- **Dig-deeper heuristic:** if existing reviews already cover obvious bugs, style,
  or naming issues, shift your focus to:
  - Higher-level design gaps (see Phase 2c)
  - Edge cases or error paths the existing reviewers missed
  - Race conditions, concurrency, or state-management issues
  - Missing tests for behavior that *is* implemented
  - Subtle API contract violations or backward-compatibility breaks
  - Performance or operational concerns (memory, latency, observability)
  - Hidden coupling or future maintenance burdens
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
dependency relationships), **clone the repo and check out the PR branch locally.**

You don't need to ask for permission — just clone it to `/tmp/pr-review-<number>`
and clean up when done. This is especially important for unfamiliar codebases or
large PRs where the diff alone doesn't provide enough context.

**When to clone (balance thoroughness vs. token efficiency):**

Always clone when:

- The PR touches code you're not familiar with and you need surrounding context
  (framework patterns, base classes, shared types, module boundaries).
- The PR is in a repo you don't have in `/<repo-name>`
- You need to trace types/imports across multiple packages in a monorepo.
- The diff is large (>500 lines) and context is hard to follow in the browser.
- You suspect a bug that would be confirmed by reading surrounding code or running tests.

Usually skip cloning when:

- The repo is already in `/<repo-name>` and you can read files directly.
- The PR is small, self-contained, and the diff tells the full story.
- You're already familiar with the codebase patterns from prior reviews.

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
   on (`rg`, `find`, `read`). Understand the framework, base classes, service
   layer, and module boundaries. Focus on:
   - Base classes and interfaces the PR extends/implements
   - Existing patterns in sibling modules/services
   - Database schemas, type definitions, and shared utilities
   - How other apps in the monorepo are structured
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
- Housekeeping: leftover `.gitkeep` in directories that now have files, stale re-exports, dead imports

#### G. AI/LLM-Specific (when the PR includes prompts, judge configs, or AI content)

- **Prompt engineering:** Do examples bias the judge toward a specific domain? Are edge cases in the prompt realistic?
- **Shared context:** Is preliminary system context duplicated across prompt files? Should it be extracted into a shared preamble?
- **Config structure:** If multiple configs share a pattern, should it be formalized (e.g., structured sections, schema extensions) to make adding new entries easy?
- **Over-optimization risk:** Are there too many examples that might cause the LLM to over-fit to those scenarios?

#### H. Delegation & Uncertainty

- If a change touches a domain you're not confident about (DB schema, business logic, external API contracts), **tag a domain expert** rather than guessing. Example: `@engineer-name to confirm we have this data in the database.`
- When you're unsure but have a hunch, frame it as a question or soft suggestion ("I would maybe iterate on this..."), not a prescriptive finding.

**Peer-Review de-duplication rule:** Before writing up a finding, scan the
existing review comments (from Phase 2b). If the same file/line/issue was already
flagged, skip it. Only present genuinely new findings to the user.

**Leverage prior reviews as a filter:** because the easy issues are already
caught, your job is to find the *non-obvious* ones. Do a second pass specifically
looking for issues the first reviewers likely overlooked.

### Phase 4 — Deliver the Review

Separate the review into **two layers**: a high-level summary (the review body) and low-level inline comments on specific lines.

#### Layer 1 — High-level review body (architecture + verdict)

Keep this concise. It should cover:

1. **Acknowledge existing reviews** (if any). One brief sentence at the top.
2. **Architecture assessment** — is the design sound? (see Phase 2c)
3. **Overall verdict** — Approve / Request Changes / Comment
4. **A brief severity list** of the findings (just titles, no details)
5. **Highlight positives** — call out good patterns, clean tests
6. **Nibble signature** — always end with `---\n*🤖 nibble-generated review*` so reviewers can identify AI-generated reviews

```markdown
## High-level Review

Solid library foundation with good abstractions. The six-service pipeline is
well-factored and testable.

However, **requesting changes** due to a critical logic bug and a failing CI check.

### Blockers
- 🔴 `not_equals` operator logic is inverted on multi-value paths
- 🟡 CI `Lint, Test and Build` is failing
- 🟡 Missing test coverage for `not_equals` and `exists` operators

### Minor
- 🔵 Provider instance should be cached (not recreated per call)
- 🔵 Placeholder regex only replaces first occurrence

See inline comments for details.

---
*🤖 nibble-generated review*
```

#### Layer 2 — Inline comments (code-level findings)

Each finding that maps to a specific line should be an **inline comment**, not text in the review body. This keeps the review body readable and puts actionable feedback directly on the diff.

Draft each inline comment with:

- `path`: file path (e.g. `apps/api/src/modules/catalog/catalog.module.ts`)
- `line`: line number on the RIGHT side of the diff (the PR's version)
- `side`: always `"RIGHT"` for comments on the PR's changes
- `body`: the comment text (severity emoji + description + suggestion)

```json
{
  "path": "apps/agent-observability/src/services/candidate-builder.service.ts",
  "line": 115,
  "side": "RIGHT",
  "body": "🔴 **Critical — `not_equals` logic is inverted on multi-value paths**\n\n`arr.some((v) => v !== filterValue)` means 'match if ANY element is different' — almost always true for arrays with >1 element. The intended semantics for `not_equals` on a collection is 'match if NONE of the elements equal the filter value.'\n\n**Impact:** A generation-scoped judge with a `not_equals` filter on a multi-value path would incorrectly match traces that DO contain the excluded value, causing false-positive verdicts.\n\nSuggested fix: change to `arr.every((v) => v !== filterValue)`."
}
```

**Rule of thumb:** If a finding references a specific file and line, it belongs as an inline comment. Meta concerns (CI status, missing tests for an entire operator, architectural concerns) belong in the review body.

#### Severity guide

| Severity | When to use |
|----------|------------|
| 🔴 Critical | Bugs, security vulnerabilities, data loss risk |
| 🟡 Major | Design issues, missing error handling, incorrect logic |
| 🔵 Minor | Style, naming, small improvements |
| 💡 Suggestion | Optional improvements, questions, alternative approaches |

### Phase 5 — Submit the Review

#### Option A — Review body only (no inline comments)

Use `gh pr review` for a simple text-only review:

```bash
# Approve with comments
gh pr review <NUMBER> --repo <owner/repo> --approve --body "<review body>"

# Request changes
gh pr review <NUMBER> --repo <owner/repo> --request-changes --body "<review body>"

# Comment only (no explicit approve/reject)
gh pr review <NUMBER> --repo <owner/repo> --comment --body "<review body>"
```

#### Option B — Review with inline comments (preferred for peer reviews)

Use the **Reviews API** to create a single review that includes both the high-level body and inline comments on specific lines. This is the preferred approach when you have code-level findings.

**Step 1 — Fetch the head commit SHA:**

```bash
gh pr view <NUMBER> --repo <owner/repo> --json headRefOid --jq '.headRefOid'
```

**Step 2 — Build a JSON payload with the review body and comments array:**

```json
{
  "commit_id": "<headRefOid>",
  "body": "## High-level Review\n\n...\n\n---\n*🤖 nibble-generated review*",
  "event": "REQUEST_CHANGES",
  "comments": [
    {
      "path": "apps/agent-observability/src/services/candidate-builder.service.ts",
      "line": 115,
      "side": "RIGHT",
      "body": "🔴 **Critical — ..."
    },
    {
      "path": "apps/agent-observability/src/services/judge-runner.service.ts",
      "line": 48,
      "side": "RIGHT",
      "body": "🔵 **Minor — ..."
    }
  ]
}
```

**Step 3 — Post via `gh api`:**

```bash
cat <<'EOF' > /tmp/review-payload.json
{ "commit_id": "<sha>", "body": "...", "event": "REQUEST_CHANGES", "comments": [...] }
EOF
gh api repos/<owner>/<repo>/pulls/<number>/reviews --input /tmp/review-payload.json
```

**Important constraints:**

- Each inline comment must reference a line that appears in the diff (added or context lines).
- `side`: `"RIGHT"` for comments on the PR's version of the code.
- `line`: the line number in the NEW file (the PR branch's version).
- For multi-line comments, also include `start_line` and `start_side`.
- Always use the latest `commit_id` (head of the PR branch), not the base.

**Always confirm with the user before submitting.** Show the high-level review text and the list of inline comments (file + line + severity) and let them edit or approve. Never submit without explicit confirmation.

### Phase 5b — Inline Comments via API (Peer-Review only, Human-in-the-Loop)

> **Note:** The preferred way to post inline comments is via the **Reviews API** (Phase 5, Option B), which creates both the review body and all inline comments in a single atomic call. Use the standalone comment API below only if you need to add comments to an already-submitted review.

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

# Submit top-level review (body only)
gh pr review <NUMBER> --repo <owner/repo> --approve --body "<body>"
gh pr review <NUMBER> --repo <owner/repo> --request-changes --body "<body>"
gh pr review <NUMBER> --repo <owner/repo> --comment --body "<body>"

# Submit review with inline comments (preferred — body + comments in one call)
# Build a JSON payload with "commit_id", "body", "event", and "comments" array,
# then post via the Reviews API:
gh api repos/{owner}/{repo}/pulls/{number}/reviews --input /tmp/review-payload.json

# Inline comment on specific line (standalone, only if adding to existing review)
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
- **Nibble signature:** Always end every review body with `---\n*🤖 nibble-generated review*`
  so that human reviewers can identify which comments were AI-generated.
