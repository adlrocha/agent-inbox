# Blueprint: pricing.toml path + model price population

## Summary
Move the nibble usage pricing override file from `~/.config/nibble/pricing.toml` to `~/.nibble/pricing.toml` so it lives alongside all other nibble state. Populate that file (and update bundled defaults) with current USD-per-1M-token prices for every model appearing in the user's usage report.

## What's Changing
- `src/usage/pricing.rs`
  - `default_pricing_path()` returns `~/.nibble/pricing.toml` instead of `~/.config/nibble/pricing.toml`.
  - Doc comments updated to reference the new path.
  - Bundled Anthropic defaults refreshed to current June 2026 pricing (Opus dropped from $15/$75 to $5/$25; Fable 5 and Opus 4.8 added).
- `src/usage/mod.rs`
  - Removed unused re-export `default_pricing_path` surfaced by the path change.
- `src/usage/report.rs`
  - Fixed an integer-underflow bug in `fmt_int` that panicked when formatting certain token/cost numbers with non-zero first-chunk lengths.
  - Added a `PRICE/1M` column immediately to the right of `BUCKET` showing the input/output price rate used for that row.
  - Table width is now detected from the terminal via `TIOCGWINSZ` and the bucket column expands to fill available space (minimum width enforced for usability).
- `src/db/mod.rs`
  - `upsert_token_usage` now refreshes token/cost fields when it encounters an existing row, so pricing-table updates are reflected for historical records on the next scan.
- `install.sh`
  - Pricing stub is written to `~/.nibble/pricing.toml` instead of `~/.config/nibble/pricing.toml`.
- `docs/usage-tracking.md`
  - Updated all references from `~/.config/nibble/pricing.toml` to `~/.nibble/pricing.toml`.
  - Documented that re-scans now refresh existing rows, so a full table clear is no longer required after price changes.
- `~/.nibble/pricing.toml`
  - Created with per-provider/model entries for all models in the report, sourced from each provider's public pricing page or from OpenRouter as a general-purpose fallback.

## Invariants
1. (INV-1) `nibble usage pricing` and `nibble usage report` load overrides from `~/.nibble/pricing.toml`; a missing file falls back to bundled defaults.
2. (INV-2) Anthropic bundled defaults match the prices shown on `https://www.anthropic.com/pricing#api` as of 2026-06-22.
3. (INV-3) Every non-synthetic model bucket in the supplied usage report has a corresponding pricing entry so its cost is no longer reported as `n/a`.
4. (INV-4) Re-scanning after a pricing-table change updates estimated costs for existing rows (no `n/a` or stale `$0.00` for priced models).

## Acceptance Criteria
1. (AC-1) After the change, `cargo build` succeeds and `cargo test` passes.
2. (AC-2) Running `nibble usage pricing` reports the override source as `~/.nibble/pricing.toml` and lists populated prices for all models.
3. (AC-3) Running `nibble usage report --since 4w` shows non-`n/a` cost estimates for every model bucket that has tokens, and the grand total reflects current pricing.
