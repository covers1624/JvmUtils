use crate::install::JavaInstall;
use crate::locator::platform::PlatformJavaLocator;
use crate::locator::{scan_folder, JavaLocator};
use std::path::Path;
use crate::log_debug;

impl JavaLocator for PlatformJavaLocator {
    fn locate(&self) -> Option<Vec<JavaInstall>> {
        log_debug!("Searching for JVM's installed in common system locations.");

        let mut vec: Vec<JavaInstall> = Vec::new();
        // Oracle
        scan_folder(&mut vec, Path::new("/usr/java"));

        // Common distro locations
        scan_folder(&mut vec, Path::new("/usr/lib/jvm"));
        scan_folder(&mut vec, Path::new("/usr/lib32/jvm"));

        // Manually installed locations
        scan_folder(&mut vec, Path::new("/opt/jdk"));
        scan_folder(&mut vec, Path::new("/opt/jdks"));

        scan_folder(&mut vec, &dirs::home_dir()?.join(".local/jdks"));
        Some(vec)
    }
}
