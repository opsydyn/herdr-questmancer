use questmancer::{
    app::{ConnectionState, Model, OutputPreview, RuntimeSettings, View},
    domain::{DomainState, GuildAttention, GuildSummons, PaneId, Presence, Timestamp},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    interaction::reduce_action,
    ui,
    ui::input::Action,
};
use ratatui::{Terminal, backend::TestBackend};

fn live_model() -> Model {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut model = Model::new(View::Guild);
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

fn model_with_presence(presence: Presence, attention: GuildAttention) -> Model {
    let mut model = live_model();
    let agent = model.domain_mut().agents.values_mut().next().unwrap();
    agent.presence = presence;
    agent.presence_since = Timestamp::from_millis(1_000);
    agent.attention = attention;
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
fn empty_desk_explains_how_to_put_a_site_under_construction() {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(121_000));

    let screen = render(&model, 80, 24);

    assert!(screen.contains("No agents online"));
    assert!(screen.contains("Start an agent to put a site under construction"));
}

#[test]
fn working_desk_uses_the_injected_clock_for_elapsed_time() {
    let model = model_with_presence(Presence::Working, GuildAttention::Clear);

    let screen = render(&model, 130, 32);

    assert!(screen.contains("working 2m"));
}

#[test]
fn elapsed_time_can_be_hidden_without_leaving_extra_spacing() {
    let mut model = model_with_presence(Presence::Working, GuildAttention::Clear);
    model.set_settings(RuntimeSettings {
        show_elapsed_time: false,
        ..RuntimeSettings::default()
    });

    let screen = render(&model, 130, 32);

    assert!(screen.contains("working"));
    assert!(!screen.contains("working 2m"));
}

#[test]
fn done_unseen_is_an_update_awaiting_the_webmaster_in_the_narrow_projection() {
    let model = model_with_presence(
        Presence::Done,
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(61_000)),
    );

    let screen = render(&model, 60, 18);

    assert!(screen.contains("UPDATE READY - AWAITING WEBMASTER"));
}

#[test]
fn exited_is_a_broken_link_in_the_narrow_projection() {
    let model = model_with_presence(
        Presence::Exited,
        GuildAttention::unread(
            GuildSummons::AdventurerDeparted,
            Timestamp::from_millis(61_000),
        ),
    );

    let screen = render(&model, 60, 18);

    assert!(screen.contains("BROKEN LINK"));
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
fn live_page_hides_output_cached_for_a_different_pane() {
    let mut model = live_model();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w9:p9"),
        revision: 99,
        text: "stale output from another page".into(),
        loading: false,
        error: None,
    }));

    let screen = render(&model, 60, 18);

    assert!(!screen.contains("stale output from another page"));
    assert!(screen.contains("loading selected page..."));
}

#[test]
fn live_page_hides_nested_output_when_the_selected_pane_is_webmaster() {
    let mut model = live_model();
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 7,
        text: "THE HERDR CYBERCAFE\nCAFE WALL / 56K CABLE RUN\nNESTED WEBMASTER CONTROL CENTRE"
            .into(),
        loading: false,
        error: None,
    }));

    let screen = render(&model, 60, 18);

    assert!(screen.contains("RECENT OUTPUT"));
    assert!(!screen.contains("CAFE WALL / 56K CABLE RUN"));
    assert!(!screen.contains("THE HERDR CYBERCAFE"));
    assert!(!screen.contains("NESTED WEBMASTER CONTROL CENTRE"));
}

#[test]
fn zero_and_tiny_desk_areas_are_panic_free() {
    let model = live_model();

    for (width, height) in [(0, 0), (0, 1), (1, 0), (1, 1), (2, 2), (3, 2), (3, 3)] {
        let _ = render(&model, width, height);
    }
}

#[test]
fn footer_advertises_only_actions_valid_for_the_current_context() {
    let empty = render(&Model::new(View::Guild), 160, 24);
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
