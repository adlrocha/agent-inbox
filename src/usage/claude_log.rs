//! Parser for Claude Code session transcripts at
//! `~/.claude/projects/<slug>/<sessionId>.jsonl`.

use anyhow::Result;
use serde::Deserialize;
use std::path::PathBuf;

#[derive(Debug, Deserialize)]
struct Line {
    #[serde(rename = "type")]
    ty: Option<String>,
    message: Option<Message>,
    uuid: Option<String>,
    #[serde(rename = "sessionId")]
    session_id: Option<String>,
    timestamp: Option<String>,
    cwd: Option<String>,
}

#[derive(Debug, Deserialize)]
struct Message {
    model: Option<String>,
    usage: Option<Usage>,
}

#[derive(Debug, Deserialize)]
struct Usage {
    #[serde(default)]
    input_tokens: i64,
    #[serde(default)]
    output_tokens: i64,
    #[serde(default)]
    cache_creation_input_tokens: i64,
    #[serde(default)]
    cache_read_input_tokens: i64,
}

#[derive(Debug, Clone)]
pub struct UsageRecord {
    pub model: String,
    pub message_id: String,
    pub session_id: String,
    pub ts: i64,
    pub cwd: Option<String>,
    pub input_tokens: i64,
    pub output_tokens: i64,
    pub cache_read_tokens: i64,
    pub cache_write_tokens: i64,
}

pub fn projects_root() -> PathBuf {
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".claude").join("projects")
}

pub fn iter_records<F>(mut sink: F) -> Result<()>
where
    F: FnMut(UsageRecord),
{
    let root = projects_root();
    if !root.exists() {
        return Ok(());
    }
    for entry in walkdir::WalkDir::new(&root)
        .into_iter()
        .filter_map(|e| e.ok())
    {
        let path = entry.path();
        if path.extension().and_then(|s| s.to_str()) != Some("jsonl") {
            continue;
        }
        let file = match std::fs::File::open(path) {
            Ok(f) => f,
            Err(_) => continue,
        };
        use std::io::{BufRead, BufReader};
        let reader = BufReader::new(file);
        for line in reader.lines().map_while(Result::ok) {
            let line = line.trim();
            if line.is_empty() {
                continue;
            }
            let Ok(parsed) = serde_json::from_str::<Line>(line) else {
                continue;
            };
            if parsed.ty.as_deref() != Some("assistant") {
                continue;
            }
            let Some(msg) = parsed.message else { continue };
            let Some(usage) = msg.usage else { continue };
            let Some(model) = msg.model else { continue };
            let Some(message_id) = parsed.uuid else {
                continue;
            };
            let Some(session_id) = parsed.session_id else {
                continue;
            };
            let ts = parsed.timestamp.as_deref().and_then(parse_ts).unwrap_or(0);
            sink(UsageRecord {
                model,
                message_id,
                session_id,
                ts,
                cwd: parsed.cwd,
                input_tokens: usage.input_tokens,
                output_tokens: usage.output_tokens,
                cache_read_tokens: usage.cache_read_input_tokens,
                cache_write_tokens: usage.cache_creation_input_tokens,
            });
        }
    }
    Ok(())
}

fn parse_ts(s: &str) -> Option<i64> {
    chrono::DateTime::parse_from_rfc3339(s)
        .ok()
        .map(|dt| dt.timestamp())
}
