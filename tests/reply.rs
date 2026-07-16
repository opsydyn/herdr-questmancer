use questmancer::{
    app::{CharacterSet, DisplayPreferences, Model, View},
    interaction::reduce_action,
    ui,
    ui::input::Action,
};
use ratatui::{Terminal, backend::TestBackend};

fn render(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::render(frame, model)).unwrap();
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn counsel_modal_renders_draft_and_contextual_keys() {
    let mut model = Model::new(View::Guild);
    model.open_counsel();
    for character in "use jsonb".chars() {
        model.push_counsel_character(character);
    }
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &model)).unwrap();

    let buffer = terminal.backend().buffer();
    let screen = (0..24)
        .map(|y| {
            (0..80)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("ISSUE COUNSEL"));
    assert!(screen.contains("use jsonb"));
    assert!(screen.contains("[enter] send"));
    assert!(screen.contains("[esc] cancel"));
}

#[test]
fn search_modal_renders_query_status_and_contextual_keys() {
    let mut model = Model::new(View::Guild);
    let _ = reduce_action(&mut model, Action::Search);
    for character in "missing".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Submit);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &model)).unwrap();

    let buffer = terminal.backend().buffer();
    let screen = (0..24)
        .map(|y| {
            (0..80)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.contains("SEARCH ADVENTURERS"));
    assert!(screen.contains("missing"));
    assert!(screen.contains("No adventurer or campaign answers \"missing\"."));
    assert!(screen.contains("[enter] find"));
    assert!(screen.contains("[esc] cancel"));
}

#[test]
fn search_modal_sanitizes_query_and_status_in_ascii_mode() {
    let mut model = Model::new(View::Guild);
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let _ = reduce_action(&mut model, Action::Search);
    for character in "café\u{1b}".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }
    let _ = reduce_action(&mut model, Action::Submit);
    let backend = TestBackend::new(80, 24);
    let mut terminal = Terminal::new(backend).unwrap();

    terminal.draw(|frame| ui::render(frame, &model)).unwrap();

    let buffer = terminal.backend().buffer();
    let screen = (0..24)
        .map(|y| {
            (0..80)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");
    assert!(screen.is_ascii(), "{screen:?}");
    assert!(screen.contains("SEARCH ADVENTURERS"));
    assert!(screen.contains("caf??"), "{screen}");
}

#[test]
fn help_modal_renders_current_questmancer_controls_in_both_views() {
    for view in [View::Guild, View::Delve] {
        let mut model = Model::new(view);
        let _ = reduce_action(&mut model, Action::ShowHelp);

        let screen = render(&model, 80, 24);

        assert!(
            screen.contains("QUESTMANCER'S FIELD GUIDE"),
            "{view:?}\n{screen}"
        );
        for label in [
            "Observe",
            "Issue Counsel",
            "Acknowledge Summons",
            "Inspect Spoils",
            "[esc/?] Close guide",
        ] {
            assert!(
                screen.contains(label),
                "{view:?} missing {label:?}\n{screen}"
            );
        }
        for stale in ["visit", "seen", "reviewr", "reply"] {
            assert!(
                !screen.to_lowercase().contains(stale),
                "{view:?} leaked {stale:?}\n{screen}"
            );
        }
    }
}

#[test]
fn help_modal_is_ascii_and_tiny_terminal_safe() {
    let mut model = Model::new(View::Guild);
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let _ = reduce_action(&mut model, Action::ShowHelp);

    let ascii = render(&model, 64, 18);
    assert!(ascii.is_ascii(), "{ascii:?}");
    assert!(ascii.contains("QUESTMANCER'S FIELD GUIDE"), "{ascii}");

    for (width, height) in [(0, 0), (1, 1), (2, 1), (3, 2), (4, 3)] {
        let screen = render(&model, width, height);
        assert!(screen.is_ascii(), "{width}x{height}: {screen:?}");
    }
}
