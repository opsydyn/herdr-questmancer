use questmancer::{
    app::{Model, View},
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

fn render(view: View, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).expect("test terminal");
    let model = Model::new(view);

    terminal
        .draw(|frame| ui::render(frame, &model))
        .expect("render succeeds");

    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).expect("cell exists").symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

#[test]
fn empty_guild_hall_is_ready_for_a_commission() {
    let screen = render(View::Guild, 80, 24);

    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    assert!(screen.contains("The hearth is warm. The guild awaits its next commission."));
}

#[test]
fn empty_cafe_is_still_an_actionable_view() {
    let screen = render(View::Delve, 80, 24);

    assert!(screen.contains("THE HERDR CYBERCAFE"));
    assert!(screen.contains("All workstations are free"));
    assert!(screen.contains("[1] desk"));
    assert!(screen.contains("[2] cafe"));
}

#[test]
fn tiny_terminal_does_not_panic() {
    let screen = render(View::Delve, 1, 1);

    assert_eq!(screen.lines().count(), 1);
}
