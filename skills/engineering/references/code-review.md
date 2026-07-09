# Code Review

Review for correctness first, then robustness, then clarity. Review your own
change before requesting review; review others' changes with specificity and
respect.

## What to check

- **Correctness** — does it do what the task/spec says? Off-by-one, inverted
  conditions, wrong default, mutation of shared state, race conditions.
- **Edge cases** — empty/null/missing, min/max, unicode, large input, concurrent
  access, partial failure.
- **Error handling** — are expected errors typed and handled? Are unexpected
  errors propagated with context? Anything swallowed?
- **Security** — input crossing a trust boundary validated/encoded? Authz
  checked? Secrets or log injection? (Load `security-basics.md`.)
- **Tests** — do they cover the new behavior and its failure modes? Testing
  behavior, not implementation? (Load `testing.md`.)
- **Design depth (Ousterhout)** — are modules deep (simple interface, rich
  implementation) or shallow/pass-through? Has information leaked across
  boundaries? Are there too many tiny classes/functions? (Load `design.md`.)
- **Comments** — do comments describe the non-obvious (why, invariants,
  contracts)? Flag comments that restate the code, or tricky code missing a
  why-comment.
- **Clarity** — names reveal intent? Anything clever that needs a comment? Dead
  code, unused params, leftover debug?

## Giving feedback

- Specific and actionable: file:line, what's wrong, suggested fix.
- Separate "must fix" (correctness/security) from "consider" (style).
- Praise good decisions; don't nitpick style the repo doesn't enforce.

## Self-review before requesting review

1. Read the full diff.
2. Run tests + lint + typecheck.
3. Check for TODOs, debug logs, commented-out code, accidental changes.
4. Write a description that explains *why*, not just *what*.
