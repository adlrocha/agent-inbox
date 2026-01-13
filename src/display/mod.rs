use crate::models::{Task, TaskStatus};
use chrono::Utc;

// ANSI color codes
const RESET: &str = "\x1b[0m";
const BOLD: &str = "\x1b[1m";
const DIM: &str = "\x1b[2m";

// Colors
const RED: &str = "\x1b[31m";
const GREEN: &str = "\x1b[32m";
const YELLOW: &str = "\x1b[33m";
const BLUE: &str = "\x1b[34m";
const MAGENTA: &str = "\x1b[35m";
const CYAN: &str = "\x1b[36m";
const WHITE: &str = "\x1b[37m";
const GRAY: &str = "\x1b[90m";

// Bright colors
const BRIGHT_RED: &str = "\x1b[91m";
const BRIGHT_GREEN: &str = "\x1b[92m";
const BRIGHT_YELLOW: &str = "\x1b[93m";
const BRIGHT_BLUE: &str = "\x1b[94m";
const BRIGHT_CYAN: &str = "\x1b[96m";

// Icons (using Unicode)
const ICON_ATTENTION: &str = "⚠️ ";
const ICON_RUNNING: &str = "▶️ ";
const ICON_COMPLETED: &str = "✓";
const ICON_FAILED: &str = "✗";
const ICON_ARROW: &str = "→";

pub fn display_task_list(tasks: &[Task]) {
    let mut needs_attention = Vec::new();
    let mut running = Vec::new();
    let mut completed = Vec::new();
    let mut failed = Vec::new();

    for task in tasks {
        match task.status {
            TaskStatus::NeedsAttention => needs_attention.push(task),
            TaskStatus::Running => running.push(task),
            TaskStatus::Completed => completed.push(task),
            TaskStatus::Failed => failed.push(task),
        }
    }

    let total_active = needs_attention.len() + running.len();

    if total_active == 0 && completed.is_empty() && failed.is_empty() {
        println!("{}{}No active tasks{}", DIM, GRAY, RESET);
        println!("{}Start a conversation in Claude.ai or Gemini to create tasks{}", DIM, RESET);
        return;
    }

    // Header with box drawing
    println!();
    println!("{}{}╭─────────────────────────────────────────────╮{}", BOLD, CYAN, RESET);
    println!("{}{}│  {}Agent Inbox{}                              │{}", BOLD, CYAN, WHITE, CYAN, RESET);
    println!("{}{}╰─────────────────────────────────────────────╯{}", BOLD, CYAN, RESET);
    println!();

    // Summary line with colors
    let mut summary_parts = Vec::new();

    if needs_attention.len() > 0 {
        summary_parts.push(format!("{}{}{} need attention{}", BOLD, BRIGHT_YELLOW, needs_attention.len(), RESET));
    }
    if running.len() > 0 {
        summary_parts.push(format!("{}{}{} running{}", BOLD, BRIGHT_BLUE, running.len(), RESET));
    }
    if completed.len() > 0 {
        summary_parts.push(format!("{}{} completed{}", GREEN, completed.len(), RESET));
    }
    if failed.len() > 0 {
        summary_parts.push(format!("{}{} failed{}", BRIGHT_RED, failed.len(), RESET));
    }

    if !summary_parts.is_empty() {
        println!("{}", summary_parts.join(&format!("{}  •  {}", GRAY, RESET)));
        println!();
    }

    // Needs Attention section (most important)
    if !needs_attention.is_empty() {
        println!("{}{}{} NEEDS ATTENTION{}", BOLD, BRIGHT_YELLOW, ICON_ATTENTION, RESET);
        println!("{}{}{}", GRAY, "─".repeat(50), RESET);
        for (idx, task) in needs_attention.iter().enumerate() {
            print_task_summary(idx + 1, task);
        }
        println!();
    }

    // Running section
    if !running.is_empty() {
        println!("{}{}{} RUNNING{}", BOLD, BRIGHT_BLUE, ICON_RUNNING, RESET);
        println!("{}{}{}", GRAY, "─".repeat(50), RESET);
        let start_idx = needs_attention.len();
        for (idx, task) in running.iter().enumerate() {
            print_task_summary(start_idx + idx + 1, task);
        }
        println!();
    }

    // Completed section
    if !completed.is_empty() {
        println!("{}{} {} COMPLETED{}", BOLD, GREEN, ICON_COMPLETED, RESET);
        println!("{}{}{}", GRAY, "─".repeat(50), RESET);
        let start_idx = needs_attention.len() + running.len();
        for (idx, task) in completed.iter().enumerate() {
            print_task_summary(start_idx + idx + 1, task);
        }
        println!();
    }

    // Failed section
    if !failed.is_empty() {
        println!("{}{} {} FAILED{}", BOLD, BRIGHT_RED, ICON_FAILED, RESET);
        println!("{}{}{}", GRAY, "─".repeat(50), RESET);
        let start_idx = needs_attention.len() + running.len() + completed.len();
        for (idx, task) in failed.iter().enumerate() {
            print_task_summary(start_idx + idx + 1, task);
        }
        println!();
    }

    // Footer with helpful info
    if !completed.is_empty() {
        println!("{}{} Completed tasks auto-clear after 1 hour{}", DIM, GRAY, RESET);
    }
    println!("{}{} Run {}agent-inbox show <id>{} for details{}", DIM, GRAY, CYAN, GRAY, RESET);
    println!();
}

fn print_task_summary(idx: usize, task: &Task) {
    // Agent badge with color
    let agent_label = if let Some(pid) = task.pid {
        format!("{}:{}", task.agent_type, pid)
    } else {
        task.agent_type.clone()
    };

    let (agent_color, badge): (&str, String) = match task.agent_type.as_str() {
        "claude_web" => (MAGENTA, "claude.ai".to_string()),
        "gemini_web" => (BLUE, "gemini".to_string()),
        "claude_code" => (CYAN, "claude-code".to_string()),
        "opencode" => (GREEN, "opencode".to_string()),
        _ => (WHITE, agent_label.clone()),
    };

    let elapsed = format_elapsed(task.updated_at.timestamp());

    // Status indicator
    let status_indicator = match task.status {
        TaskStatus::NeedsAttention => format!("{}{}", BRIGHT_YELLOW, "●"),
        TaskStatus::Running => format!("{}{}", BRIGHT_BLUE, "●"),
        TaskStatus::Completed => format!("{}{}", GREEN, "●"),
        TaskStatus::Failed => format!("{}{}", BRIGHT_RED, "●"),
    };

    // Print task line with colors
    print!("  {}{}{:2}.{} ", GRAY, BOLD, idx, RESET);
    print!("{}{} ", status_indicator, RESET);
    print!("{}{}[{}]{} ", BOLD, agent_color, badge, RESET);
    print!("{}\"{}\"{} ", WHITE, truncate(&task.title, 60), RESET);
    println!("{}{}{}", DIM, elapsed, RESET);

    // Additional info indented
    if task.status == TaskStatus::NeedsAttention {
        if let Some(reason) = &task.attention_reason {
            println!("      {}{} {}{}", YELLOW, ICON_ARROW, reason, RESET);
        }
    }

    if task.status == TaskStatus::Failed {
        if let Some(code) = task.exit_code {
            println!("      {}{} Exit code: {}{}", RED, ICON_ARROW, code, RESET);
        }
    }
}

pub fn display_task_detail(task: &Task) {
    println!();
    println!("{}{}╭─────────────────────────────────────────────╮{}", BOLD, CYAN, RESET);
    println!("{}{}│  {}Task Details{}                            │{}", BOLD, CYAN, WHITE, CYAN, RESET);
    println!("{}{}╰─────────────────────────────────────────────╯{}", BOLD, CYAN, RESET);
    println!();

    // Status badge
    let (status_color, status_text) = match task.status {
        TaskStatus::Running => (BRIGHT_BLUE, "RUNNING"),
        TaskStatus::Completed => (GREEN, "COMPLETED"),
        TaskStatus::NeedsAttention => (BRIGHT_YELLOW, "NEEDS ATTENTION"),
        TaskStatus::Failed => (BRIGHT_RED, "FAILED"),
    };

    println!("{}{}Status:{} {}{}{}{}", BOLD, GRAY, RESET, BOLD, status_color, status_text, RESET);
    println!();

    println!("{}{}ID:{} {}{}{}", BOLD, GRAY, RESET, CYAN, task.task_id, RESET);
    println!("{}{}Agent:{} {}{}{}", BOLD, GRAY, RESET, MAGENTA, task.agent_type, RESET);
    println!("{}{}Title:{} {}{}{}", BOLD, GRAY, RESET, WHITE, task.title, RESET);
    println!();

    println!("{}{}Timestamps:{}", BOLD, GRAY, RESET);
    println!("  {}Created:  {}{}{}", GRAY, RESET, format_datetime(&task.created_at), RESET);
    println!("  {}Updated:  {}{}{}", GRAY, RESET, format_datetime(&task.updated_at), RESET);
    if let Some(completed) = task.completed_at {
        println!("  {}Completed: {}{}{}", GRAY, GREEN, format_datetime(&completed), RESET);
    }
    println!();

    if task.pid.is_some() || task.ppid.is_some() {
        println!("{}{}Process Info:{}", BOLD, GRAY, RESET);
        if let Some(pid) = task.pid {
            println!("  {}PID:     {}{}{}", GRAY, RESET, pid, RESET);
        }
        if let Some(ppid) = task.ppid {
            println!("  {}Parent:  {}{}{}", GRAY, RESET, ppid, RESET);
        }
        if let Some(monitor_pid) = task.monitor_pid {
            println!("  {}Monitor: {}{}{}", GRAY, RESET, monitor_pid, RESET);
        }
        println!();
    }

    if let Some(reason) = &task.attention_reason {
        println!("{}{} Attention Reason:{} {}{}{}", BOLD, YELLOW, RESET, YELLOW, reason, RESET);
        println!();
    }

    if let Some(code) = task.exit_code {
        println!("{}{} Exit Code:{} {}{}{}", BOLD, RED, RESET, RED, code, RESET);
        println!();
    }

    if let Some(context) = &task.context {
        println!("{}{}Context:{}", BOLD, GRAY, RESET);
        if let Some(url) = &context.url {
            println!("  {}URL:        {}{}{}", GRAY, BRIGHT_CYAN, url, RESET);
        }
        if let Some(path) = &context.project_path {
            println!("  {}Project:    {}{}{}", GRAY, CYAN, path, RESET);
        }
        if let Some(session) = &context.session_id {
            println!("  {}Session ID: {}{}{}", GRAY, RESET, session, RESET);
        }
        if !context.extra.is_empty() {
            println!("  {}Extra:{}", GRAY, RESET);
            for (key, value) in &context.extra {
                println!("    {}{}: {}{}", GRAY, key, RESET, value);
            }
        }
        println!();
    }
}

fn format_datetime(dt: &chrono::DateTime<Utc>) -> String {
    dt.format("%Y-%m-%d %H:%M:%S UTC").to_string()
}

fn format_elapsed(timestamp: i64) -> String {
    let now = Utc::now().timestamp();
    let elapsed = now - timestamp;

    if elapsed < 60 {
        format!("({}s ago)", elapsed)
    } else if elapsed < 3600 {
        format!("({}m ago)", elapsed / 60)
    } else if elapsed < 86400 {
        format!("({}h ago)", elapsed / 3600)
    } else {
        format!("({}d ago)", elapsed / 86400)
    }
}

fn truncate(s: &str, max_len: usize) -> String {
    if s.len() <= max_len {
        s.to_string()
    } else {
        format!("{}...", &s[..max_len - 3])
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_format_elapsed() {
        let now = Utc::now().timestamp();

        assert_eq!(format_elapsed(now - 30), "30s ago");
        assert_eq!(format_elapsed(now - 120), "2m ago");
        assert_eq!(format_elapsed(now - 3660), "1h ago");
        assert_eq!(format_elapsed(now - 90000), "1d ago");
    }
}
