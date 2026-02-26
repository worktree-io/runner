//! `worktree-io` — open GitHub, Linear, and local Centy issues as git worktree workspaces.

/// Configuration loading and serialization.
pub mod config;
/// Git operations: cloning, fetching, branch detection, and worktree creation.
pub mod git;
/// Pre/post-open hook execution.
pub mod hooks;
/// Issue reference types and parsing.
pub mod issue;
/// Editor and terminal openers.
pub mod opener;
/// Per-repository hook configuration loaded from `.worktree.toml`.
pub mod repo_hooks;
/// `worktree://` URL scheme registration.
pub mod scheme;
/// Workspace TTL management and registry.
pub mod ttl;
/// Workspace creation and lifecycle.
pub mod workspace;

pub use config::Config;
pub use issue::{DeepLinkOptions, IssueRef};
pub use workspace::Workspace;
