mod adoptium;

use crate::hashing::hash_directory;
use crate::install::{Architecture, JavaInstall, JavaVersion, Vendor};
use crate::provisioning::adoptium::AdoptiumProvisioner;
use crate::{log_debug, log_warn};
use mvn_version::ComparableVersion;
use pathdiff::diff_paths;
use rand::distr::Alphanumeric;
use rand::Rng;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::ffi::OsStr;
use std::fs::File;
use std::io::{Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, io};

pub struct InstallationManager {
    base_dir: PathBuf,
    installs: HashMap<String, Manifest>,
    provisioners: Vec<Box<dyn JvmProvisioner>>,
}

pub fn adoptium() -> Box<dyn JvmProvisioner> {
    Box::new(AdoptiumProvisioner::new())
}

/// Manages and provides JVM installations for a given system.
impl InstallationManager {
    pub fn new(base_dir: impl AsRef<Path>) -> io::Result<Self> {
        let base_dir = base_dir.as_ref();
        fs::create_dir_all(&base_dir)?;

        let installs: HashMap<String, Manifest> = fs::read_dir(base_dir)
            .ok()
            .into_iter()
            .flatten()
            .flatten()
            .map(|e| e.path())
            .filter(|e| e.is_file() && e.extension().eq(&Some(OsStr::new("json"))))
            .flat_map(|path| -> Option<(String, Manifest)> {
                let file = File::open(&path);
                if file.is_err() {
                    log_warn!("Failed to open file {:?}, it will be ignored. {:?}", &path, file.unwrap_err());
                    return None;
                }
                let manifest = serde_json::from_reader(file.unwrap());
                if manifest.is_err() {
                    log_warn!("Failed to read manifest json from {:?}, it will be ignored. {:?}", &path, manifest.unwrap_err());
                    return None;
                }

                let manifest: Manifest = manifest.unwrap();
                Some((manifest.id.clone(), manifest))
            })
            .collect();

        Ok(Self {
            base_dir: fs::canonicalize(base_dir.to_path_buf())?,
            installs,
            provisioners: Vec::new(),
        })
    }

    pub fn with_provisioner(&mut self, provisioner: Box<dyn JvmProvisioner>) -> &mut Self {
        self.provisioners.push(provisioner);
        self
    }

    /// Ask the provisioning system to provide a JVM matching the given provisioning
    /// request.
    ///
    /// The provisioning system will make all efforts to provide a JVM that it has
    /// previously provisioned, or a matching one was found on the system from
    /// the given locator configuration. After all these options are exhausted, a
    /// new JVM will attempt to be provisioned from the configured providers.
    ///
    /// * `request`- The provisioning request to fulfill.
    ///
    /// # Returns
    /// A result containing the home directory of the newly provisioned jvm.
    // TODO this should return something that includes the id and the home directory.
    //      We should also provide an endpoint to 'validate' the installation for a given id.
    pub fn provide(&mut self, request: ProvisionRequest) -> io::Result<PathBuf> {
        // Check if we have done this before, if so, return it.
        if let Some(existing) = self.find_existing(&request) {
            return Ok(existing);
        }

        // We have to provision one! yay
        for b in &self.provisioners {
            let result = b.provision(&self.base_dir, &request);
            if result.is_ok() {
                let provisioned = result?;
                log_debug!("Jdk provisioned. {:?}", &provisioned.install_dir);

                log_debug!("Creating installation hash..");
                let hash = hash_directory(&provisioned.install_dir)?;
                let manifest = Manifest {
                    id: self.new_unique_id(),
                    version: provisioned.version,
                    known_vendor: provisioned.known_vendor,
                    vendor: provisioned.vendor,
                    semver: provisioned.semver,
                    architecture: provisioned.architecture,
                    install_dir: diff_paths(&provisioned.install_dir, &self.base_dir).unwrap(),
                    is_jdk: provisioned.is_jdk,
                    hash: hash,
                };
                log_debug!("Writing manifest..");
                let install_dir = self.base_dir.join(&manifest.install_dir);

                fs::write(self.base_dir.join(&manifest.id).with_extension("json"), serde_json::to_string_pretty(&manifest)?)?;

                self.installs.insert(manifest.id.clone(), manifest);
                return Ok(JavaInstall::get_home_dir(install_dir));
            }

            log_warn!("Provisioner {} failed to provision jvm {:?}. {}", b.name(), &request, result.unwrap_err());
        }

        Err(Error::new(ErrorKind::NotFound, "No providers were able to fulfill the request."))
    }

    fn find_existing(&self, request: &ProvisionRequest) -> Option<PathBuf> {
        log_debug!("Trying to fulfill request for {:?}(semver={:?}, jre={}, x86_on_arch={}) from previous provisions.", &request.version, &request.semver, &request.jre_allowed, &request.x86_on_arm);
        let partial_filter = self.installs.values()
            .filter(|e| request.jre_allowed || e.is_jdk)
            .filter(|e| e.version == request.version)
            .filter(|e| request.semver.is_none() || request.semver.eq(&Some(e.semver.clone()))); // ew we need to clone this.

        #[cfg(any(target_os = "windows", target_os = "macos"))]
        let arch = Architecture::current();
        #[cfg(any(target_os = "windows", target_os = "macos"))]
        let partial_filter = partial_filter
            .filter(|e| arch.is_none() || e.architecture == arch.unwrap() || (request.x86_on_arm && e.architecture == Architecture::X86_64 && arch.unwrap() == Architecture::Aarch64));

        let mut candidates: Vec<&Manifest> = partial_filter
            .collect();
        // Hot wire to avoid pointless stuff.
        if candidates.is_empty() {
            return None;
        }

        candidates.sort_by_key(|e| ComparableVersion::new(&e.semver));

        let chosen = candidates.first()?;
        let dir = JavaInstall::get_home_dir(self.base_dir.join(&chosen.install_dir));
        log_debug!("Found existing jvm {:?}(semver={}, jre={}, arch={:?}, id={}) at {:?}", chosen.version, chosen.semver, !chosen.is_jdk, chosen.architecture, chosen.id, &dir);
        Some(dir)
    }

    fn new_unique_id(&self) -> String {
        let mut rng = rand::rng();
        loop {
            let str: String = (0..6)
                .map(|_| rng.sample(Alphanumeric) as char)
                .collect();
            if !self.installs.contains_key(&str) {
                return str;
            }
        }
    }
}

/// Represents a request for the provisioning system to fulfill.
#[derive(Debug)]
pub struct ProvisionRequest {
    version: JavaVersion,
    semver: Option<String>,
    jre_allowed: bool,
    x86_on_arm: bool,
}

impl ProvisionRequest {
    /// Create a ProvisionRequest for the specified JavaVersion.
    ///
    /// This will accept any version as long as the major matches.
    ///
    /// * `version` - The major Java Version requested.
    ///
    /// # Returns
    /// A request for the given major Java Version
    pub fn from_version(version: JavaVersion) -> Self {
        Self {
            version,
            semver: None,
            jre_allowed: false,
            x86_on_arm: false,
        }
    }

    /// Create a ProvisionRequest for the specified Java semver version.
    ///
    /// This will fulfill the request for only the specified semver
    /// version.
    ///
    /// * `semver` - The semver version requested.
    ///
    /// # Returns
    /// Some containing the request, otherwise None. This may fail if
    /// the major java version is unable to be parsed from the given semver string,
    /// which should be impossible, but better than panik.
    pub fn from_semver(semver: String) -> Option<Self> {
        Some(Self {
            version: JavaVersion::parse(&semver)?,
            semver: Some(semver),
            jre_allowed: false,
            x86_on_arm: false,
        })
    }

    /// Modify the ProvisionRequest to specify if it's allowed to fulfill the
    /// request with a jre installation.
    ///
    /// This does not prevent a jdk from being used to fulfill the request,
    /// this only changes the semantics of provisioning new jdks. If one is
    /// to be provisioned and this is set to `true`, the new JVM will be a jre,
    /// if available. If the provisioning endpoints are unable to find a jre to
    /// provision, a jdk will be provisioned.
    ///
    /// Conversely, if this is set to `false`, the request *must* be fulfilled
    /// with a jdk.
    ///
    /// * `jre_allowed` - If the provisioning system is allowed to return a jre.
    ///
    /// # Returns
    /// The same builder ref, for chaining.
    pub fn with_jre_only(&mut self, jre_allowed: bool) -> &mut Self {
        self.jre_allowed = jre_allowed;
        self
    }

    /// Modify the ProvisionRequest to specify if it's allowed to fulfill the
    /// request with an x86_64 JVM on an aarch64 system.
    ///
    /// This assumes the OS is capable of running x86_64 vms via some
    /// form of emulation layer, such as macOS Rosetta 2.
    ///
    /// This setting only takes effect on aarch64 platforms. Only
    /// Windows and macOS will attempt to perform this. Currently, Linux
    /// is explicitly excluded as Linux aarch64 is known to be a primary
    /// target of OpenJDK. // TODO make this a semantic of the provider? Some vendors may not have these semantics.
    ///
    /// * `x86_on_arm` - If the provisioning system is allowed to fulfill an aarch64
    /// request with an x86_64 JVM on macOS or Windows.
    ///
    /// # Returns
    /// The same builder ref, for chaining.
    pub fn with_x86_on_arm(&mut self, x86_on_arm: bool) -> &mut Self {
        self.x86_on_arm = x86_on_arm;
        self
    }
}

#[derive(Debug)]
pub struct ProvisionResult {
    version: JavaVersion,
    known_vendor: Option<Vendor>,
    vendor: String,
    semver: String,
    architecture: Architecture,
    install_dir: PathBuf,
    is_jdk: bool,
}

pub trait JvmProvisioner {
    fn name(&self) -> &'static str;

    fn provision(&self, base_dir: &Path, request: &ProvisionRequest) -> io::Result<ProvisionResult>;
}

#[derive(Serialize, Deserialize)]
#[derive(Debug, Clone)]
struct Manifest {
    id: String,
    version: JavaVersion,
    known_vendor: Option<Vendor>,
    vendor: String,
    semver: String,
    architecture: Architecture,
    install_dir: PathBuf,
    is_jdk: bool,
    hash: String,
}

impl Manifest {}
