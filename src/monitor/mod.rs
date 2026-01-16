// Detectors module kept for potential future use
#[allow(dead_code)]
pub mod detectors;

use crate::db::Database;
use crate::models::TaskStatus;
use anyhow::Result;
use std::thread;
use std::time::Duration;

/// Simple process monitor for CLI tools
///
/// With the 3-state model (Running, Completed, Exited):
/// - CLI tools start as "Running"
/// - When process exits → "Exited"
///
/// Note: We don't try to detect "Completed" (waiting for input) for CLI tools
/// because it's unreliable. The wrapper script handles reporting completion
/// with exit codes.
pub struct TaskMonitor {
    db: Database,
    poll_interval: Duration,
}

impl TaskMonitor {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            poll_interval: Duration::from_secs(5),
        }
    }

    /// Monitor a process and update task status when it exits
    pub fn monitor_task(&self, task_id: String, pid: i32) -> Result<()> {
        loop {
            // Check if process is still alive
            if !is_process_alive(pid) {
                // Process died, mark as exited
                if let Some(mut task) = self.db.get_task_by_id(&task_id)? {
                    // Monitor doesn't know exit code, wrapper will update with correct code
                    task.set_exited(None);
                    self.db.update_task(&task)?;
                }
                break;
            }

            // Get current task state
            let task = match self.db.get_task_by_id(&task_id)? {
                Some(t) => t,
                None => {
                    // Task was deleted, stop monitoring
                    break;
                }
            };

            // Stop monitoring if task is already completed or exited
            if task.status == TaskStatus::Completed || task.status == TaskStatus::Exited {
                break;
            }

            // Sleep before next check
            thread::sleep(self.poll_interval);
        }

        Ok(())
    }
}

fn is_process_alive(pid: i32) -> bool {
    // Check if /proc/<pid> exists
    std::path::Path::new(&format!("/proc/{}", pid)).exists()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_is_process_alive() {
        // Current process should be alive
        let current_pid = std::process::id() as i32;
        assert!(is_process_alive(current_pid));

        // PID 999999 very unlikely to exist
        assert!(!is_process_alive(999999));
    }
}
