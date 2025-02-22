use crate::hashing::sha256_file;
use crate::install::{Architecture, JavaVersion, OS};
use crate::provisioning::{JvmProvisioner, ProvisionRequest, ProvisionResult};
use crate::{install, log_info};
use flate2::read::GzDecoder;
use serde::Deserialize;
use std::fs::{DirEntry, File};
use std::io::{copy, BufReader, Error, ErrorKind};
use std::path::{Path, PathBuf};
use std::{fs, io};
use tar::Archive;
use zip::ZipArchive;

const ADOPTIUM_API: &'static str = "https://api.adoptium.net";

pub struct AdoptiumProvisioner {}
impl AdoptiumProvisioner {
    pub fn new() -> AdoptiumProvisioner {
        AdoptiumProvisioner {}
    }
}

impl JvmProvisioner for AdoptiumProvisioner {
    fn name(&self) -> &'static str {
        "Adoptium/Eclipse Temurin"
    }

    fn provision(&self, base_dir: &Path, request: &ProvisionRequest) -> io::Result<ProvisionResult> {
        log_info!("Attempting to find compatible Adoptium JVM for request {:?}.", request);
        if let Some(selected) = Self::select_compatible(request) {
            log_info!("Selected Adoptium release: {:?}", &selected);
            let archive_path = base_dir.join(&selected.name);

            Self::download_binary(&archive_path, &selected)?;
            let install_dir = Self::extract(base_dir, &archive_path)?;
            fs::remove_file(&archive_path)?;

            return Ok(ProvisionResult {
                version: request.version,
                known_vendor: Some(install::Vendor::Temurin),
                vendor: "Temurin".into(),
                semver: selected.openjdk_version,
                architecture: selected.architecture,
                install_dir,
                is_jdk: selected.image_type == "jdk",
            });
        }

        Err(Error::new(ErrorKind::NotFound, "No compatible releases found for the given request."))
    }
}

impl AdoptiumProvisioner {
    fn extract(base_dir: impl AsRef<Path>, archive: impl AsRef<Path>) -> io::Result<PathBuf> {
        fn split_extension(archive: impl AsRef<Path>) -> Option<(String, String)> {
            let name = archive.as_ref().file_name()?.to_str()?;
            if name.ends_with(".zip") {
                Some((name[..name.len() - 4].into(), "zip".into()))
            } else if name.ends_with(".tar.gz") {
                Some((name[..name.len() - 7].into(), "tar.gz".into()))
            } else {
                None
            }
        }
        let archive = archive.as_ref();
        let (base, ext) = split_extension(archive).unwrap();
        let dest = base_dir.as_ref().join(base);
        log_info!("Extracting archive {:?} to {:?}.", archive, dest);
        if ext == "zip" {
            let file = File::open(archive)?;
            let mut archive = ZipArchive::new(BufReader::new(file))?;
            archive.extract(&dest)?;
        } else if ext == "tar.gz" {
            let file = File::open(archive)?;
            let mut archive = Archive::new(GzDecoder::new(file));
            archive.unpack(&dest)?;
        } else {
            return Err(Error::new(ErrorKind::NotFound, format!("Unknown archive format, unable to extract. {:?}", archive.extension())));
        }

        // This looks a bit weird, but here's the rub:
        // In order to facilitate x86 and arm64 installs next to each other
        // we just extract the entire zip/tar into a directory with the same name
        // as the zip, this just makes things infinately easier as there is already os/arch
        // metadata stored.
        // Consider the following paths:
        // base_dir = ./.jvms
        // archive = OpenJDK17U-jre_x64_linux_hotspot_17.0.14_7.tar.gz
        // dest = ./.jvms/OpenJDK17U-jre_x64_linux_hotspot_17.0.14_7/
        //
        // The archive contains a folder:
        // jdk-17.0.14+7-jre
        //
        // The final path we want to return from here is:
        // ./.jvms/OpenJDK17U-jre_x64_linux_hotspot_17.0.14_7/jdk-17.0.14+7-jre/

        let dir: Vec<DirEntry> = fs::read_dir(&dest)
            .into_iter()
            .flatten()
            .flatten()
            .collect();

        if dir.len() != 1 {
            // I have no idea what state we'd be in to have this occur,
            // we clearly did not extract a directory with stuff inside it,
            // there is more than one base file. So lets just return that and assume it'll be fine.
            Ok(dest.to_path_buf())
        } else {
            Ok(dir.first().unwrap().path())
        }
    }

    fn download_binary(dest: impl AsRef<Path>, selected: &SelectedRelease) -> io::Result<()> {
        let dest = dest.as_ref();
        log_info!("Downloading archive {} to {:?}.", selected.link, dest);
        let response = ureq::get(&selected.link).call();
        if let Err(error) = response {
            return Err(Error::new(ErrorKind::Other, error));
        }
        let mut reader = response.unwrap().into_body().into_reader();
        let mut file = File::create(dest)?;
        copy(&mut reader, &mut file)?;
        drop(file);

        let len = fs::metadata(dest)?.len();
        if len != (selected.size as u64) {
            return Err(Error::new(ErrorKind::Other, format!("Downloaded file does not match known file length. Got {}, expected {}", len, selected.size)));
        }

        let hash = sha256_file(dest)?;
        if hash != selected.checksum {
            return Err(Error::new(ErrorKind::Other, format!("Downloaded file does not match known file hash. Got {}, expected {}", hash, selected.checksum)));
        }

        Ok(())
    }

    fn select_compatible(request: &ProvisionRequest) -> Option<SelectedRelease> {
        let os = OS::current()?; // TODO allow provisioning jvms for not this os/arch?
        let arch = Architecture::current()?;
        // Try exact
        if let Some(release) = Self::api_request(&os, &arch, &request.version, &request.semver, request.jre_allowed) {
            return Some(release);
        }
        // If we want a JRE and one doesn't exist, try again for a jdk.
        if request.jre_allowed {
            if let Some(release) = Self::api_request(&os, &arch, &request.version, &request.semver, false) {
                return Some(release);
            }
        }

        // If we didn't find a VM for aarch64, and we are not on Linux (mac/windows), and we are allowed to substitute,
        // try again, but for an x86 vm.
        if arch == Architecture::Aarch64 && os != OS::Linux && request.x86_on_arm {
            // Try for a jre on x86
            if let Some(release) = Self::api_request(&os, &Architecture::X86_64, &request.version, &request.semver, request.jre_allowed) {
                return Some(release);
            }
            // If we want a JRE and one doesn't exist, try again for a jdk.
            if request.jre_allowed {
                if let Some(release) = Self::api_request(&os, &Architecture::X86_64, &request.version, &request.semver, false) {
                    return Some(release);
                }
            }
        }

        None
    }

    fn api_request(os: &OS, arch: &Architecture, version: &JavaVersion, semver: &Option<String>, jre: bool) -> Option<SelectedRelease> {
        let releases = ureq::get(Self::url(os, arch, version, semver, jre))
            .call()
            .ok()?
            .body_mut()
            .read_json::<Vec<Release>>()
            .ok()?;

        let release = releases.first()?;
        let binary = release.binaries.first()?;
        if let Some(pkg) = &binary.package {
            Some(SelectedRelease {
                name: pkg.name.clone(),
                link: pkg.link.clone(),
                size: pkg.size,
                checksum: pkg.checksum.clone(),
                image_type: binary.image_type.clone(),
                openjdk_version: release.version_data.openjdk_version.clone(),
                architecture: arch.clone(),
            })
        } else {
            None
        }
    }

    fn url(os: &OS, arch: &Architecture, version: &JavaVersion, semver: &Option<String>, jre: bool) -> String {
        let mut url = String::from(ADOPTIUM_API);
        url.push_str("/v3/assets");
        if let Some(semver) = semver {
            url.push_str("/version/");
            url.push_str(semver);
        } else {
            url.push_str("/feature_releases/");
            url.push_str(version.short_string().as_str());
            url.push_str("/ga");
        }
        url.push_str("?project=jdk");
        url.push_str("&image_type=");
        url.push_str(if jre { "jre" } else { "jdk" });
        url.push_str("&vendor=eclipse");
        url.push_str("&jvm_impl=hotspot");
        url.push_str("&heap_size=normal");
        url.push_str("&architecture=");
        url.push_str(match arch {
            Architecture::X86 => "x86",
            Architecture::X86_64 => "x64",
            Architecture::Arm => "arm",
            Architecture::Aarch64 => "aarch64",
            Architecture::Powerpc => "ppc",
            Architecture::Powerpc64 => "ppc64",
        });
        url.push_str("&os=");
        url.push_str(match os {
            OS::Linux => "linux",
            OS::MacOS => "mac",
            OS::Windows => "windows",
        });
        url
    }
}

#[allow(unused)]
#[derive(Debug)]
struct SelectedRelease {
    name: String,
    link: String,
    size: i64,
    checksum: String,
    image_type: String,
    openjdk_version: String,

    architecture: Architecture,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct Release {
    release_notes: Option<ReleaseNotes>,
    release_link: String,
    release_type: ReleaseType,
    source: Option<Source>,
    version_data: VersionData,
    download_count: Option<i64>,
    aqavit_results_link: Option<String>,
    release_name: String,
    updated_at: String,
    vendor: Vendor,
    id: String,
    binaries: Vec<Binary>,
    timestamp: String,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct ReleaseNotes {
    size: Option<i64>,
    name: String,
    link: String,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum ReleaseType {
    GA,
    EA,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct Source {
    size: Option<i64>,
    name: String,
    link: String,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct VersionData {
    patch: Option<i32>,
    security: Option<i32>,
    pre: Option<String>,
    openjdk_version: String,
    major: Option<i32>,
    minor: Option<i32>,
    build: Option<i32>,
    semver: String,
    adopt_build_number: Option<i32>,
    optional: Option<String>,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Vendor {
    Eclipse
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct Binary {
    package: Option<Package>,
    installer: Option<Installer>,
    os: String, // TODO this is an enum, meh?
    updated_at: String,
    c_lib: Option<String>, // TODO this is also an enum, meh?
    jvm_impl: Option<String>,
    scm_ref: Option<String>,
    project: Option<String>, // TODO this is also an enum, meh?
    heap_size: String, // TODO this is also an enum, meh?
    architecture: String, // TODO this is also an enum, meh?
    image_type: String, // TODO this is also an enum, meh?,
    download_count: Option<i64>,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct Package {
    metadata_link: Option<String>,
    size: i64,
    checksum_link: Option<String>,
    name: String,
    link: String,
    checksum: String,
    signature_link: Option<String>,
    download_count: Option<i64>,
}

#[allow(unused)]
#[derive(Debug)]
#[derive(Deserialize)]
struct Installer {
    metadata_link: Option<String>,
    size: Option<i64>,
    checksum_link: Option<String>,
    name: String,
    link: String,
    checksum: Option<String>,
    signature_link: Option<String>,
    download_count: Option<i64>,
}
