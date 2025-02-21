#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "macos")]
mod macos;
#[cfg(target_os = "windows")]
mod windows;

/// A JavaLocator capable of locating JVM's installed in known platform
/// specific system paths or registry locations.
#[derive(Default)]
pub struct PlatformJavaLocator {}

impl PlatformJavaLocator {
    pub fn new() -> Self {
        Default::default()
    }
}
