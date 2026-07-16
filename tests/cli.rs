use clap::Parser;
use questmancer::{
    app::View,
    cli::{Cli, Command},
};

#[test]
fn omits_the_initial_view_by_default() {
    let cli = Cli::try_parse_from(["questmancer", "ui"]).expect("valid CLI");
    assert_eq!(cli.command, Command::Ui { view: None });
}

#[test]
fn accepts_guild_and_delve_as_initial_views() {
    for (value, expected) in [("guild", View::Guild), ("delve", View::Delve)] {
        let cli = Cli::try_parse_from(["questmancer", "ui", "--view", value]).expect("valid CLI");
        assert_eq!(
            cli.command,
            Command::Ui {
                view: Some(expected)
            }
        );
    }
}

#[test]
fn rejects_removed_initial_view_aliases() {
    for value in ["desk", "cafe"] {
        assert!(Cli::try_parse_from(["questmancer", "ui", "--view", value]).is_err());
    }
}
