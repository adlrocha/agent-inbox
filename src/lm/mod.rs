use crate::config::LmConfig;
use anyhow::Result;
use std::path::{Path, PathBuf};

pub struct ModelEntry {
    pub path: PathBuf,
    pub size_bytes: u64,
    pub is_active: bool,
    /// True when the filename contains an MTP marker (-MTP- or -A3B-).
    pub is_mtp: bool,
}

/// Return true if the filename pattern suggests an MTP model.
fn looks_like_mtp(name: &str) -> bool {
    let n = name.to_uppercase();
    n.contains("-MTP-") || n.contains("-A3B-")
}

/// Parse the active model path out of a systemd unit ExecStart line.
/// Looks for `-m <path>` anywhere in the file.
fn read_active_model(unit_path: &str) -> Option<PathBuf> {
    let contents = std::fs::read_to_string(unit_path).ok()?;
    // Walk tokens; the token after `-m` is the model path.
    let tokens: Vec<&str> = contents.split_whitespace().collect();
    for i in 0..tokens.len().saturating_sub(1) {
        if tokens[i] == "-m" {
            return Some(PathBuf::from(tokens[i + 1]));
        }
    }
    None
}

/// Scan `dir` for .gguf files and return their paths sorted by name.
fn scan_dir(dir: &Path) -> Vec<PathBuf> {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return Vec::new();
    };
    let mut paths: Vec<PathBuf> = entries
        .flatten()
        .filter_map(|e| {
            let p = e.path();
            if p.extension().and_then(|x| x.to_str()) == Some("gguf") {
                Some(p)
            } else {
                None
            }
        })
        .collect();
    paths.sort();
    paths
}

pub fn list_models(cfg: &LmConfig) -> Result<Vec<ModelEntry>> {
    let active = read_active_model(&cfg.service_unit);

    let mut entries = Vec::new();
    for dir_str in &cfg.model_dirs {
        let dir = expand_home(dir_str);
        for path in scan_dir(&dir) {
            let size_bytes = path.metadata().map(|m| m.len()).unwrap_or(0);
            let name = path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .to_string();
            let is_mtp = looks_like_mtp(&name);
            let is_active = active
                .as_ref()
                .map(|a| a.canonicalize().ok() == path.canonicalize().ok() || *a == path)
                .unwrap_or(false);
            entries.push(ModelEntry {
                path,
                size_bytes,
                is_active,
                is_mtp,
            });
        }
    }
    Ok(entries)
}

/// Print a formatted table of model entries to stdout.
pub fn print_list(entries: &[ModelEntry]) {
    if entries.is_empty() {
        eprintln!("No .gguf models found. Check [lm] model_dirs in ~/.nibble/config.toml");
        return;
    }

    // Column widths
    let name_width = entries
        .iter()
        .map(|e| {
            e.path
                .file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("")
                .len()
        })
        .max()
        .unwrap_or(10)
        .max(10);

    println!(
        "  {:<width$}  {:>8}  {}",
        "MODEL",
        "SIZE",
        "FLAGS",
        width = name_width
    );
    println!("  {}  --------  -----", "-".repeat(name_width));

    for e in entries {
        let name = e.path.file_name().and_then(|n| n.to_str()).unwrap_or("?");
        let size = format_size(e.size_bytes);
        let mut flags = Vec::new();
        if e.is_active {
            flags.push("active");
        }
        if e.is_mtp {
            flags.push("mtp");
        }
        let flag_str = if flags.is_empty() {
            String::new()
        } else {
            format!("[{}]", flags.join(", "))
        };

        let active_marker = if e.is_active { "▶" } else { " " };
        println!(
            "{} {:<width$}  {:>8}  {}",
            active_marker,
            name,
            size,
            flag_str,
            width = name_width
        );
    }
}

fn format_size(bytes: u64) -> String {
    const GIB: u64 = 1 << 30;
    const MIB: u64 = 1 << 20;
    if bytes >= GIB {
        format!("{:.1}G", bytes as f64 / GIB as f64)
    } else if bytes >= MIB {
        format!("{:.0}M", bytes as f64 / MIB as f64)
    } else {
        format!("{}B", bytes)
    }
}

fn expand_home(s: &str) -> PathBuf {
    if s.starts_with("~/") {
        let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
        PathBuf::from(home).join(&s[2..])
    } else {
        PathBuf::from(s)
    }
}
