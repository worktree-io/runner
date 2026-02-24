use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};

/// A time-to-live duration controlling how long a workspace remains active.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct Ttl(#[serde(with = "humantime_serde")] Duration);

impl Ttl {
    /// Create a new [`Ttl`] wrapping the given duration.
    #[must_use]
    pub const fn new(duration: Duration) -> Self {
        Self(duration)
    }

    /// Return the inner [`Duration`].
    #[must_use]
    pub const fn duration(self) -> Duration {
        self.0
    }
}

impl std::fmt::Display for Ttl {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{}", humantime::format_duration(self.0))
    }
}

impl std::str::FromStr for Ttl {
    type Err = humantime::DurationError;

    fn from_str(s: &str) -> std::result::Result<Self, Self::Err> {
        humantime::parse_duration(s).map(Self)
    }
}

/// A workspace entry stored in the registry.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkspaceRecord {
    /// Absolute path to the worktree directory.
    pub path: PathBuf,
    /// When this workspace was first created.
    #[serde(with = "humantime_serde")]
    pub created_at: SystemTime,
}

/// Persistent registry of all known workspaces and their creation timestamps.
#[derive(Debug, Clone, Serialize, Deserialize, Default)]
#[serde(default)]
pub struct WorkspaceRegistry {
    /// Registered workspaces.
    #[serde(rename = "workspace")]
    pub workspace: Vec<WorkspaceRecord>,
}

/// Returns `true` if the workspace has exceeded its TTL at the given instant.
///
/// Returns `false` when `created_at` is in the future relative to `now`.
#[must_use]
pub fn is_expired(record: &WorkspaceRecord, ttl: &Ttl, now: SystemTime) -> bool {
    now.duration_since(record.created_at)
        .map(|age| age >= ttl.0)
        .unwrap_or(false)
}

/// Returns the subset of workspaces that are both still present on disk and
/// have exceeded the given TTL.
///
/// Entries whose [`WorkspaceRecord::path`] no longer exists on disk are
/// silently skipped, making the registry self-healing on the next prune call.
#[must_use]
pub fn prune<'a>(
    records: &'a [WorkspaceRecord],
    ttl: &Ttl,
    now: SystemTime,
) -> Vec<&'a WorkspaceRecord> {
    records
        .iter()
        .filter(|r| r.path.exists() && is_expired(r, ttl, now))
        .collect()
}

impl WorkspaceRegistry {
    /// Return the path to the workspace registry file
    /// (`~/.config/worktree/workspaces.toml`).
    ///
    /// # Errors
    ///
    /// Returns an error if the home directory cannot be determined.
    pub fn path() -> Result<PathBuf> {
        let home = dirs::home_dir().context("Could not determine home directory")?;
        Ok(home
            .join(".config")
            .join("worktree")
            .join("workspaces.toml"))
    }

    /// Load the registry from disk, returning an empty registry if the file
    /// does not yet exist.
    ///
    /// # Errors
    ///
    /// Returns an error if the file cannot be read or parsed.
    pub fn load() -> Result<Self> {
        let path = Self::path()?;
        if !path.exists() {
            return Ok(Self::default());
        }
        // LLVM_COV_EXCL_START
        let content = std::fs::read_to_string(&path)
            .with_context(|| format!("Failed to read registry from {}", path.display()))?;
        toml::from_str(&content)
            .with_context(|| format!("Failed to parse registry at {}", path.display()))
        // LLVM_COV_EXCL_STOP
    }

    /// Persist the registry to disk.
    ///
    /// # Errors
    ///
    /// Returns an error if the config directory cannot be created or the file
    /// cannot be written.
    pub fn save(&self) -> Result<()> {
        // LLVM_COV_EXCL_START
        let path = Self::path()?;
        if let Some(parent) = path.parent() {
            std::fs::create_dir_all(parent)
                .with_context(|| format!("Failed to create config dir {}", parent.display()))?;
        }
        let content = toml::to_string(self).context("Failed to serialize workspace registry")?;
        std::fs::write(&path, content)
            .with_context(|| format!("Failed to write registry to {}", path.display()))
        // LLVM_COV_EXCL_STOP
    }

    /// Register a workspace by path with the current timestamp.
    ///
    /// Idempotent: if `path` is already present, the existing entry is left
    /// unchanged.
    pub fn register(&mut self, path: PathBuf) {
        if !self.workspace.iter().any(|r| r.path == path) {
            self.workspace.push(WorkspaceRecord {
                path,
                created_at: SystemTime::now(),
            });
        }
    }
}

#[cfg(test)]
#[path = "ttl_tests.rs"]
mod ttl_tests;
