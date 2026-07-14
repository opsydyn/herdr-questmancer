use herdr_webmaster::{
    app::{ConnectionState, Model, OutputPreview, View},
    domain::{DomainState, PaneId, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    interaction::reduce_action,
    ui,
    ui::input::Action,
};
use ratatui::{Terminal, backend::TestBackend};

fn live_model() -> Model {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut model = Model::new(View::Desk);
    model.replace_domain(DomainState::from_snapshot(
        &response.result.snapshot,
        Timestamp::from_millis(1_000),
    ));
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(121_000));
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 7,
        text: "which schema should I use?".into(),
        loading: false,
        error: None,
    }));
    model
}

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
fn wide_desk_renders_sites_mail_and_live_agent_details() {
    let screen = render(&live_model(), 130, 32);

    assert!(screen.contains("YOUR SITES"));
    assert!(screen.contains("WEBMASTER MAIL"));
    assert!(screen.contains("LIVE PAGE"));
    assert!(screen.contains("webmaster"));
    assert!(screen.contains("Codex"));
    assert!(screen.contains("NEEDS WEBMASTER"));
    assert!(screen.contains("blocked 2m"));
    assert!(screen.contains("which schema should I use?"));
}

#[test]
fn eighty_column_desk_keeps_attention_and_selected_agent_visible() {
    let screen = render(&live_model(), 80, 24);

    assert!(screen.contains("WEBMASTER MAIL"));
    assert!(screen.contains("Codex"));
    assert!(screen.contains("NEEDS WEBMASTER"));
    assert!(screen.contains("[enter] visit"));
}

#[test]
fn narrow_desk_falls_back_without_losing_the_selected_agent() {
    let screen = render(&live_model(), 60, 18);

    assert!(screen.contains("Codex"));
    assert!(screen.contains("blocked"));
    assert!(screen.contains("which schema"));
}

#[test]
fn disconnected_desk_preserves_data_and_shows_connection_state() {
    let mut model = live_model();
    model.set_connection(ConnectionState::Reconnecting { attempt: 3 });

    let screen = render(&model, 100, 24);

    assert!(screen.contains("reconnecting #3"));
    assert!(screen.contains("Codex"));
}

#[test]
fn footer_advertises_only_actions_valid_for_the_current_context() {
    let empty = render(&Model::new(View::Desk), 160, 24);
    assert!(!empty.contains("[enter] visit"));
    assert!(!empty.contains("[r] reply"));
    assert!(!empty.contains("[o] output"));
    assert!(!empty.contains("[space] seen"));
    assert!(!empty.contains("[/] search"));
    assert!(!empty.contains("[v] reviewr"));

    let mut live = live_model();
    let selected = render(&live, 160, 24);
    assert!(selected.contains("[/] search"));
    assert!(selected.contains("[enter] visit"));
    assert!(selected.contains("[r] reply"));
    assert!(selected.contains("[o] output"));
    assert!(selected.contains("[space] seen"));
    assert!(!selected.contains("[v] reviewr"));

    let _ = reduce_action(&mut live, Action::MarkSeen);
    live.set_reviewr_available(true);
    let seen = render(&live, 160, 24);
    assert!(!seen.contains("[space] seen"));
    assert!(seen.contains("[v] reviewr"));
}
