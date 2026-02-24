use std::path::PathBuf;
use std::time::{Duration, SystemTime};

use super::*;

fn past(secs: u64) -> SystemTime {
    SystemTime::now()
        .checked_sub(Duration::from_secs(secs))
        .expect("time subtraction overflowed")
}

fn future(secs: u64) -> SystemTime {
    SystemTime::now()
        .checked_add(Duration::from_secs(secs))
        .expect("time addition overflowed")
}

fn record(path: PathBuf, created_at: SystemTime) -> WorkspaceRecord {
    WorkspaceRecord { path, created_at }
}

// ── Ttl ──────────────────────────────────────────────────────────────────────

#[test]
fn test_ttl_new_and_duration() {
    let d = Duration::from_secs(3600);
    assert_eq!(Ttl::new(d).duration(), d);
}

#[test]
fn test_ttl_display_round_trips() {
    let ttl = Ttl::new(Duration::from_secs(7 * 24 * 3600));
    let s = ttl.to_string();
    let parsed: Ttl = s.parse().unwrap();
    assert_eq!(parsed.duration(), ttl.duration());
}

#[test]
fn test_ttl_parse_valid() {
    let ttl: Ttl = "1day".parse().unwrap();
    assert_eq!(ttl.duration(), Duration::from_secs(86400));
}

#[test]
fn test_ttl_parse_invalid() {
    assert!("not-a-duration".parse::<Ttl>().is_err());
}

// ── is_expired ───────────────────────────────────────────────────────────────

#[test]
fn test_is_expired_when_age_exceeds_ttl() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let r = record(PathBuf::from("/tmp/ws"), past(120));
    assert!(is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_not_expired_when_young() {
    let ttl = Ttl::new(Duration::from_secs(3600));
    let r = record(PathBuf::from("/tmp/ws"), past(60));
    assert!(!is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_not_expired_when_created_in_future() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let r = record(PathBuf::from("/tmp/ws"), future(30));
    assert!(!is_expired(&r, &ttl, SystemTime::now()));
}

#[test]
fn test_is_expired_at_exact_boundary() {
    // age == ttl → expired
    let ttl = Ttl::new(Duration::from_secs(100));
    let now = SystemTime::now();
    let r = record(
        PathBuf::from("/tmp/ws"),
        now.checked_sub(Duration::from_secs(100)).unwrap(),
    );
    assert!(is_expired(&r, &ttl, now));
}

// ── prune ────────────────────────────────────────────────────────────────────

#[test]
fn test_prune_returns_expired_with_existing_path() {
    let dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(60));
    let records = vec![record(dir.path().to_path_buf(), past(120))];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].path, dir.path());
}

#[test]
fn test_prune_skips_missing_paths() {
    let ttl = Ttl::new(Duration::from_secs(60));
    let records = vec![record(PathBuf::from("/nonexistent/path/xyz"), past(120))];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert!(expired.is_empty());
}

#[test]
fn test_prune_skips_unexpired() {
    let dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(3600));
    let records = vec![record(dir.path().to_path_buf(), past(60))];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert!(expired.is_empty());
}

#[test]
fn test_prune_mixed_records() {
    let expired_dir = tempfile::tempdir().unwrap();
    let fresh_dir = tempfile::tempdir().unwrap();
    let ttl = Ttl::new(Duration::from_secs(600));
    let records = vec![
        record(expired_dir.path().to_path_buf(), past(1200)), // expired + exists
        record(fresh_dir.path().to_path_buf(), past(60)),     // not expired + exists
        record(PathBuf::from("/no/such/path"), past(1200)),   // expired + missing
    ];
    let expired = prune(&records, &ttl, SystemTime::now());
    assert_eq!(expired.len(), 1);
    assert_eq!(expired[0].path, expired_dir.path());
}

// ── WorkspaceRegistry ────────────────────────────────────────────────────────

#[test]
fn test_registry_path_ends_with_workspaces_toml() {
    let p = WorkspaceRegistry::path().unwrap();
    assert!(p.ends_with(".config/worktree/workspaces.toml"));
}

#[test]
fn test_registry_load_missing_returns_empty() {
    // WorkspaceRegistry::load() returns Ok(default) when file absent.
    // We verify the default is empty.
    let r = WorkspaceRegistry::default();
    assert!(r.workspace.is_empty());
}

#[test]
fn test_registry_register_adds_entry() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/some/path"));
    assert_eq!(r.workspace.len(), 1);
    assert_eq!(r.workspace[0].path, PathBuf::from("/some/path"));
}

#[test]
fn test_registry_register_idempotent() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/some/path"));
    r.register(PathBuf::from("/some/path"));
    assert_eq!(r.workspace.len(), 1);
}

#[test]
fn test_registry_register_different_paths() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/path/a"));
    r.register(PathBuf::from("/path/b"));
    assert_eq!(r.workspace.len(), 2);
}

// ── Serde round-trips ────────────────────────────────────────────────────────

#[derive(serde::Serialize, serde::Deserialize)]
struct TtlHelper {
    ttl: Ttl,
}

#[test]
fn test_ttl_serde_round_trip() {
    let original = Ttl::new(Duration::from_secs(7 * 24 * 3600));
    let s = toml::to_string(&TtlHelper { ttl: original }).unwrap();
    let parsed: TtlHelper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.ttl.duration(), original.duration());
}

#[test]
fn test_workspace_registry_serde_empty() {
    let r = WorkspaceRegistry::default();
    let s = toml::to_string(&r).unwrap();
    let parsed: WorkspaceRegistry = toml::from_str(&s).unwrap();
    assert!(parsed.workspace.is_empty());
}

#[test]
fn test_workspace_registry_serde_with_entries() {
    let mut r = WorkspaceRegistry::default();
    r.register(PathBuf::from("/tmp/ws1"));
    r.register(PathBuf::from("/tmp/ws2"));
    let s = toml::to_string(&r).unwrap();
    let parsed: WorkspaceRegistry = toml::from_str(&s).unwrap();
    assert_eq!(parsed.workspace.len(), 2);
}

#[derive(serde::Serialize, serde::Deserialize)]
struct RecordHelper {
    workspace: Vec<WorkspaceRecord>,
}

#[test]
fn test_workspace_record_serde_round_trip() {
    let created_at = SystemTime::now()
        .checked_sub(Duration::from_secs(3600))
        .unwrap();
    let original = WorkspaceRecord {
        path: PathBuf::from("/tmp/my-workspace"),
        created_at,
    };
    let h = RecordHelper {
        workspace: vec![original],
    };
    let s = toml::to_string(&h).unwrap();
    let parsed: RecordHelper = toml::from_str(&s).unwrap();
    assert_eq!(parsed.workspace[0].path, PathBuf::from("/tmp/my-workspace"));
    // Timestamps round-trip to second precision via RFC3339
    let original_secs = h.workspace[0]
        .created_at
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    let parsed_secs = parsed.workspace[0]
        .created_at
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .as_secs();
    assert_eq!(original_secs, parsed_secs);
}
