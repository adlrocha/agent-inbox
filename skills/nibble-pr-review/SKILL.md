---
name: nibble-pr-review
description: Track and review GitHub PRs where you are requested as a reviewer.
  Uses gh CLI to list pending reviews, inspect diffs, analyze changes, and submit
  review comments through an interactive agent-driven flow.
---

# PR Review Skill

Review GitHub PRs as an assistant that enhances the user's own reviewing
capabilities and judgement.

## Prerequisites

- `gh` CLI installed and authenticated (`gh auth status`)

**Sandbox auth:** `gh` credentials are not inherited from the host. The sandbox
receives `GITHUB_TOKEN` at spawn time. If unauthenticated:

```bash
# On the host:
export GITHUB_TOKEN=$(gh auth token)
nibble sandbox attach /path/to/repo --fresh
```

## Commands Reference

Gather these as needed — you don't need to run them all for every review.

```bash
# Dashboard — all pending reviews (cross-repo)
gh search prs --review-requested=@me --state open --json repository,number,title,url,author,updatedAt

# Current repo only (fallback)
gh pr list --search "review-requested:@me state:open" --json number,title,author,updatedAt,url,labels

# Current user login (for self-review detection)
gh api user --jq '.login'

# PR details
gh pr view <NUMBER> --repo <owner/repo> --json title,body,author,baseRefName,headRefName,additions,deletions,changedFiles,labels,reviews,comments,url,reviewDecision

# Diff
gh pr diff <NUMBER> --repo <owner/repo>

# Existing reviews and inline comments (peer-review dedup)
gh api repos/{owner}/{repo}/pulls/{number}/reviews
gh api repos/{owner}/{repo}/pulls/{number}/comments

# CI checks
gh pr checks <NUMBER> --repo <owner/repo>

# Conversation thread
gh pr view <NUMBER> --repo <owner/repo> --comments

# Latest commit SHA (needed for review submission)
gh pr view <NUMBER> --repo <owner/repo> --json headRefOid --jq '.headRefOid'

# Clone + checkout for local deep-dive
gh repo clone <owner/repo> /tmp/pr-review-<number>
cd /tmp/pr-review-<number> && gh pr checkout <NUMBER>
# Clean up when done: rm -rf /tmp/pr-review-<number>

# Submit review with body + inline comments (single atomic call)
gh api repos/{owner}/{repo}/pulls/{number}/reviews --input /tmp/review-payload.json
```

## Flow

### 1. Dashboard

Present pending PRs as a numbered table. Flag ones labeled `urgent`/`security`/`hotfix`,
stale PRs, or large PRs (>500 lines). User picks one to review.

### 2. Gather Context

Fetch PR details, diff, CI status, and conversation. Detect whether this is a
**self-review** (user's own PR — treat as pre-flight checklist: completeness,
CI, typos, debug leftovers) or a **peer review** (someone else's PR — full review).

For peer reviews, **always fetch existing reviews and inline comments** before
analyzing. Skip anything already flagged — no rehashing.

Present a summary that **always shows the PR URL prominently** (the user needs
to navigate to it easily). Include size, CI status, and a brief changes overview.

### 3. Review

Do a thorough review. If the diff lacks context, clone the repo locally to
read surrounding code, run tests, or check lint. Trust your judgement — dig
into what matters and skip what doesn't.

**Key areas:** correctness, edge cases, error paths, design fit, security,
performance, test coverage, naming, dead code. For AI/LLM code: also check
prompt bias, duplicated context across files, and over-optimization risk.

**Peer-review dig deeper:** when existing reviews already caught the obvious
issues, shift focus to design gaps, subtle edge cases, concurrency, missing
tests for implemented behavior, backward-compatibility breaks, and operational
concerns.

**If the architecture is fundamentally flawed**, flag that first — don't
nitpick variable names on a design that shouldn't exist in its current form.

When unsure about a domain, frame findings as questions rather than prescriptions.

### 4. Submit

**Always deliver a review with two layers** — a high-level body and inline
comments on specific lines. This is a single atomic submission via the Reviews
API.

**Fetch the head commit SHA first:**
```bash
gh pr view <NUMBER> --repo <owner/repo> --json headRefOid --jq '.headRefOid'
```

**Build the payload:**
```json
{
  "commit_id": "<headRefOid>",
  "body": "## Review\n\n<summary>\n\n### Blockers\n- 🔴 ...\n\n### Minor\n- 🔵 ...\n\nSee inline comments for details.\n\n---\n*🤖 nibble-generated review*",
  "event": "APPROVE | REQUEST_CHANGES | COMMENT",
  "comments": [
    {
      "path": "path/to/file.ts",
      "line": 115,
      "side": "RIGHT",
      "body": "🔴 **Critical — title**\n\nDescription + suggested fix."
    }
  ]
}
```

**Post it:**
```bash
cat <<'EOF' > /tmp/review-payload.json
{ ... }
EOF
gh api repos/<owner>/<repo>/pulls/<number>/reviews --input /tmp/review-payload.json
```

**Constraints:** Each inline comment must reference a line in the diff (added or
context). Use `side: "RIGHT"` and the line number from the PR branch. For
multi-line comments, include `start_line` and `start_side`.

**Always confirm with the user before submitting.** Show the review body and
inline comment list. Let them edit or approve.

### Severity Guide

| Emoji | Severity | When to use |
|-------|----------|-------------|
| 🔴 | Critical | Bugs, security vulnerabilities, data loss |
| 🟡 | Major | Design issues, missing error handling |
| 🔵 | Minor | Style, naming, small improvements |
| 💡 | Suggestion | Optional ideas, questions |

Always end review bodies with `---\n*🤖 nibble-generated review*`.
