use anyhow::Result;
use clap::Parser;
use herdr_webmaster::{
    cli::{Cli, Command},
    runtime::RuntimeRegistration,
    terminal,
};

#[tokio::main]
async fn main() -> Result<()> {
    terminal::install_panic_hook();
    let cli = Cli::parse();

    match cli.command {
        Command::Ui { view } => {
            let initial_view = view.unwrap_or_default();
            let _runtime = RuntimeRegistration::from_env(initial_view)?;
            terminal::run(initial_view).await
        }
    }
}
