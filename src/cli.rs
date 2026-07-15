use clap::{Parser, Subcommand};

use crate::app::View;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
#[command(name = "questmancer", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub enum Command {
    /// Open the interactive Questmancer interface.
    Ui {
        /// Initial view to display.
        #[arg(long, value_enum)]
        view: Option<View>,
    },
}
