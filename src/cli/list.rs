use crate::cli::Execute;
use clap::Args;
use jvm_utils::locator::LocatorBuilder;
use std::io;

#[derive(Args)]
pub(crate) struct ListCommand {
    /// Enable Json output
    #[clap(short, long)]
    json: bool,

    /// Enable pretty json output, requires --json
    #[clap(short, long, requires = "json")]
    pretty: bool,

    /// Don't search known system paths
    #[clap(long)]
    without_system: bool,

    /// Don't search known Gradle paths
    #[clap(long)]
    without_intellij: bool,

    /// Don't search known Gradle paths
    #[clap(long)]
    without_gradle: bool,
}

impl Execute for ListCommand {
    fn execute(self) -> io::Result<()> {
        let mut locator = LocatorBuilder::new();
        if !self.without_system {
            locator.with_platform_locator();
        }
        if !self.without_intellij {
            locator.with_intellij_locator();
        }
        if !self.without_gradle {
            locator.with_gradle_locator();
        }

        let located = locator.locate();
        if self.json {
            if self.pretty {
                println!("{}", serde_json::to_string_pretty(&located)?)
            } else {
                println!("{}", serde_json::to_string(&located)?)
            }
        } else {
            for x in located {
                let version = x.lang_version;
                let path = x.java_home;
                println!("Found java version {version:?} at {path:?}")
            }
        }

        Ok(())
    }
}
