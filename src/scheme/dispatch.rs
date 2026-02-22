use anyhow::Result;

use super::SchemeStatus;

#[cfg(target_os = "macos")]
use super::macos;

#[cfg(target_os = "linux")]
use super::linux;

#[cfg(target_os = "windows")]
use super::windows;

// LLVM_COV_EXCL_START
#[cfg(target_os = "macos")]
pub(super) fn platform_install() -> Result<()> {
    macos::install()
}
#[cfg(target_os = "macos")]
pub(super) fn platform_uninstall() -> Result<()> {
    macos::uninstall()
}
// LLVM_COV_EXCL_STOP
#[cfg(target_os = "macos")]
pub(super) fn platform_status() -> Result<SchemeStatus> {
    macos::status()
}

#[cfg(target_os = "linux")]
pub(super) fn platform_install() -> Result<()> {
    linux::install()
}
#[cfg(target_os = "linux")]
pub(super) fn platform_uninstall() -> Result<()> {
    linux::uninstall()
}
#[cfg(target_os = "linux")]
pub(super) fn platform_status() -> Result<SchemeStatus> {
    linux::status()
}

#[cfg(target_os = "windows")]
pub(super) fn platform_install() -> Result<()> {
    windows::install()
}
#[cfg(target_os = "windows")]
pub(super) fn platform_uninstall() -> Result<()> {
    windows::uninstall()
}
#[cfg(target_os = "windows")]
pub(super) fn platform_status() -> Result<SchemeStatus> {
    windows::status()
}

#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub(super) fn platform_install() -> Result<()> {
    anyhow::bail!("URL scheme registration is not supported on this platform")
}
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub(super) fn platform_uninstall() -> Result<()> {
    anyhow::bail!("URL scheme registration is not supported on this platform")
}
#[cfg(not(any(target_os = "macos", target_os = "linux", target_os = "windows")))]
pub(super) fn platform_status() -> Result<SchemeStatus> {
    Ok(SchemeStatus::NotInstalled)
}
