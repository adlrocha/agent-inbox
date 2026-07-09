# Interface Design

An interface is a contract — and per Ousterhout a module's **depth** is how much
simpler its interface is than its implementation. Aim for deep modules. (Full
philosophy in `design.md`.)

## Depth & hiding (the core)

- **Deep modules:** a small interface hiding a lot of machinery. This is the
  goal. Shallow modules (complex interface, little behind it) leak complexity to
  callers — avoid them even if they look "clean."
- **Hide information:** the interface must not leak implementation decisions
  (formats, protocols, internal structures). If a caller must know an internal
  detail to use the module, that detail has leaked.
- **Different layer, different abstraction:** if an interface just forwards to
  the layer below, remove the layer.
- **Pull complexity downward:** absorb hard cases inside the module behind a
  simple interface rather than exposing them to every caller.
- **General-purpose:** design for the problem, not the one current caller — but
  stop at "somewhat general-purpose," not speculative.

## The contract

- **Make wrong states unrepresentable** via types/enums so invalid combinations
  can't be constructed.
- **Fail fast:** validate at the boundary; reject bad input immediately with a
  clear error.
- **Name by intent**, not by type or storage (`parseToken`, not `str2obj`).
- **Least surprise:** match language / stdlib / framework conventions.
- **Minimize exceptions:** prefer semantics where the case isn't an error (see
  `error-handling.md`); where an error is real, declare *how* it fails.

## Versioning & compatibility

- Additive changes (new optional params, new enum values) are safe; removals,
  renames, or semantic changes are breaking.
- For public APIs: version or deprecate before removing.

## When designing

1. List the callers and their needs before the signature.
2. **Design it twice** — sketch another interface and compare depth and leakage.
3. Write the interface comment first; if you can't describe it simply, the
   abstraction is wrong.
