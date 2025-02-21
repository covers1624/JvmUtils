use crate::install::JavaInstall;
use crate::locator::{scan_folder, JavaLocator, LocateProperties};
use log::debug;

/// A JavaLocator capable of locating JVM's installed by Intellij IDEA.
#[derive(Default)]
pub struct IntelliJJavaLocator {}

impl IntelliJJavaLocator {
    pub fn new() -> Self {
        Default::default()
    }
}

impl JavaLocator for IntelliJJavaLocator {
    fn locate(&self, props: &LocateProperties) -> Option<Vec<JavaInstall>> {
        let dir = dirs::home_dir()?.join(".jdks");
        debug!("Searching for JVM's installed by Intellij toolchains in path: {:?}", &dir);

        let mut vec: Vec<JavaInstall> = Vec::new();
        scan_folder(&mut vec, props, &dir);
        Some(vec)
    }
}
