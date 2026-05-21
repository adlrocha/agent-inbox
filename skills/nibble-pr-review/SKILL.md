---
name: nibble-pr-review
description: Track and review GitHub PRs where you are requested as a reviewer.
  Uses gh CLI to list pending reviews, inspect diffs, analyze changes, and submit
  review comments through an interactive agent-driven flow.
---

# PR Review Skill

Track pending GitHub PR reviews and perform structured code reviews using the
`gh` CLI and your coding agent capabilities.

## Prerequisites

- `gh` CLI installed and authenticated (`gh auth status`)
- Git repository with a GitHub remote (or explicit `--repo owner/name`)

## Flow

### Phase 1 — Dashboard

Gather all PRs awaiting your review and present a prioritized dashboard.

```bash
# List PRs where you are requested as a reviewer
gh pr list --search "review-requested:@me state:open" --json number,title,repository,author,updatedAt,url,labels
```

If the user wants reviews across **multiple repos**, use the search API:

```bash
# Search across all repos the user has access to
gh search prs --review-requested=@me --state open --json repository,number,title,url,author,updatedAt
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
# Core PR info
gh pr view <NUMBER> --json title,body,author,baseRefName,headRefName,additions,deletions,changedFiles,labels,reviews,comments

# The actual diff
gh pr diff <NUMBER>

# Existing review comments (check if others already reviewed)
gh api repos/{owner}/{repo}/pulls/{number}/reviews
gh api repos/{owner}/{repo}/pulls/{number}/comments

# CI status
gh pr checks <NUMBER>

# Conversation thread
gh pr view <NUMBER> --comments
```

Present a summary to the user:

```
🔍 Reviewing: #42 — Refactor auth middleware
   Repo: owner/repo-a
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

### Phase 5 — Submit the Review

Give the user options for how to submit:

```bash
# Approve with comments
gh pr review <NUMBER> --approve --body "<review body>"

# Request changes
gh pr review <NUMBER> --request-changes --body "<review body>"

# Comment only (no explicit approve/reject)
gh pr review <NUMBER> --comment --body "<review body>"
```

For **inline comments** on specific lines:

```bash
# Use the API for line-specific comments
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  -f body="Comment text" \
  -f path="src/file.rs" \
  -f line=42 \
  -f side="RIGHT"
```

For inline comments on the **original** (left) side of the diff, use:
```bash
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  -f body="Comment text" \
  -f path="src/file.rs" \
  -f line=42 \
  -f side="LEFT"
```

**Always confirm with the user before submitting.** Show the full review text and
let them edit or approve. Never submit without explicit confirmation.

### Phase 6 — Loop

After submitting (or skipping), return to the dashboard:

```
✅ Review submitted for #42 (Approved / Changes Requested / Commented)

Return to dashboard? [Y/n]
```

If yes, re-run Phase 1 (the reviewed PR should now be gone from the list).

## Quick Commands Reference

```bash
# Dashboard — all pending reviews
gh pr list --search "review-requested:@me state:open" --json number,title,repository,author,updatedAt,url

# Cross-repo search
gh search prs --review-requested=@me --state open

# PR details + diff
gh pr view <NUMBER> --json title,body,author,additions,deletions,changedFiles,labels
gh pr diff <NUMBER>

# Existing reviews and inline comments
gh api repos/{owner}/{repo}/pulls/{number}/reviews
gh api repos/{owner}/{repo}/pulls/{number}/comments

# CI checks
gh pr checks <NUMBER>

# Submit review
gh pr review <NUMBER> --approve --body "<body>"
gh pr review <NUMBER> --request-changes --body "<body>"
gh pr review <NUMBER> --comment --body "<body>"

# Inline comment on specific line
gh api repos/{owner}/{repo}/pulls/{number}/comments \
  -f body="<comment>" -f path="<file>" -f line=<line> -f side="RIGHT"
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
