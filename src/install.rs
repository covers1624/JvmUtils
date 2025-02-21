use crate::extract::extract_java_properties;
use num_enum::TryFromPrimitive;
use regex::Regex;
#[cfg(feature = "serde")]
use serde::{Deserialize, Serialize};
use std::path::{Path, PathBuf};

/// Represents a limited set of current and future java versions.
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone, TryFromPrimitive)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum JavaVersion {
    Java1_1 = 1,
    Java1_2 = 2,
    Java1_3 = 3,
    Java1_4 = 4,
    Java1_5 = 5,
    Java1_6 = 6,
    Java1_7 = 7,
    Java1_8 = 8,
    Java9 = 9,
    Java10 = 10,
    Java11 = 11,
    Java12 = 12,
    Java13 = 13,
    Java14 = 14,
    Java15 = 15,
    Java16 = 16,
    Java17 = 17,
    Java18 = 18,
    Java19 = 19,
    Java20 = 20,
    Java21 = 21,
    Java22 = 22,
    Java23 = 23,
    Java24 = 24,
    Java25 = 25,
    Java26 = 26,
    Java27 = 27,
    Java28 = 28,
    Java29 = 29,
    Java30 = 30,
    Java31 = 31,
    Java32 = 32,
    Java33 = 33,
    Java34 = 34,
    Java35 = 35,
    Java36 = 36,
    Java37 = 37,
    Java38 = 38,
    Java39 = 39,
    Java40 = 40,
}

impl JavaVersion {
    /// Parse a Java version from a string such as `17.0.10.7`.
    ///
    /// * `version` - The Version string to parse.
    ///
    /// # Returns
    /// Some containing the version result, otherwise None.
    pub fn parse(version: &str) -> Option<Self> {
        let re = Regex::new("^([0-9.]*)")
            .ok()?;

        let matched = &re.captures(version)?[0];
        let split: Vec<&str> = matched.split(".").collect();
        let v_split: Vec<usize> = split
            .into_iter()
            .map(|e| e.parse::<usize>().ok())
            .flatten()
            .collect();
        if v_split.len() > 1 && v_split[0] == 1 {
            return Self::try_from(v_split[1]).ok();
        }
        Self::try_from(v_split[0]).ok()
    }
}

/// Represents a limited set of CPU architectures.
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Architecture {
    X86 = 1,
    X86_64 = 2,
    Arm = 3,
    Aarch64 = 4,
    Powerpc = 5,
    Powerpc64 = 6,
}

impl Architecture {
    /// Get the current system CPU Architecture.
    ///
    /// # Returns
    /// Some containing the current architecture, otherwise None.
    pub fn current() -> Option<Self> {
        Self::parse(std::env::consts::ARCH)
    }

    /// Parses an architecture from the given string.
    ///
    /// Most of these variants exist to support weird and wonderful Java architecture names,
    /// along with the 'standardized' names adopted by Rust.
    ///
    /// * `arch` - The string to parse.
    ///
    /// # Returns
    /// Some containing the parsed architecture, otherwise None.
    pub fn parse(arch: &str) -> Option<Self> {
        match arch {
            "x86" | "i386" => Some(Self::X86),
            "x86_64" | "x64" | "amd64" => Some(Self::X86_64),
            "arm" => Some(Self::Arm),
            "aarch64" => Some(Self::Aarch64),
            "powerpc" | "ppc" => Some(Self::Powerpc),
            "powerpc64" | "ppc64" => Some(Self::Powerpc64),
            _ => None
        }
    }
}

/// Represents a limited set of Operating Systems.
#[repr(usize)]
#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum OS {
    Linux = 1,
    MacOS = 2,
    Windows = 3,
}

impl OS {
    /// Get the current Operating System.
    ///
    /// # Returns
    /// Some containing the current Operating System, otherwise None.
    pub fn current() -> Option<Self> {
        match std::env::consts::OS {
            "linux" => Some(Self::Linux),
            "macos" => Some(Self::MacOS),
            "windows" => Some(Self::Windows),
            _ => None
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
pub enum Vendor {
    Unknown,
    AdoptOpenJdk,
    Corretto,
    GraalVmCe,
    Jetbrains,
    Microsoft,
    OpenJdk,
    Temurin,
    Zulu,
}

impl Vendor {
    pub fn parse(vendor: &str) -> Option<Vendor> {
        if vendor.contains("AdoptOpenJdk") || vendor.contains("AdoptOpenJDK") {
            Some(Self::AdoptOpenJdk)
        } else if vendor.contains("Amazon.com") {
            Some(Self::Corretto)
        } else if vendor.contains("Eclipse") || vendor.contains("Temurin") {
            Some(Self::Temurin)
        } else if vendor.contains("GraalVM Community") || vendor.contains("GraalVmCe") {
            Some(Self::GraalVmCe)
        } else if vendor.contains("JetBrains") {
            Some(Self::Jetbrains)
        } else if vendor.contains("Microsoft") {
            Some(Self::Microsoft)
        } else if vendor.contains("Oracle Corporation") || vendor.contains("Sun Microsystems Inc") {
            Some(Self::OpenJdk)
        } else if vendor.contains("Azul Systems") {
            Some(Self::Zulu)
        } else {
            None
        }
    }
}

#[cfg_attr(feature = "serde", derive(Serialize, Deserialize))]
#[derive(Debug, Clone)]
pub struct JavaInstall {
    pub lang_version: JavaVersion,
    pub java_home: PathBuf,
    pub known_vendor: Option<Vendor>,
    pub vendor: String,
    pub impl_name: String,
    pub impl_version: String,
    pub runtime_name: String,
    pub runtime_version: String,
    pub architecture: Architecture,

    pub is_openj9: bool,
    pub is_jdk: bool,
}

impl JavaInstall {
    fn new(
        install_dir: impl AsRef<Path>,
        vendor: String,
        impl_name: String,
        impl_version: String,
        runtime_name: String,
        runtime_version: String,
        architecture: Architecture,
    ) -> Option<Self> {
        Some(Self {
            lang_version: JavaVersion::parse(&impl_version.as_str())?,
            java_home: install_dir.as_ref().to_path_buf(),
            known_vendor: Vendor::parse(&vendor),
            vendor,
            impl_name: impl_name.clone(),
            impl_version,
            runtime_name,
            runtime_version,
            architecture,
            is_openj9: impl_name.contains("j9"),
            is_jdk: Self::get_executable(install_dir, "javac").exists(),
        })
    }

    /// Retrieves the bin directory for a given installation directory.
    ///
    /// This method transparently resolves any platform specific offsets from the
    /// root directory of the installation. (Such as the macOS Contents/Home dir.)
    ///
    /// * `install_dir` - The installation directory.
    ///
    /// # Returns
    /// The bin directory.
    pub fn get_bin_dir(install_dir: impl AsRef<Path>) -> PathBuf {
        Self::get_home_dir(install_dir).join("bin")
    }

    /// Retrieves the potentially platform-specific path the bin directory is
    /// expected to reside.
    ///
    /// * `install_dir` - The installation directory.
    ///
    /// # Returns
    /// The directory containing the bin directory.
    pub fn get_home_dir(install_dir: impl AsRef<Path>) -> PathBuf {
        if cfg!(target_os = "macos") {
            install_dir.as_ref().join("Contents/Home")
        } else {
            install_dir.as_ref().to_path_buf()
        }
    }

    /// Retrieves the 'java' executable path for the given home directory.
    ///
    /// * `home_dir` - The home directory for the java installation.
    /// * `use_javaw` - When on windows, if `javaw` should be used instead.
    ///
    /// # Returns
    /// The path to the java executable.
    pub fn get_java_executable(home_dir: impl AsRef<Path>, use_javaw: bool) -> PathBuf {
        fn executable_name(use_javaw: bool) -> &'static str {
            if cfg!(target_os = "windows") && use_javaw {
                "javaw"
            } else {
                "java"
            }
        }
        Self::get_executable(home_dir, executable_name(use_javaw))
    }


    /// Get the path for an executable within a given home directory.
    ///
    /// Any platform specific file extensions are automatically appended to the executable.
    ///
    /// * `home_dir` - The home directory for the java installation.
    /// * `executable` - The executable to use.
    ///
    /// # Returns
    /// The path to the specified executable.
    pub fn get_executable(home_dir: impl AsRef<Path>, executable: &str) -> PathBuf {
        pub fn exe_suffix() -> &'static str {
            if cfg!(target_os = "windows") {
                "exe"
            } else {
                ""
            }
        }
        home_dir.as_ref().join("bin").join(executable).with_extension(exe_suffix())
    }

    /// Parse a Java Installation's properties and attributes from the given executable.
    ///
    /// The executable is not required to exist.
    ///
    /// * `executable` - The executable path.
    ///
    /// # Returns
    /// Maybe a JavaInstall with extracted properties and attributes.
    pub fn parse(executable: impl AsRef<Path>) -> Option<Self> {
        static PROPERTIES: [&'static str; 9] = [
            "java.home",
            "java.version",
            "java.vendor",
            "os.arch",
            "java.vm.name",
            "java.vm.version",
            "java.runtime.name",
            "java.runtime.version",
            "java.class.version",
        ];
        let props = extract_java_properties(executable, PROPERTIES)?;

        let java_home = props.get("java.home")
            .map(Path::new)?;

        let java_home_real: &Path;
        if java_home.file_name()?.eq("jre") && java_home.parent()?.join("bin").exists() {
            java_home_real = java_home.parent()?;
        } else {
            java_home_real = java_home;
        }

        Self::new(
            java_home_real,
            props.get("java.vendor")?.into(),
            props.get("java.vm.name")?.into(),
            props.get("java.version")?.into(),
            props.get("java.runtime.name")?.into(),
            props.get("java.runtime.version")?.into(),
            Architecture::parse(props.get("os.arch")?)?,
        )
    }
}
