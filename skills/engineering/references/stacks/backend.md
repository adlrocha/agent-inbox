# Backend (services, APIs, databases, jobs)

Guidance for server-side work. Adapt framework specifics to your runtime
(Node/Fastify, Python, Go, Rust, etc.). Seeded from a Node/API/DB stack and
generalized.

## Layering

- Keep routes/handlers thin: parse → authorize → call service → serialize.
- Put workflows in services, persistence in repositories. Don't reach into the
  DB from the handler.
- Move shared logic to a package only when ≥2 consumers need it.

## Data & persistence

- Own schema via migrations; never edit the DB by hand in prod.
- One transaction per unit of work; roll back on error. Don't hold transactions
  open across network calls.
- Watch N+1: fetch relations explicitly or batch. Avoid unbounded queries
  (paginate/cursor).
- Validate at the boundary; store normalized; derive display formats at the edge.

## API design

- Idempotent where it matters (payments, retries). Use idempotency keys.
- Paginate list endpoints; cap page size.
- Consistent error shape (code, message, details). Use correct status codes;
  don't leak internals.
- Version breaking changes; deprecate before removing.

## Background jobs & queues

- Jobs are idempotent and retriable; assume they may run twice.
- Don't do long work in the request path — enqueue it.
- Dead-letter and alert on poison messages; don't silently drop.

## Auth

- Authenticate at the edge; authorize at the action.
- Centralize authz checks; don't scatter `if isAdmin`.
- Tokens: short-lived, scoped, rotated.

## Observability & config

- Structured logs with request/correlation IDs. Log events, not raw objects.
- Config from env at boot, validated; fail fast on missing required config.
- Health + readiness endpoints.

## Adapt to your stack

- Node/TypeScript: respect package/workspace ownership; don't redeclare deps.
- Your framework may add conventions (middleware order, DI). Follow them over
  inventing your own.
