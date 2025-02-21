use crate::install::JavaInstall;
use crate::locator::platform::PlatformJavaLocator;
use crate::locator::{scan_folder, JavaLocator};
#[cfg(feature = "logging")]
use log::debug;
use std::path::Path;

impl JavaLocator for PlatformJavaLocator {
    fn locate(&self) -> Option<Vec<JavaInstall>> {
        #[cfg(feature = "logging")]
        debug!("Searching for JVM's installed in common system locations.");

        let mut vec: Vec<JavaInstall> = Vec::new();
        scan_folder(&mut vec, Path::new("/Library/Java/JavaVirtualMachines/"));
        scan_folder(&mut vec, Path::new("/System/Library/Java/JavaVirtualMachines/"));
        Some(vec)
    }
}
