//! Sandbox module for isolated agent execution.
//!
//! Provides containerized execution environments for Claude Code agents
//! using rootless Podman. Supports input injection via named pipes for
//! remote control via Telegram.

use crate::models::{SandboxConfig, SandboxType};
use anyhow::Result;
use std::path::{Path, PathBuf};

pub mod podman;

/// Information about a running container
#[derive(Debug, Clone)]
#[allow(dead_code)]
pub struct ContainerInfo {
    pub id: String,
    pub name: String,
    pub status: ContainerStatus,
    pub image: String,
    pub created_at: chrono::DateTime<chrono::Utc>,
    pub ports: Vec<String>,
}

/// Container runtime status
#[derive(Debug, Clone, PartialEq)]
pub enum ContainerStatus {
    Running,
    Stopped,
    Paused,
    Unknown,
}

/// Sandbox health — a richer view on top of ContainerStatus.
///
/// `ContainerStatus::Running` only tells us the container process is alive.
/// `SandboxHealth` goes one step further and verifies that `podman exec` can
/// actually run a process inside the container (catches zombie/unresponsive
/// containers that appear running but can no longer execute commands).
#[derive(Debug, Clone, PartialEq)]
pub enum SandboxHealth {
    /// Container running and `podman exec` works — ready to attach or inject.
    Healthy,
    /// Container appears running but `podman exec` fails (zombie / OOM / etc.).
    /// The container should be killed and re-spawned.
    Degraded,
    /// Container exists but is stopped (e.g. after a host reboot).
    /// Can be restarted with `sandbox.start()`.
    Stopped,
    /// Container no longer exists in the runtime.
    Dead,
}

/// Trait for sandbox implementations
#[allow(dead_code)]
pub trait Sandbox: Send + Sync {
    /// Check if the sandbox runtime is available
    fn is_available(&self) -> Result<bool>;

    /// Install/setup the sandbox runtime if needed
    fn setup(&self) -> Result<()>;

    /// Spawn a new container for an agent
    ///
    /// # Arguments
    /// * `task_id` - Unique task identifier
    /// * `repo_path` - Path to the repository on host
    /// * `config` - Sandbox configuration
    fn spawn(
        &self,
        task_id: &str,
        repo_path: &Path,
        config: &SandboxConfig,
    ) -> Result<ContainerInfo>;

    /// Start a stopped container (e.g. after a host reboot)
    fn start(&self, container_id: &str) -> Result<()>;

    /// Kill/stop a container
    fn kill(&self, container_id: &str) -> Result<()>;

    /// Get container status
    fn status(&self, container_id: &str) -> Result<ContainerStatus>;

    /// List all containers managed by this sandbox
    fn list(&self) -> Result<Vec<ContainerInfo>>;

    /// Get logs from a container
    fn logs(&self, container_id: &str, tail: Option<usize>) -> Result<String>;

    /// Execute a command inside the container
    fn exec(&self, container_id: &str, command: &[&str]) -> Result<String>;

    /// Check whether the container is running and can execute processes.
    ///
    /// Returns `SandboxHealth::Healthy` if the container is running and
    /// `podman exec` succeeds, `SandboxHealth::Degraded` if the container
    /// appears running but exec fails (zombie/OOM/etc.),
    /// `SandboxHealth::Stopped` if the container exists but is stopped (reboot),
    /// or `SandboxHealth::Dead` if the container no longer exists.
    fn health_check(&self, container_id: &str) -> SandboxHealth {
        match self.status(container_id) {
            Ok(ContainerStatus::Running) => {}
            Ok(ContainerStatus::Stopped) => return SandboxHealth::Stopped,
            _ => return SandboxHealth::Dead,
        }

        // Try running a trivial command to confirm exec capability.
        let can_exec = self.exec(container_id, &["true"]).is_ok();

        if can_exec {
            SandboxHealth::Healthy
        } else {
            SandboxHealth::Degraded
        }
    }
}

/// Factory function to get the appropriate sandbox implementation
#[allow(dead_code)]
pub fn get_sandbox(sandbox_type: SandboxType) -> Result<Box<dyn Sandbox>> {
    match sandbox_type {
        SandboxType::None => Err(anyhow::anyhow!("No sandbox implementation for type 'none'")),
        SandboxType::Podman => Ok(Box::new(podman::PodmanSandbox::new())),
    }
}

/// Check if podman is installed and available
#[allow(dead_code)]
pub fn is_podman_available() -> bool {
    std::process::Command::new("podman")
        .arg("--version")
        .output()
        .map(|o| o.status.success())
        .unwrap_or(false)
}

/// Get the base directory for nibble data
pub fn get_data_dir() -> Result<PathBuf> {
    let home = dirs::home_dir().ok_or_else(|| anyhow::anyhow!("Could not find home directory"))?;
    let data_dir = home.join(".nibble");
    std::fs::create_dir_all(&data_dir)?;
    Ok(data_dir)
}

/// Get the cache directory for sandbox dependencies
pub fn get_cache_dir() -> Result<PathBuf> {
    let cache_dir = get_data_dir()?.join("cache");
    std::fs::create_dir_all(&cache_dir)?;
    Ok(cache_dir)
}

/// Compute the container working directory from a host repo path.
///
/// The repository is mounted at `/<basename>` inside the container and
/// that path becomes the working directory. This gives each repo its own
/// session namespace naturally.
///
/// Examples:
/// - `/home/user/nibble` → `/nibble`
/// - `/home/user/nibble--feature-x` → `/nibble--feature-x`
pub fn container_working_dir(repo_path: &std::path::Path) -> String {
    let basename = repo_path
        .file_name()
        .and_then(|n| n.to_str())
        .unwrap_or("workspace");
    format!("/{}", basename)
}

/// Compute the Pi session directory name for a container working directory.
///
/// Pi encodes the cwd as a directory slug by replacing `/` with `--` and
/// wrapping in `--..--`. For a repo mounted at `/nibble` this yields
/// `--nibble--`.
pub fn pi_session_dir_name(container_dir: &str) -> String {
    let inner = container_dir.trim_start_matches('/');
    if inner.is_empty() {
        "--workspace--".to_string()
    } else {
        format!("--{}--", inner.replace('/', "--"))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_sandbox_type_serialization() {
        assert_eq!(SandboxType::None.as_str(), "none");
        assert_eq!(SandboxType::Podman.as_str(), "podman");
    }

    #[test]
    fn test_sandbox_type_deserialization() {
        use std::str::FromStr;
        assert_eq!(SandboxType::from_str("none").unwrap(), SandboxType::None);
        assert_eq!(
            SandboxType::from_str("podman").unwrap(),
            SandboxType::Podman
        );
        assert!(SandboxType::from_str("invalid").is_err());
    }

    #[test]
    fn test_sandbox_config_default() {
        let config = SandboxConfig::default();
        assert_eq!(config.image, "nibble-sandbox:latest");
        assert!(config.privileged);
        assert_eq!(config.port_ranges.len(), 2);
    }
}
