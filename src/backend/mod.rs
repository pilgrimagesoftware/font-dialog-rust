//! Per-platform [`crate::FontDialogBackend`] implementations, selected at
//! compile time by `cfg(target_os = ...)`.

#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

#[cfg(target_os = "linux")]
mod linux;

#[cfg(target_os = "macos")]
pub fn current() -> impl crate::FontDialogBackend {
    macos::MacOsBackend
}

#[cfg(target_os = "windows")]
pub fn current() -> impl crate::FontDialogBackend {
    windows::WindowsBackend
}

#[cfg(target_os = "linux")]
pub fn current() -> impl crate::FontDialogBackend {
    linux::LinuxBackend
}

#[cfg(not(any(target_os = "macos", target_os = "windows", target_os = "linux")))]
compile_error!("font-dialog has no backend for this target platform");
