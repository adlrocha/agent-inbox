# Frontend (web UI)

Guidance for browser-facing UI — components, state, data fetching, forms,
accessibility, performance. Adapt to your framework (Astro/React, Vue, Svelte,
etc.). Seeded from an Astro/React/Tailwind stack and generalized.

## Component boundaries

- Components own one concern. Props in, events out; minimize two-way binding.
- Keep data fetching and business logic out of presentational components where
  possible.
- Colocate styles/tests with the component.

## State

- Push state up only as far as the closest common ancestor. Don't globalize what
  one subtree needs.
- Distinguish **server state** (cacheable, fetched) from **UI state** (local,
  ephemeral). Use the right tool for each.
- Derive, don't duplicate: if `B = f(A)`, don't store `B`.

## Data fetching & forms

- Handle loading / error / empty / success explicitly in the UI.
- Validate forms on submit (and optionally inline); show errors near the field.
- Encode user input on render (framework auto-escaping) — never `innerHTML`
  untrusted data.

## Accessibility

- Use semantic elements (`button`, `label`, `nav`). Real focus management.
- Every interactive element is keyboard reachable; keep a visible focus state.
- Sufficient color contrast; don't rely on color alone.
- Alt text for meaningful images; use `aria` only when semantics are insufficient.

## i18n

- No hardcoded user-facing strings once i18n exists. Use message keys.
- Format dates/numbers/currency per locale; don't roll your own.
- Pluralization rules differ by language — use the intl library.

## Performance

- Don't ship large bundles: code-split routes, lazy-load heavy widgets.
- Avoid layout thrash and unnecessary re-renders; memoize deliberately.
- Images: sized, lazy-loaded, modern formats.
- Measure before optimizing.

## Adapt to your framework

- React: island/server-component boundaries, effect dependencies, stable keys.
- Astro: server vs client islands, hydration directives.
- Follow the repo's existing patterns over inventing new ones.
