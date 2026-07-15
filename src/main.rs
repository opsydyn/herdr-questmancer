use anyhow::Result;
use clap::Parser;
use herdr_webmaster::{
    cli::{Cli, Command},
    terminal,
};

#[tokio::main]
async fn main() -> Result<()> {
    terminal::install_panic_hook();
    let cli = Cli::parse();

    match cli.command {
        Command::Ui { view } => terminal::run(view).await,
    }
}
