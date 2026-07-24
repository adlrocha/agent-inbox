# Software Design (Ousterhout)

Design guidance from John Ousterhout, *A Philosophy of Software Design* (2nd ed.,
2021). Apply to any non-trivial design decision: a new module or API, layering,
splitting or merging code, or restructuring. For interface specifics see
`interface-design.md`; for errors see `error-handling.md`.

## The core problem: complexity

Complexity is anything that makes software hard to understand or modify. It has
two sources:

- **Dependencies** — B can't be understood or changed without knowing about A.
- **Obscurity** — important information isn't obvious (vague names, missing docs,
  hidden side effects, unclear invariants).

Complexity is **incremental**: it accretes through many small tactical
decisions, not one big mistake. Fight it on every change.

## Strategic vs tactical programming

The goal is a **good design**, not working code. "Just get it working"
(tactical programming) is the single largest source of complexity. Strategic
programming invests up front (~10-20% extra) in good design and reinvests each
time the code is touched. The job is designing, not typing.

## Principles

### Deep modules
A module's interface should be dramatically simpler than its implementation: a
small interface hiding a lot of machinery. **Shallow modules** (complex
interface, little behind it) leak complexity to callers. Do not split code into
many tiny methods/classes just to look "clean" — shallow modules and
pass-through methods add complexity, they don't remove it. *(Deliberate
disagreement with the small-functions / Clean Code school.)*

### Information hiding (the most important principle)
Modules hide implementation decisions. **Information leakage** — a design
decision reflected in more than one place — defeats hiding. If two modules both
encode knowledge of a format/protocol/assumption, that knowledge has leaked.

### General-purpose modules are Deeper
Design around the problem the module solves, not the one current caller.
Specialized-to-one-caller modules are shallow and leaky. Sweet spot: "somewhat
general-purpose" — broad enough to hide complexity, specific enough to stay
simple. Don't over-generalize into speculative frameworks.

### Different Layer, Different Abstraction
If a layer mostly forwards to the layer below (pass-through methods), the
abstraction is wrong. Each layer must earn its place by adding something real;
otherwise remove it.

### Pull Complexity Downward
Better for one module to absorb complexity behind a simple interface than to
spread that complexity across many callers. Don't push pain upward.

### Better Together Or Further Apart?
Bring code together when there is (a) substantial information sharing, (b) a
simple interface between the parts, or (c) only one caller. Otherwise keep it
apart. Splitting for "Single Responsibility" that creates shallow modules or
leaks information is a net loss.

### Comments are part of design
- Comments compensate for the information lost when intent is encoded in code.
- Describe what **isn't** obvious: why, the abstraction, invariants, interface
  contracts. Do not restate the code.
- **Comment-first**: write the interface comment before the implementation.
- Good comments let modules be deeper — the comment carries the abstraction.

### Define Errors Out Of Existence
Exceptions are complexity (inverted control flow, duplicated handling,
obscurity). Minimize them: redefine semantics so the case isn't an error; mask
or absorb exceptions in deep modules; don't propagate generically; specialize at
boundaries. (See `error-handling.md`.)

### Design It Twice
For any non-trivial problem, sketch at least two different designs before
committing. The first idea is rarely best; comparing alternatives surfaces
hidden assumptions.

### Decide What Matters (2nd ed.)
Separate what is important from what isn't, and focus design effort on the
important parts. Not every decision deserves deep design investment.

### Consistency, obviousness, names
- Consistency across the codebase lowers obscurity — follow existing patterns.
- Code should be obvious; cleverness costs more than it saves.
- Names should convey the full meaning of the thing.

## Design loop

1. State the problem and the real constraints.
2. **Design it twice** — sketch ≥2 approaches; compare for depth and leaked info.
3. Prefer deep, general-purpose modules that hide information.
4. Write interface comments first; if you can't describe one simply, the
   abstraction is wrong.
5. Minimize exceptions (define them out of existence).
6. When modifying existing code, invest in keeping — improving — the design.
   Don't patch tactically.

> Source: John Ousterhout, *A Philosophy of Software Design*, 2nd ed. (2021).
> Where this differs from popular conventions (small functions, self-documenting
> code, explicit error propagation), this reference follows Ousterhout.
