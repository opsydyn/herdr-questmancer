#![cfg(feature = "scene-preview")]

use std::{fs, io, path::PathBuf};

use crossterm::event::{Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use questmancer::{runtime_loop::RuntimeExit, terminal::preview_exit_for_event};

fn key(code: KeyCode, modifiers: KeyModifiers) -> Event {
    Event::Key(KeyEvent::new_with_kind(
        code,
        modifiers,
        KeyEventKind::Press,
    ))
}

#[test]
fn preview_exits_only_for_its_small_explicit_exit_surface() {
    assert_eq!(
        preview_exit_for_event(None).unwrap(),
        Some(RuntimeExit::InputClosed)
    );
    assert_eq!(
        preview_exit_for_event(Some(Ok(key(KeyCode::Char('q'), KeyModifiers::NONE)))).unwrap(),
        Some(RuntimeExit::Quit)
    );
    assert_eq!(
        preview_exit_for_event(Some(Ok(key(KeyCode::Char('c'), KeyModifiers::CONTROL)))).unwrap(),
        Some(RuntimeExit::Quit)
    );

    for event in [
        key(KeyCode::Char('1'), KeyModifiers::NONE),
        key(KeyCode::Char('2'), KeyModifiers::NONE),
        key(KeyCode::Down, KeyModifiers::NONE),
        key(KeyCode::Enter, KeyModifiers::NONE),
        key(KeyCode::Char('r'), KeyModifiers::NONE),
        key(KeyCode::Char('/'), KeyModifiers::NONE),
        key(KeyCode::Char(' '), KeyModifiers::NONE),
        key(KeyCode::Char('q'), KeyModifiers::SHIFT),
        Event::Paste("do not send this to an agent".to_owned()),
        Event::Resize(120, 40),
    ] {
        assert_eq!(preview_exit_for_event(Some(Ok(event))).unwrap(), None);
    }

    let error = preview_exit_for_event(Some(Err(io::Error::other("input failed"))));
    assert!(error.is_err());
}

#[test]
fn preview_binary_is_developer_only_and_default_entry_stays_legacy() {
    let root = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
    let main = fs::read_to_string(root.join("src/main.rs")).unwrap();
    assert!(main.contains("terminal::run(view).await"));

    for path in [
        root.join("herdr-plugin.toml"),
        root.join("herdr/install.sh"),
        root.join("herdr/run.sh"),
        root.join("herdr/control.sh"),
    ] {
        let contents = fs::read_to_string(&path).unwrap();
        assert!(
            !contents.contains("questmancer-scene-preview"),
            "preview leaked into {}",
            path.display()
        );
    }
}
