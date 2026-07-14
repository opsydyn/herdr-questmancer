use std::path::PathBuf;

use herdr_webmaster::herdr::environment::{EnvironmentError, HerdrEnvironment};

#[test]
fn requires_socket_path() {
    let result = HerdrEnvironment::from_lookup(|name| match name {
        "HERDR_BIN_PATH" => Some("/usr/local/bin/herdr".into()),
        _ => None,
    });

    assert!(matches!(
        result,
        Err(EnvironmentError::Missing("HERDR_SOCKET_PATH"))
    ));
}

#[test]
fn requires_binary_path() {
    let result = HerdrEnvironment::from_lookup(|name| match name {
        "HERDR_SOCKET_PATH" => Some("/tmp/herdr.sock".into()),
        _ => None,
    });

    assert!(matches!(
        result,
        Err(EnvironmentError::Missing("HERDR_BIN_PATH"))
    ));
}

#[test]
fn parses_required_plugin_paths() {
    let environment = HerdrEnvironment::from_lookup(|name| match name {
        "HERDR_SOCKET_PATH" => Some("/tmp/herdr.sock".into()),
        "HERDR_BIN_PATH" => Some("/opt/herdr/bin/herdr".into()),
        _ => None,
    })
    .expect("valid environment");

    assert_eq!(environment.socket_path(), PathBuf::from("/tmp/herdr.sock"));
    assert_eq!(environment.bin_path(), PathBuf::from("/opt/herdr/bin/herdr"));
}

