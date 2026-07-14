use clap::{Parser, Subcommand};

use crate::app::View;

#[derive(Clone, Debug, Eq, Parser, PartialEq)]
#[command(name = "herdr-webmaster", version, about)]
pub struct Cli {
    #[command(subcommand)]
    pub command: Command,
}

#[derive(Clone, Debug, Eq, PartialEq, Subcommand)]
pub enum Command {
    /// Open the interactive webmaster interface.
    Ui {
        /// Initial view to display.
        #[arg(long, value_enum, default_value_t = View::Desk)]
        view: View,
    },
}
