mod cli;
mod db;
mod display;
mod models;
mod monitor;

use anyhow::{Context, Result};
use clap::Parser;
use cli::{Cli, Commands, ReportAction};
use db::Database;
use models::{Task, TaskContext, TaskStatus};
use std::collections::HashMap;
use std::thread;
use std::time::Duration;

fn main() -> Result<()> {
    let cli = Cli::parse();

    // Ensure data directory exists
    db::ensure_data_dir()?;

    // Open database
    let db_path = db::default_db_path();
    let db = Database::open(&db_path).context("Failed to open database")?;

    // Run cleanup on every invocation
    let _ = db.cleanup_old_completed(3600); // 1 hour default

    match cli.command {
        None => {
            // Default: show tasks needing attention
            let tasks = db.list_tasks(Some(TaskStatus::NeedsAttention))?;
            display::display_task_list(&tasks);
        }
        Some(Commands::List { all, status }) => {
            let tasks = if let Some(status_str) = status {
                let status = TaskStatus::from_str(&status_str)
                    .map_err(|e| anyhow::anyhow!(e))?;
                db.list_tasks(Some(status))?
            } else if all {
                db.list_tasks(None)?
            } else {
                db.list_tasks(Some(TaskStatus::NeedsAttention))?
            };

            display::display_task_list(&tasks);
        }
        Some(Commands::Show { task_id }) => {
            let task = db
                .get_task_by_id(&task_id)?
                .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

            display::display_task_detail(&task);
        }
        Some(Commands::Clear { task_id }) => {
            let deleted = db.delete_task(&task_id)?;
            if deleted {
                println!("Task {} cleared", task_id);
            } else {
                println!("Task not found: {}", task_id);
            }
        }
        Some(Commands::ClearAll) => {
            let completed = db.list_tasks(Some(TaskStatus::Completed))?;
            let failed = db.list_tasks(Some(TaskStatus::Failed))?;

            let mut count = 0;
            for task in completed.iter().chain(failed.iter()) {
                db.delete_task(&task.task_id)?;
                count += 1;
            }

            println!("Cleared {} tasks", count);
        }
        Some(Commands::Reset { force }) => {
            let all_tasks = db.list_tasks(None)?;
            let task_count = all_tasks.len();

            if task_count == 0 {
                println!("No tasks to clear.");
                return Ok(());
            }

            // Show what will be cleared
            println!("This will delete ALL {} tasks:", task_count);
            for task in &all_tasks {
                println!("  - [{}] {}", task.agent_type, task.title);
            }
            println!();

            // Confirm unless --force
            if !force {
                use std::io::{self, Write};
                print!("Are you sure you want to delete ALL tasks? (yes/no): ");
                io::stdout().flush()?;

                let mut input = String::new();
                io::stdin().read_line(&mut input)?;

                if input.trim().to_lowercase() != "yes" {
                    println!("Aborted. No tasks were deleted.");
                    return Ok(());
                }
            }

            // Delete all tasks
            let mut count = 0;
            for task in all_tasks {
                db.delete_task(&task.task_id)?;
                count += 1;
            }

            println!("✓ Cleared all {} tasks", count);
        }
        Some(Commands::Watch) => {
            println!("Watching tasks (Ctrl+C to exit)...\n");

            loop {
                // Clear screen
                print!("\x1B[2J\x1B[1;1H");

                let tasks = db.list_tasks(None)?;
                display::display_task_list(&tasks);

                thread::sleep(Duration::from_secs(2));
            }
        }
        Some(Commands::Cleanup { retention_secs }) => {
            let deleted = db.cleanup_old_completed(retention_secs)?;
            println!("Cleaned up {} old completed tasks", deleted);
        }
        Some(Commands::Report { action }) => match action {
            ReportAction::Start {
                task_id,
                agent_type,
                cwd,
                title,
                pid,
                ppid,
            } => {
                let mut task = Task::new(task_id, agent_type, title, pid, ppid);

                // Add context
                task.context = Some(TaskContext {
                    url: None,
                    project_path: Some(cwd),
                    session_id: None,
                    extra: HashMap::new(),
                });

                db.insert_task(&task)?;
                println!("Task started: {}", task.task_id);
            }
            ReportAction::Complete { task_id, exit_code } => {
                let mut task = db
                    .get_task_by_id(&task_id)?
                    .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

                task.complete(exit_code);
                db.update_task(&task)?;
                println!("Task completed: {}", task_id);
            }
            ReportAction::NeedsAttention { task_id, reason } => {
                let mut task = db
                    .get_task_by_id(&task_id)?
                    .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

                task.needs_attention(reason);
                db.update_task(&task)?;
                println!("Task needs attention: {}", task_id);
            }
            ReportAction::Failed { task_id, exit_code } => {
                let mut task = db
                    .get_task_by_id(&task_id)?
                    .ok_or_else(|| anyhow::anyhow!("Task not found: {}", task_id))?;

                task.complete(Some(exit_code));
                db.update_task(&task)?;
                println!("Task failed: {}", task_id);
            }
        },
        Some(Commands::Monitor { task_id, pid }) => {
            // Create a monitor and start monitoring
            let monitor = monitor::TaskMonitor::new(db);
            monitor.monitor_task(task_id, pid)?;
        }
    }

    Ok(())
}
