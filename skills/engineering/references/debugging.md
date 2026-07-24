# Debugging

Find the root cause, not a symptom that silences it.

## Loop

1. **Reproduce reliably.** A flaky repro is half-fixed. Minimize the steps and
   the data that trigger it.
2. **Isolate.** Bisect (`git bisect`, input minimization, binary search over
   config). Narrow to the smallest input/config/change that reproduces.
3. **Hypothesize.** State a falsifiable hypothesis: "X is null because Y."
4. **Verify the hypothesis.** Add a check/log/test that proves it *before* fixing.
5. **Fix the smallest correct thing.** Don't refactor while debugging.
6. **Confirm the fix** and that you didn't break neighbors (run related tests).
7. **Add a regression test** that fails without the fix.

## Anti-patterns

- Changing things until the symptom disappears, without knowing why.
- Catching/ignoring the error to make it "work."
- Adding sleeps to fix timing bugs (the race is still there).
- Fixing the symptom upstream instead of the cause at the source.

## When stuck

- Read the error and the full stack trace; check the *first* error.
- Check recent changes (`git log`/`git diff`) and recent env/config changes.
- Explain the problem aloud or in writing — it often reveals the gap.
