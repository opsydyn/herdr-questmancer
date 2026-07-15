use clap::Parser;
use herdr_webmaster::{
    app::View,
    cli::{Cli, Command},
};

#[test]
fn omits_the_initial_view_by_default() {
    let cli = Cli::try_parse_from(["herdr-webmaster", "ui"]).expect("valid CLI");

    assert_eq!(cli.command, Command::Ui { view: None });
}

#[test]
fn accepts_cafe_as_initial_view() {
    let cli = Cli::try_parse_from(["herdr-webmaster", "ui", "--view", "cafe"]).expect("valid CLI");

    assert_eq!(
        cli.command,
        Command::Ui {
            view: Some(View::Cafe)
        }
    );
}
