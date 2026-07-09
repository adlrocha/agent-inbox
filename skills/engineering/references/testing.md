# Testing

Tests prove behavior. Write them from the spec or desired behavior, not by
mirroring implementation — otherwise refactors break tests for the wrong reason.

## What to test

- Every acceptance criterion / invariant at least once.
- **Boundaries:** empty collection, single element, min/max, off-by-one,
  null/undefined/NaN, very large input, unicode/whitespace.
- **Error paths:** every documented error case (bad input, missing resource,
  permission denied, timeout, partial failure).
- **State transitions:** valid, invalid, missing, and recovery transitions.

## How to test

- Name tests by behavior (`AC-3: rejects expired token`), not `test1`.
- One concept per test; independent and order-free.
- Test the public surface, not internals, unless internals are the contract.
- Deterministic: inject a clock/random/network stub. No sleep-based timing.

## Coverage

Coverage is a heuristic for "did I test this path", not a target. 100% coverage
with tests that mirror the code proves nothing. Prioritize risky branches and
boundary/error paths.

## Organization

Follow the repo's existing layout. If none:

```
tests/
  unit/<feature>.test.<ext>         # pure logic, fast
  integration/<feature>.test.<ext>  # real dependencies (DB, fs, http)
  fixtures/                         # shared data
```

Keep fast unit tests separate from slower integration/e2e tests.
