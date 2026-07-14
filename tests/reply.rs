use herdr_webmaster::{
    app::{Model, View},
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn reply_modal_renders_draft_and_contextual_keys() {
    let mut model = Model::new(View::Desk);
    model.open_reply();
    for character in "use jsonb".chars() {
        model.push_reply_character(character);
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
    assert!(screen.contains("SHOUT OVER"));
    assert!(screen.contains("use jsonb"));
    assert!(screen.contains("[enter] send"));
    assert!(screen.contains("[esc] cancel"));
}
