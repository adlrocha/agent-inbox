//! Scan claude + pi transcripts and upsert per-message usage rows.

use anyhow::Result;

use crate::db::Database;
use crate::usage::pricing::PricingTable;
use crate::usage::{claude_log, pi_log};

#[derive(Debug, Default, Clone, Copy)]
pub struct ScanStats {
    pub claude_seen: u64,
    pub claude_inserted: u64,
    pub pi_seen: u64,
    pub pi_inserted: u64,
}

pub fn scan_all(db: &Database, pricing: &PricingTable) -> Result<ScanStats> {
    let mut stats = ScanStats::default();

    claude_log::iter_records(|r| {
        stats.claude_seen += 1;
        // Claude Code transcripts come from Anthropic.
        let api_provider = "anthropic";
        let cost = pricing.estimate_cost(
            api_provider,
            &r.model,
            r.input_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.cache_write_tokens,
        );
        match db.upsert_token_usage(
            "claude",
            Some(api_provider),
            &r.model,
            &r.session_id,
            &r.message_id,
            r.ts,
            r.cwd.as_deref(),
            r.input_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.cache_write_tokens,
            cost,
        ) {
            Ok(true) => stats.claude_inserted += 1,
            Ok(false) => {}
            Err(e) => eprintln!("nibble usage: claude upsert failed: {e}"),
        }
    })?;

    pi_log::iter_records(|r| {
        stats.pi_seen += 1;
        let api_provider = r.api_provider.as_deref().unwrap_or("");
        // Prefer pi's self-reported cost when present; otherwise fall back to
        // our pricing table (useful if you fill in pricing for a free model
        // to estimate "what would this have cost on a paid provider").
        let cost = if r.reported_cost_usd > 0.0 {
            r.reported_cost_usd
        } else {
            pricing.estimate_cost(
                api_provider,
                &r.model,
                r.input_tokens,
                r.output_tokens,
                r.cache_read_tokens,
                r.cache_write_tokens,
            )
        };
        match db.upsert_token_usage(
            "pi",
            r.api_provider.as_deref(),
            &r.model,
            &r.session_id,
            &r.message_id,
            r.ts,
            r.cwd.as_deref(),
            r.input_tokens,
            r.output_tokens,
            r.cache_read_tokens,
            r.cache_write_tokens,
            cost,
        ) {
            Ok(true) => stats.pi_inserted += 1,
            Ok(false) => {}
            Err(e) => eprintln!("nibble usage: pi upsert failed: {e}"),
        }
    })?;

    Ok(stats)
}
