use crate::cli::Execute;
use clap::Args;
use jvm_utils::locator::LocatorBuilder;
use std::io;

#[derive(Args)]
pub(crate) struct ListCommand {
    /// Enable Json output
    #[clap(short, long)]
    json: bool,
}

impl Execute for ListCommand {
    fn execute(self) -> io::Result<()> {
        let locator = LocatorBuilder::new()
            .with_platform_locator()
            .with_intellij_locator()
            .with_gradle_locator();

        let located = locator.locate();
        if self.json {
            println!("{}", serde_json::to_string(&located)?)
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
