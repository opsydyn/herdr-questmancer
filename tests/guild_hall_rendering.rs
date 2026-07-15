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
    "Elowen Typeweaver".clone_into(
        &mut model
            .domain_mut()
            .agents
            .values_mut()
            .next()
            .unwrap()
            .persona
            .name,
    );
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
fn wide_guild_hall_renders_every_operational_region() {
    let screen = render(&live_model(), 130, 32);

    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    assert!(screen.contains("QUEST BOARD"));
    assert!(screen.contains("PARTY ROSTER"));
    assert!(screen.contains("CALLS FOR COUNSEL"));
    assert!(screen.contains("SCRYING TABLE"));
    assert!(screen.contains("SPOILS DESK"));
    assert!(screen.contains("CHRONICLE"));
    assert!(screen.contains("Elowen"));
    assert!(screen.contains("requests counsel"));
    assert!(screen.contains("blocked 2m"));
    assert!(screen.contains("which schema should I use?"));
}

#[test]
fn empty_guild_hall_is_warm_and_ready() {
    let mut model = Model::new(View::Guild);
    model.set_now(Timestamp::from_millis(121_000));

    let screen = render(&model, 80, 24);

    assert!(screen.contains("The hearth is warm. The guild awaits its next commission."));
}

#[test]
fn working_guild_hall_uses_the_injected_clock_for_elapsed_time() {
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
fn returned_spoils_are_visible_in_the_narrow_projection() {
    let model = model_with_presence(
        Presence::Done,
        GuildAttention::unread(GuildSummons::SpoilsReturned, Timestamp::from_millis(61_000)),
    );

    let mut model = model;
    model.cycle_region();
    model.cycle_region();
    let screen = render(&model, 60, 18);

    assert!(screen.contains("has returned with unopened spoils"));
}

#[test]
fn departed_adventurer_is_visible_in_the_narrow_projection() {
    let model = model_with_presence(
        Presence::Exited,
        GuildAttention::unread(
            GuildSummons::AdventurerDeparted,
            Timestamp::from_millis(61_000),
        ),
    );

    let mut model = model;
    model.cycle_region();
    model.cycle_region();
    let screen = render(&model, 60, 18);

    assert!(screen.contains("departed"));
}

#[test]
fn eighty_column_guild_hall_keeps_attention_and_selected_adventurer_visible() {
    let screen = render(&live_model(), 80, 24);

    assert!(screen.contains("PARTY ROSTER"));
    assert!(screen.contains("Elowen"));
    assert!(screen.contains("requests counsel"));
    assert!(screen.contains("Observe"));
}

#[test]
fn narrow_guild_hall_focuses_one_region_without_losing_the_selected_adventurer() {
    let mut model = live_model();
    for _ in 0..4 {
        model.cycle_region();
    }
    let screen = render(&model, 60, 18);

    assert!(screen.contains("Elowen"));
    assert!(screen.contains("blocked"));
    assert!(screen.contains("which schema"));
}

#[test]
fn reconnecting_guild_hall_preserves_data_and_pairs_voice_with_the_real_cause() {
    let mut model = live_model();
    model.set_connection(ConnectionState::Reconnecting { attempt: 3 });

    let screen = render(&model, 100, 24);

    assert!(screen.contains("The scrying pool has clouded. Reconnecting"));
    assert!(screen.contains("attempt 3"));
    assert!(screen.contains("Elowen"));
}

#[test]
fn scrying_table_hides_output_cached_for_a_different_pane() {
    let mut model = live_model();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w9:p9"),
        revision: 99,
        text: "stale output from another page".into(),
        loading: false,
        error: None,
    }));

    for _ in 0..4 {
        model.cycle_region();
    }
    let screen = render(&model, 60, 18);

    assert!(!screen.contains("stale output from another page"));
    assert!(screen.contains("The scrying pool is still."));
}

#[test]
fn scrying_table_hides_nested_output_when_the_selected_pane_is_managed() {
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

    for _ in 0..4 {
        model.cycle_region();
    }
    let screen = render(&model, 60, 18);

    assert!(screen.contains("SCRYING TABLE"));
    assert!(!screen.contains("CAFE WALL / 56K CABLE RUN"));
    assert!(!screen.contains("THE HERDR CYBERCAFE"));
    assert!(!screen.contains("NESTED WEBMASTER CONTROL CENTRE"));
}

#[test]
fn zero_and_tiny_guild_hall_areas_are_panic_free() {
    let model = live_model();

    for (width, height) in [(0, 0), (0, 1), (1, 0), (1, 1), (2, 2), (3, 2), (3, 3)] {
        let _ = render(&model, width, height);
    }
}

#[test]
fn footer_advertises_only_actions_valid_for_the_current_context() {
    let empty = render(&Model::new(View::Guild), 160, 24);
    assert!(!empty.contains("Observe"));
    assert!(!empty.contains("Issue counsel"));
    assert!(!empty.contains("Acknowledge summons"));
    assert!(!empty.contains("Inspect spoils"));

    let mut live = live_model();
    let selected = render(&live, 160, 24);
    assert!(selected.contains("Observe"));
    assert!(selected.contains("Issue counsel"));
    assert!(selected.contains("Acknowledge summons"));
    assert!(selected.contains("Open Chronicle"));
    assert!(!selected.contains("Inspect spoils"));

    let _ = reduce_action(&mut live, Action::Reviewr);
    let unavailable = render(&live, 160, 24);
    assert!(unavailable.contains("The spoils cannot be inspected here"));
    let unavailable_medium = render(&live, 80, 24);
    assert!(unavailable_medium.contains("The spoils cannot be inspected here"));

    let _ = reduce_action(&mut live, Action::MarkSeen);
    live.set_reviewr_available(true);
    let seen = render(&live, 160, 24);
    assert!(!seen.contains("Acknowledge summons"));
    assert!(seen.contains("Inspect spoils"));
}
