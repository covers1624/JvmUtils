use crate::install::JavaInstall;
use crate::locator::{scan_folder, JavaLocator};
use crate::log_debug;

/// A JavaLocator capable of locating JVM's installed by Gradle toolchains.
#[derive(Default)]
pub struct GradleJavaLocator {}

impl GradleJavaLocator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl JavaLocator for GradleJavaLocator {
    fn locate(&self) -> Option<Vec<JavaInstall>> {
        let dir = dirs::home_dir()?.join(".gradle/jdks");
        log_debug!("Searching for JVM's installed by Gradle toolchains in path: {:?}", &dir);

        let mut vec: Vec<JavaInstall> = Vec::new();
        scan_folder(&mut vec, &dir);
        Some(vec)
    }
}
