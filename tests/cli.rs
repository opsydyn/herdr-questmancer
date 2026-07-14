use clap::Parser;
use herdr_webmaster::{
    app::View,
    cli::{Cli, Command},
};

#[test]
fn defaults_ui_to_the_desk() {
    let cli = Cli::try_parse_from(["herdr-webmaster", "ui"]).expect("valid CLI");

    assert_eq!(cli.command, Command::Ui { view: View::Desk });
}

#[test]
fn accepts_cafe_as_initial_view() {
    let cli = Cli::try_parse_from(["herdr-webmaster", "ui", "--view", "cafe"])
        .expect("valid CLI");

    assert_eq!(cli.command, Command::Ui { view: View::Cafe });
}

