use crate::cli::Execute;
use clap::Args;
use jvm_utils::provisioning::{adoptium, InstallationManager, ProvisionRequest};
use std::io;
use std::path::PathBuf;
use jvm_utils::install::JavaVersion::Java17;

#[derive(Args)]
pub(crate) struct ProvisionCommand {
    // The directory to provision JVM's into
    #[clap(short, long, default_value = ".jvms")]
    path: PathBuf,
}

impl Execute for ProvisionCommand {
    fn execute(self) -> io::Result<()> {
        let mut install_manager = InstallationManager::new(self.path)?;
        install_manager.with_provisioner(adoptium());
        let mut request = ProvisionRequest::from_version(Java17);
        request.with_jre_only(true);
        install_manager.provide(request)?;

        Ok(())
    }
}
