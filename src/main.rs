use clap::{Parser, Subcommand};
use env_logger::Env;

use crate::cli::list::ListCommand;
use crate::cli::provision::ProvisionCommand;
use crate::cli::Execute;
use std::io;

mod cli;

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
    List(ListCommand),

    /// Provision new jvms
    Provision(ProvisionCommand),
}

impl Execute for Commands {
    fn execute(self) -> io::Result<()> {
        match self {
            Commands::List(args) => args.execute(),
            Commands::Provision(args) => args.execute(),
        }
    }
}

fn main() -> io::Result<()> {
    let cli = Cli::parse();
    env_logger::Builder::from_env(Env::default().default_filter_or(format!("{},rustls=off,ureq=off,ureq_proto=off", if cli.verbose { "debug" } else { "info" }))).init();

    cli.command.execute()
}
