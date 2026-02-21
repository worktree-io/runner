use anyhow::Result;

#[cfg(target_os = "macos")]
mod macos;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "windows")]
mod windows;

#[derive(Debug, Clone, PartialEq)]
pub enum SchemeStatus {
    Installed { path: String },
    NotInstalled,
}

impl std::fmt::Display for SchemeStatus {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Installed { path } => write!(f, "Installed at {path}"),
            Self::NotInstalled => write!(f, "Not installed"),
        }
    }
}

pub fn install() -> Result<()> {
    platform_install()
}

pub fn uninstall() -> Result<()> {
    platform_uninstall()
}

pub fn status() -> Result<SchemeStatus> {
    platform_status()
}

// ──────────────────────────── macOS ────────────────────────────

#[cfg(target_os = "macos")]
fn platform_install() -> Result<()> { macos::install() }

#[cfg(target_os = "macos")]
fn platform_uninstall() -> Result<()> { macos::uninstall() }

#[cfg(target_os = "macos")]
fn platform_status() -> Result<SchemeStatus> { macos::status() }

// ──────────────────────────── Linux ────────────────────────────

#[cfg(target_os = "linux")]
fn platform_install() -> Result<()> { linux::install() }

#[cfg(target_os = "linux")]
fn platform_uninstall() -> Result<()> { linux::uninstall() }

#[cfg(target_os = "linux")]
fn platform_status() -> Result<SchemeStatus> { linux::status() }

// ──────────────────────────── Windows ────────────────────────────

#[cfg(target_os = "windows")]
fn platform_install() -> Result<()> { windows::install() }

#[cfg(target_os = "windows")]
fn platform_uninstall() -> Result<()> { windows::uninstall() }

#[cfg(target_os = "windows")]
fn platform_status() -> Result<SchemeStatus> { windows::status() }

// ──────────────────────────── Fallback ────────────────────────────

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_install() -> Result<()> {
    anyhow::bail!("URL scheme registration is not supported on this platform")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_uninstall() -> Result<()> {
    anyhow::bail!("URL scheme registration is not supported on this platform")
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
fn platform_status() -> Result<SchemeStatus> {
    Ok(SchemeStatus::NotInstalled)
}
