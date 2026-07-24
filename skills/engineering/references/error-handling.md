# Error Handling (Ousterhout: minimize exceptions)

Exceptions are complexity: they invert normal control flow, force every caller
to consider another path, and obscure the happy path. Ousterhout's directive:
**minimize exceptions** — design so fewer things are errors in the first place,
and absorb the rest inside deep modules.

## Reduce exceptions

- **Define errors out of existence.** Change the semantics so the case isn't an
  error: a lookup returns empty instead of throwing "not found"; a buffer that
  can't flush yet holds the data; a resize clamps instead of rejecting.
- **Mask exceptions in deep modules.** If a caller can't usefully act on an
  internal failure (transient I/O, a lower-layer hiccup), the deep module
  absorbs it (retry, default, reconcile) and exposes at most a coarse signal.
- **Don't propagate generically.** Each layer that rethrows duplicates handling
  and leaks implementation upward. Convert or specialize at the boundary into
  the caller's vocabulary — or absorb.

## When an error must surface

- Represent it as a typed value (Result / Option / domain enum / typed exception
  per language idiom) and surface only what the caller needs to act on.
- Handle at the layer that can actually recover (retry, fallback, ask the user).

## Still mandatory (these are *not* "swallowing")

- **Clean up on every path** — roll back transactions, close handles, release
  locks. Prefer RAII / `try`/`finally` / `defer`.
- **Never swallow silently.** Masking is *intentional* absorption with a
  documented reason; an empty catch that hides a bug is not masking, it's a bug.
- **Fail closed** on security / authz paths (deny on ambiguity). This is a
  safety rule, independent of the error philosophy.

## Reconciliation note

This **revises** the older "propagate with context / fail loud" stance toward
Ousterhout's "absorb / define away." Teams that use explicit, mandatory error
types (e.g. Rust `Result`, checked exceptions) should set house style: the goal
is shared — fewer error paths crossing module boundaries, each one earning its
place. See `design.md` for the surrounding philosophy.
