//! Pretty-print token usage summaries.

use anyhow::Result;
use chrono::{Local, TimeZone, Utc};

use crate::db::Database;
use crate::db::TokenUsageSummaryRow;
use crate::usage::pricing::PricingTable;

#[derive(Debug, Clone, Copy)]
pub enum ReportFormat {
    Table,
    Json,
}

/// Parse a duration like "7d", "24h", "30m". Returns seconds.
pub fn parse_since(s: &str) -> Option<i64> {
    let s = s.trim();
    if s.is_empty() {
        return None;
    }
    let (num, unit) = s.split_at(s.len() - 1);
    let n: i64 = num.parse().ok()?;
    let secs = match unit {
        "s" => n,
        "m" => n * 60,
        "h" => n * 3_600,
        "d" => n * 86_400,
        "w" => n * 604_800,
        _ => return None,
    };
    Some(secs)
}

pub fn print_report(
    db: &Database,
    group_by: &str,
    since: Option<&str>,
    format: ReportFormat,
) -> Result<()> {
    let now = Utc::now().timestamp();
    let since_ts = match since.and_then(parse_since) {
        Some(secs) => now - secs,
        None => 0,
    };
    let (rows, window) = db.token_usage_summary(group_by, since_ts)?;
    let pricing = PricingTable::load()?;

    match format {
        ReportFormat::Json => {
            let arr: Vec<_> = rows
                .iter()
                .map(|r| {
                    let priced = is_priced(&pricing, r);
                    serde_json::json!({
                        "bucket": r.bucket,
                        "provider": r.provider,
                        "model": r.model,
                        "api_provider": r.api_provider,
                        "input_tokens": r.input_tokens,
                        "output_tokens": r.output_tokens,
                        "cache_read_tokens": r.cache_read_tokens,
                        "cache_write_tokens": r.cache_write_tokens,
                        "estimated_cost_usd": r.estimated_cost_usd,
                        "messages": r.message_count,
                        "priced": priced,
                    })
                })
                .collect();
            println!(
                "{}",
                serde_json::to_string_pretty(&serde_json::json!({
                    "window": window.map(|(lo, hi)| serde_json::json!({
                        "from_ts": lo,
                        "to_ts": hi,
                        "from": fmt_ts(lo),
                        "to": fmt_ts(hi),
                    })),
                    "since_filter": since,
                    "rows": arr,
                }))?
            );
        }
        ReportFormat::Table => print_table(&pricing, &rows, window, since, group_by),
    }
    Ok(())
}

fn print_table(
    pricing: &PricingTable,
    rows: &[TokenUsageSummaryRow],
    window: Option<(i64, i64)>,
    since: Option<&str>,
    group_by: &str,
) {
    // ── Layout: adapt to terminal width, but never below a usable minimum ─────
    let Layout {
        width,
        bucket,
        price,
        input,
        output,
        cache_r,
        cache_w,
        msgs,
        cost,
    } = Layout::compute();

    // ── Header ────────────────────────────────────────────────────────────────
    println!();
    println!("┌{}┐", "─".repeat(width - 2));
    println!(
        "│ {:<w$}│",
        format!(" nibble usage report — grouped by {}", group_by),
        w = width - 3
    );
    let win_line = match window {
        Some((lo, hi)) => format!(
            "  window: {}  →  {}    ({})",
            fmt_ts_local(lo),
            fmt_ts_local(hi),
            human_span(hi - lo),
        ),
        None => "  window: (no data)".to_string(),
    };
    println!("│ {:<w$}│", win_line, w = width - 3);
    if let Some(s) = since {
        println!(
            "│ {:<w$}│",
            format!("  filter: --since {}", s),
            w = width - 3
        );
    }
    println!("└{}┘", "─".repeat(width - 2));

    if rows.is_empty() {
        println!("\n(no token usage recorded — run `nibble usage scan` first)");
        return;
    }

    // ── Column header ─────────────────────────────────────────────────────────
    println!();
    println!(
        "{:<bucket$} {:>price$} {:>input$} {:>output$} {:>cache_r$} {:>cache_w$} {:>msgs$} {:>cost$}",
        "BUCKET", "PRICE/1M", "INPUT", "OUTPUT", "CACHE_R", "CACHE_W", "MSGS", "COST_USD",
        bucket = bucket,
        price = price,
        input = input,
        output = output,
        cache_r = cache_r,
        cache_w = cache_w,
        msgs = msgs,
        cost = cost,
    );
    println!("{}", "─".repeat(width));

    // ── Body, with separators between top-level providers ────────────────────
    let mut tot_in = 0i64;
    let mut tot_out = 0i64;
    let mut tot_cr = 0i64;
    let mut tot_cw = 0i64;
    let mut tot_msgs = 0i64;
    let mut tot_cost = 0.0f64;
    let mut last_provider: Option<&str> = None;
    let mut unpriced: Vec<&TokenUsageSummaryRow> = Vec::new();

    for r in rows {
        if let Some(prev) = last_provider {
            if prev != r.provider {
                println!("{}", "·".repeat(width));
            }
        }
        last_provider = Some(&r.provider);

        let priced = is_priced(pricing, r);
        let cost_cell = if priced {
            format!("{:>cost$}", fmt_money(r.estimated_cost_usd))
        } else if has_tokens(r) {
            unpriced.push(r);
            format!("{:>cost$}", "n/a")
        } else {
            format!("{:>cost$}", fmt_money(0.0))
        };

        println!(
            "{:<bucket$} {:>price$} {:>input$} {:>output$} {:>cache_r$} {:>cache_w$} {:>msgs$} {}",
            truncate(&r.bucket, bucket),
            fmt_price_rate(pricing, r, price),
            fmt_int(r.input_tokens),
            fmt_int(r.output_tokens),
            fmt_int(r.cache_read_tokens),
            fmt_int(r.cache_write_tokens),
            fmt_int(r.message_count),
            cost_cell,
            bucket = bucket,
            price = price,
            input = input,
            output = output,
            cache_r = cache_r,
            cache_w = cache_w,
            msgs = msgs,
        );
        tot_in += r.input_tokens;
        tot_out += r.output_tokens;
        tot_cr += r.cache_read_tokens;
        tot_cw += r.cache_write_tokens;
        tot_msgs += r.message_count;
        tot_cost += r.estimated_cost_usd;
    }
    println!("{}", "═".repeat(width));
    println!(
        "{:<bucket$} {:>price$} {:>input$} {:>output$} {:>cache_r$} {:>cache_w$} {:>msgs$} {:>cost$}",
        "TOTAL (priced only)",
        "",
        fmt_int(tot_in),
        fmt_int(tot_out),
        fmt_int(tot_cr),
        fmt_int(tot_cw),
        fmt_int(tot_msgs),
        fmt_money(tot_cost),
        bucket = bucket,
        price = price,
        input = input,
        output = output,
        cache_r = cache_r,
        cache_w = cache_w,
        msgs = msgs,
        cost = cost,
    );

    // ── Unpriced footer ───────────────────────────────────────────────────────
    if !unpriced.is_empty() {
        println!();
        println!(
            "⚠  {} model(s) have no price entry — cost shown as n/a. Add entries to:",
            unpriced.len()
        );
        println!(
            "   {}",
            crate::usage::pricing::default_pricing_path().display()
        );
        println!();
        println!("   Unpriced (lookup key shown as `<api_provider>.<model>`):");
        for r in &unpriced {
            println!(
                "     • {:<30} ({} msgs, {} in / {} out tokens)",
                format!(
                    "{}.{}",
                    r.api_provider.as_deref().unwrap_or("?"),
                    r.model.as_deref().unwrap_or(&r.bucket),
                ),
                r.message_count,
                fmt_int(r.input_tokens),
                fmt_int(r.output_tokens),
            );
        }
    }
    println!();
}

fn is_priced(pricing: &PricingTable, r: &TokenUsageSummaryRow) -> bool {
    let (Some(model), Some(api)) = (r.model.as_deref(), r.api_provider.as_deref()) else {
        // Not grouped by model — cost was already aggregated; trust the stored value.
        return r.estimated_cost_usd > 0.0 || !has_tokens(r);
    };
    pricing.lookup(api, model).is_some()
}

fn has_tokens(r: &crate::db::TokenUsageSummaryRow) -> bool {
    r.input_tokens + r.output_tokens + r.cache_read_tokens + r.cache_write_tokens > 0
}

pub fn print_pricing(pricing: &PricingTable) -> Result<()> {
    println!(
        "Pricing source: {} (override via {})",
        if crate::usage::pricing::default_pricing_path().exists() {
            "user TOML + bundled defaults"
        } else {
            "bundled defaults (no user TOML found)"
        },
        crate::usage::pricing::default_pricing_path().display(),
    );
    println!();
    println!(
        "{:<14} {:<28} {:>10} {:>10} {:>12} {:>12}",
        "PROVIDER", "MODEL", "INPUT", "OUTPUT", "CACHE_R", "CACHE_W"
    );
    println!("{}", "─".repeat(90));
    let mut providers: Vec<&String> = pricing.providers.keys().collect();
    providers.sort();
    for prov in providers {
        let models = &pricing.providers[prov];
        let mut names: Vec<&String> = models.keys().collect();
        names.sort();
        for name in names {
            let p = &models[name];
            println!(
                "{:<14} {:<28} {:>10.2} {:>10.2} {:>12.2} {:>12.2}",
                prov, name, p.input, p.output, p.cache_read, p.cache_write
            );
        }
    }
    println!();
    println!("(Prices are USD per 1M tokens.)");
    Ok(())
}

// ── Formatters ───────────────────────────────────────────────────────────────

/// Column layout computed from the terminal width.
#[derive(Debug, Clone, Copy)]
struct Layout {
    width: usize,
    bucket: usize,
    price: usize,
    input: usize,
    output: usize,
    cache_r: usize,
    cache_w: usize,
    msgs: usize,
    cost: usize,
}

impl Layout {
    /// Minimum usable table width.
    const MIN_WIDTH: usize = 105;

    /// Compute column widths. Bucket expands to use free terminal space;
    /// numeric columns keep fixed compact widths.
    fn compute() -> Self {
        let term = term_width().max(Self::MIN_WIDTH);
        let price = 13;
        let input = 11;
        let output = 11;
        let cache_r = 11;
        let cache_w = 11;
        let msgs = 7;
        let cost = 11;
        // 7 spaces between the 8 columns.
        let fixed = price + input + output + cache_r + cache_w + msgs + cost + 7;
        let bucket = (term - fixed).clamp(20, 55);
        let width = bucket + fixed;
        Self {
            width,
            bucket,
            price,
            input,
            output,
            cache_r,
            cache_w,
            msgs,
            cost,
        }
    }
}

/// Return the terminal width in columns, or a sensible default.
fn term_width() -> usize {
    #[cfg(unix)]
    unsafe {
        let mut ws: libc::winsize = std::mem::zeroed();
        if libc::ioctl(libc::STDOUT_FILENO, libc::TIOCGWINSZ, &mut ws) == 0 && ws.ws_col > 0 {
            return ws.ws_col as usize;
        }
    }
    std::env::var("COLUMNS")
        .ok()
        .and_then(|s| s.parse().ok())
        .unwrap_or(108)
}

/// Format the per-1M input/output price rate applied to a row.
fn fmt_price_rate(pricing: &PricingTable, r: &TokenUsageSummaryRow, width: usize) -> String {
    let (Some(model), Some(api)) = (r.model.as_deref(), r.api_provider.as_deref()) else {
        return "-".to_string();
    };
    let s = pricing.lookup(api, model).map_or_else(
        || "-".to_string(),
        |p| format!("{:.2}/{:.2}", p.input, p.output),
    );
    truncate(&s, width)
}

fn fmt_int(n: i64) -> String {
    let s = n.abs().to_string();
    let bytes = s.as_bytes();
    let mut out = String::with_capacity(s.len() + s.len() / 3);
    if n < 0 {
        out.push('-');
    }
    let first_chunk = bytes.len() % 3;
    for (i, b) in bytes.iter().enumerate() {
        if i != 0 && i >= first_chunk && (i - first_chunk).is_multiple_of(3) {
            out.push(',');
        }
        out.push(*b as char);
    }
    out
}

fn fmt_money(v: f64) -> String {
    if v == 0.0 {
        return "$0.00".to_string();
    }
    if v >= 1000.0 {
        format!("${}", fmt_int(v.round() as i64))
    } else {
        format!("${:.2}", v)
    }
}

fn fmt_ts(ts: i64) -> String {
    Utc.timestamp_opt(ts, 0)
        .single()
        .map(|d| d.to_rfc3339())
        .unwrap_or_else(|| ts.to_string())
}

fn fmt_ts_local(ts: i64) -> String {
    Local
        .timestamp_opt(ts, 0)
        .single()
        .map(|d| d.format("%Y-%m-%d %H:%M %Z").to_string())
        .unwrap_or_else(|| ts.to_string())
}

fn human_span(secs: i64) -> String {
    if secs < 60 {
        format!("{}s", secs)
    } else if secs < 3600 {
        format!("{}m", secs / 60)
    } else if secs < 86_400 {
        format!("{}h {}m", secs / 3600, (secs % 3600) / 60)
    } else {
        let d = secs / 86_400;
        let h = (secs % 86_400) / 3600;
        format!("{}d {}h", d, h)
    }
}

fn truncate(s: &str, n: usize) -> String {
    if s.chars().count() <= n {
        s.to_string()
    } else {
        let mut out: String = s.chars().take(n.saturating_sub(1)).collect();
        out.push('…');
        out
    }
}
