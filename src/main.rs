use anyhow::Result;
use clap::Parser;
use herdr_webmaster::{
    cli::{Cli, Command},
    runtime::RuntimeRegistration,
    terminal,
};

fn main() -> Result<()> {
    terminal::install_panic_hook();
    let cli = Cli::parse();

    match cli.command {
        Command::Ui { view } => {
            let _runtime = RuntimeRegistration::from_env(view)?;
            terminal::run(view)
        }
    }
}
