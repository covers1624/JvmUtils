use crate::cli::Execute;
use clap::Args;
use jvm_utils::install::JavaInstall;
use jvm_utils::locator::LocatorBuilder;
use serde::{Deserialize, Serialize};
use std::io;
use std::path::PathBuf;

#[derive(Args)]
pub(crate) struct ListCommand {
    /// Enable Json output
    #[clap(short, long)]
    json: bool,

    /// Enable pretty json output, requires --json
    #[clap(long, requires = "json")]
    pretty: bool,

    // TODO, we should sort to latest version?
    /// Only return the first result
    #[clap(short, long)]
    first: bool,

    /// Only return the paths to the java executable
    #[clap(short, long)]
    path: bool,

    /// Use java instead of javaw on Windows.
    #[clap(long, requires = "path")]
    without_javaw: bool,

    /// Don't search known system paths
    #[clap(long)]
    without_system: bool,

    /// Don't search known Gradle paths
    #[clap(long)]
    without_intellij: bool,

    /// Don't search known Gradle paths
    #[clap(long)]
    without_gradle: bool,

    /// Don't return any OpenJ9 JVM's
    #[clap(long)]
    ignore_openj9: bool,

    /// Only return JVM's which contain a compiler
    #[clap(long)]
    jdk_only: bool,
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

        if self.ignore_openj9 {
            locator.ignore_openj9();
        }

        if self.jdk_only {
            locator.jdk_only();
        }

        let mut located = locator.locate();
        if self.first && !located.is_empty() {
            located = vec![located.remove(0)];
        }

        if self.path {
            self.emit_path(located)
        } else {
            self.emit(located)
        }
    }
}

impl ListCommand {
    fn emit(self, located: Vec<JavaInstall>) -> io::Result<()> {
        if self.json {
            if self.pretty {
                println!("{}", serde_json::to_string_pretty(&located)?)
            } else {
                println!("{}", serde_json::to_string(&located)?)
            }
        } else {
            for x in located {
                println!("Found java version {:?} at {:?}", x.lang_version, x.java_home)
            }
        }

        Ok(())
    }

    fn emit_path(self, located: Vec<JavaInstall>) -> io::Result<()> {
        #[derive(Serialize, Deserialize)]
        struct Entry {
            path: PathBuf,
        }

        let located: Vec<Entry> = located.into_iter()
            .map(|e| Entry { path: JavaInstall::get_java_executable(e.java_home, self.without_javaw) })
            .collect();

        if self.json {
            if self.pretty {
                println!("{}", serde_json::to_string_pretty(&located)?)
            } else {
                println!("{}", serde_json::to_string(&located)?)
            }
        } else {
            for x in located {
                println!("{}", x.path.display())
            }
        }

        Ok(())
    }
}
