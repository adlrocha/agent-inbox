# Security Basics

Threat-model briefly: what is the trust boundary, who can cross it, what do they
control. Then validate at the boundary and least-privilege everything else.

## Input crossing a trust boundary

- **Validate** (type/range/length/structure) and **encode/parameterize** on use.
- **Parameterize queries.** Never string-concat SQL / shell commands / HTML.
- **Limit** size/count/time to blunt abuse and DoS.

## Authn vs authz

- Authentication = *who* is this. Authorization = *may* they do this.
- Check authz on every sensitive action, not just the entry point. Re-check on
  state-changing operations.
- Fail closed: on error or ambiguity, deny.

## Secrets

- Secrets live in env / a secret store — never in code, commits, logs, or error
  messages.
- Don't echo tokens/passwords in messages or stack traces.
- Rotate; scope tokens to least privilege; expire them.

## Common pitfalls

- **Injection** — SQL, command, path traversal, LDAP, template, log. Parameterize
  and canonicalize paths.
- **XSS** — encode output; never interpolate untrusted data into HTML/JS.
- **CSRF** — anti-CSRF tokens on state-changing requests.
- **IDOR** — authorize by ownership, not by obscured IDs.
- **Insecure defaults** — verify TLS, don't disable certificate validation, set
  secure cookie flags.

## When unsure

If the feature touches auth, payments, PII, or external input, slow down, state
your assumptions, and ask for review. (See `debugging.md` for root-causing
incidents.)
