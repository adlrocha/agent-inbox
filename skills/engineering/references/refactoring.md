# Refactoring

Change structure without changing behavior. Never mix refactor and behavior
change in one step — if you must, do them as separate commits. Refactoring is
**strategic investment** in the design (Ousterhout): when you restructure, aim
for deeper modules and less information leakage — **design it twice** — not just
less code.

## Pre-conditions

- **Characterize first.** Have tests that pin current behavior. If none exist,
  write characterization tests before refactoring.
- Green tests before you start.

## Discipline

- **Small steps.** One rename / extract / inline at a time. Run tests after each.
- **Preserve behavior.** If tests change behavior, you're not refactoring — split
  it out into a separate change.
- **Prefer tooling** (safe renames, automated extracts) over hand edits.
- **Don't fix unrelated things** you notice along the way; note them and handle
  them separately.

## Common moves

- **Extract** a function/module when logic is duplicated or a block has its own
  intent.
- **Inline** when indirection adds cost without clarity.
- **Rename** so the name reveals intent, not the type.
- **Move** code closer to its data/dependents.

## Stop when

Tests are green, behavior is unchanged, and the next step isn't justified by the
task. Don't refactor speculatively.
