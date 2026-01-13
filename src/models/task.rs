use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;

#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub enum TaskStatus {
    Running,
    Completed,
    NeedsAttention,
    Failed,
}

impl TaskStatus {
    pub fn as_str(&self) -> &str {
        match self {
            TaskStatus::Running => "running",
            TaskStatus::Completed => "completed",
            TaskStatus::NeedsAttention => "needs_attention",
            TaskStatus::Failed => "failed",
        }
    }

    pub fn from_str(s: &str) -> Result<Self, String> {
        match s {
            "running" => Ok(TaskStatus::Running),
            "completed" => Ok(TaskStatus::Completed),
            "needs_attention" => Ok(TaskStatus::NeedsAttention),
            "failed" => Ok(TaskStatus::Failed),
            _ => Err(format!("Invalid task status: {}", s)),
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskContext {
    pub url: Option<String>,
    pub project_path: Option<String>,
    pub session_id: Option<String>,
    #[serde(flatten)]
    pub extra: HashMap<String, serde_json::Value>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Task {
    pub id: Option<i64>,
    pub task_id: String,
    pub agent_type: String,
    pub title: String,
    pub status: TaskStatus,
    pub created_at: DateTime<Utc>,
    pub updated_at: DateTime<Utc>,
    pub completed_at: Option<DateTime<Utc>>,
    pub pid: Option<i32>,
    pub ppid: Option<i32>,
    pub monitor_pid: Option<i32>,
    pub attention_reason: Option<String>,
    pub exit_code: Option<i32>,
    pub context: Option<TaskContext>,
    pub metadata: Option<HashMap<String, serde_json::Value>>,
}

impl Task {
    pub fn new(
        task_id: String,
        agent_type: String,
        title: String,
        pid: Option<i32>,
        ppid: Option<i32>,
    ) -> Self {
        let now = Utc::now();
        Self {
            id: None,
            task_id,
            agent_type,
            title: Self::truncate_title(&title, 100),
            status: TaskStatus::Running,
            created_at: now,
            updated_at: now,
            completed_at: None,
            pid,
            ppid,
            monitor_pid: None,
            attention_reason: None,
            exit_code: None,
            context: None,
            metadata: None,
        }
    }

    fn truncate_title(title: &str, max_len: usize) -> String {
        if title.len() <= max_len {
            title.to_string()
        } else {
            format!("{}...", &title[..max_len.saturating_sub(3)])
        }
    }

    pub fn complete(&mut self, exit_code: Option<i32>) {
        self.status = if exit_code.map(|c| c != 0).unwrap_or(false) {
            TaskStatus::Failed
        } else {
            TaskStatus::Completed
        };
        self.exit_code = exit_code;
        self.completed_at = Some(Utc::now());
        self.updated_at = Utc::now();
    }

    pub fn needs_attention(&mut self, reason: String) {
        self.status = TaskStatus::NeedsAttention;
        self.attention_reason = Some(reason);
        self.updated_at = Utc::now();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_task_creation() {
        let task = Task::new(
            "test-id".to_string(),
            "claude_code".to_string(),
            "Test task".to_string(),
            Some(1234),
            Some(1233),
        );

        assert_eq!(task.task_id, "test-id");
        assert_eq!(task.agent_type, "claude_code");
        assert_eq!(task.status, TaskStatus::Running);
        assert_eq!(task.pid, Some(1234));
    }

    #[test]
    fn test_title_truncation() {
        let long_title = "a".repeat(150);
        let task = Task::new(
            "test-id".to_string(),
            "claude_code".to_string(),
            long_title,
            None,
            None,
        );

        assert_eq!(task.title.len(), 100);
        assert!(task.title.ends_with("..."));
    }

    #[test]
    fn test_task_completion_success() {
        let mut task = Task::new(
            "test-id".to_string(),
            "claude_code".to_string(),
            "Test task".to_string(),
            None,
            None,
        );

        task.complete(Some(0));
        assert_eq!(task.status, TaskStatus::Completed);
        assert_eq!(task.exit_code, Some(0));
        assert!(task.completed_at.is_some());
    }

    #[test]
    fn test_task_completion_failure() {
        let mut task = Task::new(
            "test-id".to_string(),
            "claude_code".to_string(),
            "Test task".to_string(),
            None,
            None,
        );

        task.complete(Some(1));
        assert_eq!(task.status, TaskStatus::Failed);
        assert_eq!(task.exit_code, Some(1));
    }

    #[test]
    fn test_needs_attention() {
        let mut task = Task::new(
            "test-id".to_string(),
            "claude_code".to_string(),
            "Test task".to_string(),
            None,
            None,
        );

        task.needs_attention("Waiting for input".to_string());
        assert_eq!(task.status, TaskStatus::NeedsAttention);
        assert_eq!(task.attention_reason, Some("Waiting for input".to_string()));
    }

    #[test]
    fn test_status_serialization() {
        assert_eq!(TaskStatus::Running.as_str(), "running");
        assert_eq!(TaskStatus::Completed.as_str(), "completed");
        assert_eq!(TaskStatus::NeedsAttention.as_str(), "needs_attention");
        assert_eq!(TaskStatus::Failed.as_str(), "failed");
    }

    #[test]
    fn test_status_deserialization() {
        assert_eq!(TaskStatus::from_str("running").unwrap(), TaskStatus::Running);
        assert_eq!(
            TaskStatus::from_str("completed").unwrap(),
            TaskStatus::Completed
        );
        assert_eq!(
            TaskStatus::from_str("needs_attention").unwrap(),
            TaskStatus::NeedsAttention
        );
        assert_eq!(TaskStatus::from_str("failed").unwrap(), TaskStatus::Failed);
        assert!(TaskStatus::from_str("invalid").is_err());
    }
}
