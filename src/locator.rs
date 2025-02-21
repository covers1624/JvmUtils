pub mod gradle;
pub mod intellij;
pub mod platform;

use crate::install::{JavaInstall, JavaVersion};
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
    props: LocateProperties,
    children: Vec<Box<dyn JavaLocator>>,
}

impl LocatorBuilder {
    pub fn new() -> Self {
        Default::default()
    }

    pub fn use_javaw(&mut self) -> &mut Self {
        self.props.use_javaw = true;
        self
    }

    pub fn ignore_openj9(&mut self) -> &mut Self {
        self.props.ignore_openj9 = true;
        self
    }

    pub fn jdk_only(&mut self) -> &mut Self {
        self.props.jdk_only = true;
        self
    }

    pub fn filter(&mut self, version: JavaVersion) -> &mut Self {
        self.props.filter = Some(version);
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
            .filter_map(|e| e.locate(&self.props))
            .flatten()
            .collect();
        vec
    }
}


#[derive(Default)]
pub struct LocateProperties {
    pub use_javaw: bool,
    pub ignore_openj9: bool,
    pub jdk_only: bool,
    pub filter: Option<JavaVersion>,
}

/// A locator capable of finding Java installations somewhere on the system.
pub trait JavaLocator {
    /// Finds all available Java installations.
    ///
    /// * `props` - The properties to filter any JVM's.
    ///
    /// # Returns
    /// Some containing the JVM's found, otherwise None.
    fn locate(&self, props: &LocateProperties) -> Option<Vec<JavaInstall>>;
}

pub(crate) fn find_add_install(installs: &mut Vec<JavaInstall>, props: &LocateProperties, path: impl AsRef<Path>) -> Option<()> {
    let executable = JavaInstall::get_java_executable(&path, props.use_javaw);
    if !executable.exists() {
        return None;
    }

    let install = JavaInstall::parse(executable)?;
    #[cfg(feature = "logging")]
    debug!("Found install for {:?} at {:?}.", &install.lang_version, &install.java_home);

    add_install(installs, props, Some(install));
    Some(())
}

fn add_install(installs: &mut Vec<JavaInstall>, props: &LocateProperties, install: Option<JavaInstall>) {
    let to_add: Vec<JavaInstall> = install.into_iter()
        .filter(|e| !installs.iter().any(|existing| existing.java_home.eq(&e.java_home)))
        .filter(|e| props.filter.is_none() || props.filter.eq(&Some(e.lang_version.clone())))
        .filter(|e| !props.ignore_openj9 || !e.is_openj9)
        .filter(|e| !props.jdk_only || !e.is_jdk)
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

pub(crate) fn scan_folder(vec: &mut Vec<JavaInstall>, props: &LocateProperties, dir: impl AsRef<Path>) {
    #[cfg(feature = "logging")]
    debug!("Scanning folder for JVM's: {:?}", dir.as_ref());
    for entry in list_dir(dir) {
        let candidate_path = entry.path();
        if !candidate_path.is_dir() {
            continue;
        }
        if find_add_install(vec, props, &candidate_path).is_some() {
            continue;
        }

        let inners = list_dir(candidate_path);
        if inners.len() == 1 {
            find_add_install(vec, props, inners[0].path());
        }
    }
}
