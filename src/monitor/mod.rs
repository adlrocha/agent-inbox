pub mod detectors;

use crate::db::Database;
use crate::models::TaskStatus;
use anyhow::Result;
use detectors::{create_default_detectors, AttentionDetector, TaskContext};
use std::thread;
use std::time::{Duration, SystemTime};

pub struct TaskMonitor {
    db: Database,
    detectors: Vec<Box<dyn AttentionDetector>>,
    poll_interval: Duration,
}

impl TaskMonitor {
    pub fn new(db: Database) -> Self {
        Self {
            db,
            detectors: create_default_detectors(),
            poll_interval: Duration::from_secs(5),
        }
    }

    pub fn monitor_task(&self, task_id: String, pid: i32) -> Result<()> {
        let mut context = TaskContext {
            pid,
            last_check: SystemTime::now(),
            last_cpu_time: get_process_cpu_time(pid),
            idle_duration: Duration::from_secs(0),
        };

        loop {
            // Check if process is still alive
            if !is_process_alive(pid) {
                // Process died, mark as completed
                if let Some(mut task) = self.db.get_task_by_id(&task_id)? {
                    // Try to get exit code from process
                    task.complete(None); // Monitor doesn't know exit code, wrapper will update
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

            // Don't check if already completed or needs attention
            if task.status == TaskStatus::Completed
                || task.status == TaskStatus::Failed
                || task.status == TaskStatus::NeedsAttention
            {
                break;
            }

            // Find child processes and check them too
            // The wrapper process (pid) spawns the actual agent as a child
            let pids_to_check = get_process_tree(pid);

            // Run detectors on all processes in the tree
            for check_pid in pids_to_check {
                let current_cpu = get_process_cpu_time(check_pid);

                // Calculate idle duration - if CPU hasn't changed, increment idle time
                let check_idle_duration = if let (Some(curr), Some(last)) = (current_cpu, context.last_cpu_time) {
                    if curr == last {
                        context.idle_duration + self.poll_interval
                    } else {
                        Duration::from_secs(0)
                    }
                } else {
                    context.idle_duration
                };

                let check_context = TaskContext {
                    pid: check_pid,
                    last_check: context.last_check,
                    last_cpu_time: current_cpu,
                    idle_duration: check_idle_duration,
                };

                for detector in &self.detectors {
                    if let Some(reason) = detector.check(&task, &check_context) {
                        // Found a reason for attention
                        let mut updated_task = task.clone();
                        updated_task.needs_attention(reason.as_str());
                        self.db.update_task(&updated_task)?;

                        // Stop monitoring once we've flagged it
                        return Ok(());
                    }
                }
            }

            // Update context for next iteration
            let current_cpu = get_process_cpu_time(pid);
            if let (Some(curr), Some(last)) = (current_cpu, context.last_cpu_time) {
                if curr == last {
                    context.idle_duration += self.poll_interval;
                } else {
                    context.idle_duration = Duration::from_secs(0);
                }
            }
            context.last_check = SystemTime::now();
            context.last_cpu_time = current_cpu;

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

fn get_process_cpu_time(pid: i32) -> Option<u64> {
    let stat_path = format!("/proc/{}/stat", pid);
    let stat_content = std::fs::read_to_string(&stat_path).ok()?;

    let parts: Vec<&str> = stat_content.split_whitespace().collect();
    if parts.len() < 15 {
        return None;
    }

    // Fields 13 and 14 are utime and stime (user and system CPU time)
    let utime: u64 = parts[13].parse().ok()?;
    let stime: u64 = parts[14].parse().ok()?;

    Some(utime + stime)
}

fn get_process_tree(pid: i32) -> Vec<i32> {
    // Returns the process and all its children (recursively)
    let mut result = vec![pid];

    // Read /proc to find all child processes
    if let Ok(entries) = std::fs::read_dir("/proc") {
        for entry in entries.flatten() {
            if let Ok(file_name) = entry.file_name().into_string() {
                if let Ok(child_pid) = file_name.parse::<i32>() {
                    // Read /proc/<pid>/stat to get parent PID
                    let stat_path = format!("/proc/{}/stat", child_pid);
                    if let Ok(stat_content) = std::fs::read_to_string(&stat_path) {
                        // Parse parent PID (4th field after the command name in parentheses)
                        if let Some(ppid) = parse_ppid_from_stat(&stat_content) {
                            if ppid == pid {
                                // This is a direct child, recurse to get its children too
                                result.extend(get_process_tree(child_pid));
                            }
                        }
                    }
                }
            }
        }
    }

    result
}

fn parse_ppid_from_stat(stat_content: &str) -> Option<i32> {
    // Format: pid (comm) state ppid ...
    // Need to handle command names with spaces/parentheses
    let close_paren = stat_content.rfind(')')?;
    let after_comm = &stat_content[close_paren + 1..];
    let parts: Vec<&str> = after_comm.split_whitespace().collect();

    // First part is state, second is ppid
    if parts.len() >= 2 {
        parts[1].parse().ok()
    } else {
        None
    }
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
