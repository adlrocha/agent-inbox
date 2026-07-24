---
name: engineering
description: General-purpose software engineering best practices — code review, testing, debugging, refactoring, error handling, security, interface design. Stack-neutral; routes to stack-specific guidance. Load for non-trivial engineering work.
---

# Engineering Practices

Stack-neutral engineering discipline. Apply the universal principles, then load
the matching **topic reference** for the kind of work, and the matching **stack
reference** for stack-specific patterns.

## Universal principles

1. **Design strategically; complexity is the enemy.** Complexity (dependencies
   + obscurity) accretes one tactical choice at a time — a good design is the
   goal, not just working code. For non-trivial design decisions, load
   `references/design.md` (Ousterhout).
2. **Understand before changing.** Read the relevant code and tests. State the
   current behavior before proposing a change. Don't guess.
3. **Smallest correct change.** Touch only what the task requires. Prefer edits
   over rewrites. Match existing style.
4. **Verify before done.** Run the relevant tests / lint / build. Prove the
   change works and didn't break neighbors. State only verified facts.
5. **Tests capture behavior, not implementation.** Write tests from the spec or
   desired behavior, not by mirroring the code's internals.
6. **Minimize exceptions.** Design errors out of existence and absorb them in
   deep modules rather than propagating (see `references/error-handling.md`).
   Never swallow silently.
7. **Security at trust boundaries.** Validate/encode input crossing a boundary.
   Least privilege. Secrets stay out of code and logs.
8. **Clean history.** One logical change per commit; follow repo conventions —
   load the `git-commit-pr` skill for the commit/PR workflow.

## Topic references (load the one matching the task)

- `references/design.md` — Ousterhout design principles (deep modules, information hiding, complexity)
- `references/code-review.md` — reviewing your own or others' changes
- `references/testing.md` — what to test, boundaries, coverage, organization
- `references/debugging.md` — reproduce, isolate, root-cause, regression-test
- `references/refactoring.md` — behavior-preserving, small steps
- `references/interface-design.md` — contracts, errors, naming, compatibility
- `references/error-handling.md` — typed errors, propagation, cleanup
- `references/security-basics.md` — trust boundaries, injection, authz, secrets

## Stack routing

Identify the stack you're working in, then load the matching reference for
framework conventions, patterns, and common pitfalls. **Currently supported
stacks:**

- **backend** → `references/stacks/backend.md` (services, APIs, databases, jobs)
- **frontend** → `references/stacks/frontend.md` (web UI, components, state, a11y)

If the stack isn't listed, apply the universal principles and the closest topic
reference. **Adding a stack:** create `references/stacks/<name>.md` and add a
line to the list above.
