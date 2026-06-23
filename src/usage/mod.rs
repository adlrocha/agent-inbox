//! Token-usage tracking across Claude Code and pi CLI sessions.
//!
//! Walks the JSONL transcripts both CLIs write under `~/.claude/projects/`
//! and `~/.pi/agent/sessions/`, then upserts per-message token counts into
//! the nibble SQLite DB. A pricing table (TOML override or bundled defaults)
//! is applied to estimate hypothetical pay-as-you-go cost.

pub mod aggregate;
pub mod claude_log;
pub mod pi_log;
pub mod pricing;
pub mod report;

pub use aggregate::scan_all;
pub use pricing::PricingTable;
pub use report::{print_pricing, print_report, ReportFormat};
