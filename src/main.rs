use clap::{Args, Parser, Subcommand};
use env_logger::Env;
use jvm_utils::locator::LocatorBuilder;
use std::io;

#[derive(Parser)]
#[command(version, about, long_about = None)]
#[command(propagate_version = true)]
struct Cli {
    /// Enable verbose output
    #[clap(short, long)]
    verbose: bool,

    #[command(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Lists all jvms on the system.
    List(ListArgs)
}

#[derive(Args)]
struct ListArgs {}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(if cli.verbose { "debug" } else { "info" })).init();

    match cli.command {
        Commands::List(_) => list()
    }
}

fn list() -> io::Result<()> {
    let locator = LocatorBuilder::new()
        .with_platform_locator()
        .with_intellij_locator()
        .with_gradle_locator();

    for x in locator.locate() {
        let version = x.lang_version;
        let path = x.java_home;
        println!("Found java version {version:?} at {path:?}")
    }

    Ok(())
}
