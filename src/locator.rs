pub mod gradle;
pub mod intellij;
pub mod platform;

use crate::install::{JavaInstall, JavaVersion, Vendor};
use crate::locator::gradle::GradleJavaLocator;
use crate::locator::intellij::IntelliJJavaLocator;
use crate::locator::platform::PlatformJavaLocator;
#[cfg(feature = "logging")]
use log::debug;
use std::fs;
use std::path::Path;

/// A modular Java locator.
#[derive(Default)]
pub struct LocatorBuilder {
    use_javaw: bool,
    ignore_openj9: bool,
    jdk_only: bool,
    filter: Option<JavaVersion>,
    vendor_filter: Option<Vendor>,
    children: Vec<Box<dyn JavaLocator>>,
}

impl LocatorBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn use_javaw(&mut self) -> &mut Self {
        self.use_javaw = true;
        self
    }

    pub fn ignore_openj9(&mut self) -> &mut Self {
        self.ignore_openj9 = true;
        self
    }

    pub fn jdk_only(&mut self) -> &mut Self {
        self.jdk_only = true;
        self
    }

    pub fn filter(&mut self, version: &JavaVersion) -> &mut Self {
        self.filter = Some(version.clone());
        self
    }

    pub fn vendor_filter(&mut self, vendor: &Vendor) -> &mut Self {
        self.vendor_filter = Some(vendor.clone());
        self
    }

    pub fn with_platform_locator(&mut self) -> &mut Self {
        self.with_locator(Box::new(PlatformJavaLocator::new()))
    }

    pub fn with_gradle_locator(&mut self) -> &mut Self {
        self.with_locator(Box::new(GradleJavaLocator::new()))
    }

    pub fn with_intellij_locator(&mut self) -> &mut Self {
        self.with_locator(Box::new(IntelliJJavaLocator::new()))
    }

    pub fn with_locator(&mut self, locator: Box<dyn JavaLocator>) -> &mut Self {
        self.children.push(locator);
        self
    }

    pub fn locate(&self) -> Vec<JavaInstall> {
        let vec: Vec<JavaInstall> = self.children.iter()
            .filter_map(|e| e.locate())
            .flatten()
            .filter(|e| self.filter.is_none() || self.filter.eq(&Some(e.lang_version.clone())))
            .filter(|e| !self.ignore_openj9 || !e.is_openj9)
            .filter(|e| !self.jdk_only || e.is_jdk)
            .filter(|e| self.vendor_filter.is_none() || self.vendor_filter.eq(&e.known_vendor))
            .collect();
        vec
    }
}

/// A locator capable of finding Java installations somewhere on the system.
pub trait JavaLocator {
    /// Finds all available Java installations.
    ///
    /// * `props` - The properties to filter any JVM's.
    ///
    /// # Returns
    /// Some containing the JVM's found, otherwise None.
    fn locate(&self) -> Option<Vec<JavaInstall>>;
}

pub(crate) fn find_add_install(installs: &mut Vec<JavaInstall>, path: impl AsRef<Path>) -> Option<()> {
    // Always use javaw when probing on windows, to avoid console windows being created.
    let executable = JavaInstall::get_java_executable(&path, true);
    if !executable.exists() {
        return None;
    }

    let install = JavaInstall::parse(executable)?;
    #[cfg(feature = "logging")]
    debug!("Found install for {:?} at {:?}.", &install.lang_version, &install.java_home);

    add_install(installs, Some(install));
    Some(())
}

fn add_install(installs: &mut Vec<JavaInstall>, install: Option<JavaInstall>) {
    let to_add: Vec<JavaInstall> = install.into_iter()
        .filter(|e| !installs.iter().any(|existing| existing.java_home.eq(&e.java_home)))
        .collect();

    installs.extend(to_add);
}

pub(crate) fn list_dir(dir: impl AsRef<Path>) -> Vec<fs::DirEntry> {
    fs::read_dir(dir.as_ref())
        .ok()
        .into_iter()
        .flatten()
        .filter_map(|e| e.ok())
        .collect()
}

pub(crate) fn scan_folder(vec: &mut Vec<JavaInstall>, dir: impl AsRef<Path>) {
    #[cfg(feature = "logging")]
    debug!("Scanning folder for JVM's: {:?}", dir.as_ref());
    for entry in list_dir(dir) {
        let candidate_path = entry.path();
        if !candidate_path.is_dir() {
            continue;
        }
        if find_add_install(vec, &candidate_path).is_some() {
            continue;
        }

        let inners = list_dir(candidate_path);
        if inners.len() == 1 {
            find_add_install(vec, inners[0].path());
        }
    }
}
