use crate::install::JavaInstall;
use crate::locator::platform::PlatformJavaLocator;
use crate::locator::{scan_folder, JavaLocator, LocateProperties};
#[cfg(feature = "logging")]
use log::debug;
use std::path::Path;

impl JavaLocator for PlatformJavaLocator {
    fn locate(&self, props: &LocateProperties) -> Option<Vec<JavaInstall>> {
        #[cfg(feature = "logging")]
        debug!("Searching for JVM's installed in common system locations.");

        let mut vec: Vec<JavaInstall> = Vec::new();
        // Oracle
        scan_folder(&mut vec, props, Path::new("/usr/java"));

        // Common distro locations
        scan_folder(&mut vec, props, Path::new("/usr/lib/jvm"));
        scan_folder(&mut vec, props, Path::new("/usr/lib32/jvm"));

        // Manually installed locations
        scan_folder(&mut vec, props, Path::new("/opt/jdk"));
        scan_folder(&mut vec, props, Path::new("/opt/jdks"));

        scan_folder(&mut vec, props, &dirs::home_dir()?.join(".local/jdks"));
        Some(vec)
    }
}
