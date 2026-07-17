use questmancer::{
    app::{
        CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Model, Motion, OutputPreview,
        RuntimeSettings, View,
    },
    command::CommandResult,
    domain::{
        AgentKey, ChronicleEntry, ChronicleEvent, DomainState, Epithet, GuildAttention,
        GuildSummons, PaneId, Presence, Timestamp,
    },
    herdr::{
        environment::HerdrEnvironment,
        protocol::{SessionSnapshotResult, SuccessResponse},
        supervisor::ConnectionUpdate,
    },
    interaction::reduce_action,
    runtime_loop::{apply_command_result, apply_connection_update, bootstrap_model},
    ui,
    ui::input::Action,
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::{Buffer, CellWidth},
};
use std::time::Duration;

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
    let buffer = render_buffer(model, width, height);
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n")
}

fn render_buffer(model: &Model, width: u16, height: u16) -> Buffer {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal.draw(|frame| ui::render(frame, model)).unwrap();
    terminal.backend().buffer().clone()
}

#[test]
fn wide_guild_hall_renders_every_operational_region() {
    let screen = render(&live_model(), 130, 32);

    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    assert!(screen.contains("QUEST BOARD"));
    assert!(screen.contains("PARTY ROSTER"));
    assert!(screen.contains("CALLS FOR COUNSEL"));
    assert!(screen.contains("SCRYING TABLE"));
    assert!(screen.contains("SPOILS VAULT"));
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
fn connected_room_never_renders_connecting_notice() {
    let environment = HerdrEnvironment::new("/tmp/herdr.sock", "/usr/bin/herdr");
    let mut model = bootstrap_model(Model::new(View::Guild), Some(&environment));
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();

    apply_connection_update(
        &mut model,
        ConnectionUpdate::Connected(response.result.snapshot),
        Timestamp::from_millis(1_000),
    );

    let screen = render(&model, 130, 32);
    assert!(screen.contains("CONNECTED"), "{screen}");
    assert!(!screen.contains("connecting to Herdr"), "{screen}");
}

#[test]
fn working_guild_hall_uses_the_injected_clock_for_elapsed_time() {
    let model = model_with_presence(Presence::Working, GuildAttention::Clear);

    let screen = render(&model, 130, 32);

    assert!(screen.contains("working 2m"));
}

#[test]
fn long_party_labels_keep_one_row_per_visible_elapsed_entry() {
    let mut model = live_model();
    let template = model.domain().agents.values().next().unwrap().clone();
    model.domain_mut().agents.clear();
    for index in 0..6 {
        let mut adventurer = template.clone();
        adventurer.key = AgentKey::new(format!("agent-{index}"));
        adventurer.presence = Presence::Working;
        adventurer.persona.name = format!("Agent-{index} with a deliberately long guild name");
        model
            .domain_mut()
            .agents
            .insert(adventurer.key.clone(), adventurer);
    }
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-0"));
    model.set_region(questmancer::app::Region::Party);

    let screen = render(&model, 60, 10);

    assert!(
        screen.contains("Agent-2"),
        "third logical roster row wrapped out:\n{screen}"
    );
    assert!(
        screen.matches("working 2m").count() >= 3,
        "elapsed labels were clipped:\n{screen}"
    );
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
    assert!(!selected.contains("Open Chronicle"));
    assert!(!selected.contains("Inspect spoils"));

    let _ = reduce_action(&mut live, Action::InspectSpoils);
    let unavailable = render(&live, 160, 24);
    assert!(unavailable.contains("The spoils cannot be inspected here"));
    let unavailable_medium = render(&live, 80, 24);
    assert!(unavailable_medium.contains("The spoils cannot be inspected here"));

    let _ = reduce_action(&mut live, Action::AcknowledgeSummons);
    live.set_reviewr_available(true);
    let seen = render(&live, 160, 24);
    assert!(!seen.contains("Acknowledge summons"));
    assert!(seen.contains("Inspect spoils"));
}

#[test]
fn footer_navigation_and_contextual_actions_are_truthful_at_layout_boundaries() {
    let mut model = live_model();
    model.set_reviewr_available(true);

    for (current, expected, refused) in [
        ("QUEST BOARD", "[tab] Next region", "[tab] Open Chronicle"),
        ("PARTY ROSTER", "[tab] Next region", "[tab] Open Chronicle"),
        (
            "CALLS FOR COUNSEL",
            "[tab] Open Chronicle",
            "[tab] Next region",
        ),
        ("CHRONICLE", "[tab] Next region", "[tab] Open Chronicle"),
        ("ADVENTURER", "[tab] Next region", "[tab] Open Chronicle"),
    ] {
        let narrow = render(&model, 79, 24);
        assert!(narrow.contains(current), "{narrow}");
        assert!(narrow.contains(expected), "{narrow}");
        assert!(!narrow.contains(refused), "{narrow}");

        for width in [80, 119, 120] {
            let screen = render(&model, width, 24);
            assert!(!screen.contains("[tab]"), "width {width}\n{screen}");
        }

        model.cycle_region();
    }

    for width in [80, 119, 120] {
        let screen = render(&model, width, 24);
        for action in [
            "Observe",
            "Issue counsel",
            "Scry again",
            "Acknowledge summons",
            "Inspect spoils",
        ] {
            assert!(
                screen.contains(action),
                "missing {action} at width {width}\n{screen}"
            );
        }
    }
}

#[test]
fn managed_adventurer_footer_hides_every_refused_pane_action() {
    let mut model = live_model();
    model.set_managed_pane_id(Some(PaneId::new("w1:p1")));
    model.set_reviewr_available(true);

    for width in [79, 80, 119, 120] {
        let screen = render(&model, width, 24);
        for invalid in ["Observe", "Issue counsel", "Scry again", "Inspect spoils"] {
            assert!(
                !screen.contains(invalid),
                "advertised {invalid} at width {width}\n{screen}"
            );
        }
    }
}

#[test]
fn narrow_diagnostics_remain_visible_in_every_focused_region() {
    let mut model = live_model();
    let titles = [
        "QUEST BOARD",
        "PARTY ROSTER",
        "CALLS FOR COUNSEL",
        "CHRONICLE",
        "ADVENTURER",
    ];

    for title in titles {
        let _ = reduce_action(&mut model, Action::InspectSpoils);
        let unavailable = render(&model, 79, 24);
        assert!(
            unavailable.contains(title),
            "missing {title}\n{unavailable}"
        );
        assert!(
            unavailable.contains("The spoils cannot be inspected here"),
            "missing Reviewr diagnostic in {title}\n{unavailable}"
        );
        assert_eq!(
            unavailable
                .matches("The spoils cannot be inspected here")
                .count(),
            1,
            "duplicate Reviewr diagnostic in {title}\n{unavailable}"
        );

        apply_command_result(
            &mut model,
            CommandResult::Failed {
                operation: "load output",
                message: "pane vanished".to_owned(),
            },
            Timestamp::from_millis(122_000),
        );
        let failed = render(&model, 79, 24);
        assert!(failed.contains(title), "missing {title}\n{failed}");
        assert!(
            failed.contains("load output failed: pane vanished"),
            "missing command failure in {title}\n{failed}"
        );
        assert_eq!(
            failed.matches("load output failed: pane vanished").count(),
            1,
            "duplicate command failure in {title}\n{failed}"
        );

        model.cycle_region();
    }
}

#[test]
fn reconnect_banner_preserves_the_real_disconnect_cause_with_or_without_a_party() {
    for mut model in [live_model(), Model::new(View::Guild)] {
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Disconnected("socket closed by peer".to_owned()),
            Timestamp::from_millis(122_000),
        );
        apply_connection_update(
            &mut model,
            ConnectionUpdate::Reconnecting {
                attempt: 3,
                delay: Duration::from_secs(1),
            },
            Timestamp::from_millis(122_001),
        );

        let screen = render(&model, 100, 24);

        assert!(screen.contains("The scrying pool has clouded. Reconnecting"));
        assert!(screen.contains("Cause: socket closed by peer"), "{screen}");
        assert!(screen.contains("Reconnect attempt 3"), "{screen}");
    }
}

#[test]
fn ascii_guild_hall_sanitizes_all_external_text_and_border_glyphs() {
    let mut model = live_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let selected = model.selected_agent_key().unwrap().clone();
    let (workspace, pane) = {
        let agent = model.domain_mut().agents.get_mut(&selected).unwrap();
        "Cødex\u{1b}".clone_into(&mut agent.name);
        agent.persona.name = "Élowen\u{1b}Name".to_owned();
        agent.persona.epithet = Epithet::new("Keeper ☃\u{7}");
        agent.custom_status = Some("blocked ☠\u{7}".to_owned());
        (agent.workspace_id.clone(), agent.pane_id.clone())
    };
    model
        .domain_mut()
        .campaigns
        .get_mut(&workspace)
        .unwrap()
        .label = "Café ☃\u{1b}".to_owned();
    model.domain_mut().chronicle.append(ChronicleEntry::new(
        Timestamp::from_millis(121_500),
        Some(selected),
        Some(workspace),
        Some(pane.clone()),
        8,
        ChronicleEvent::CounselRequested,
        "Chronicle ✓\u{1b}",
    ));
    model.set_output_preview(Some(OutputPreview {
        pane_id: pane,
        revision: 8,
        text: "Output λ\u{1b}[31m".to_owned(),
        loading: false,
        error: None,
    }));
    model.set_action_feedback("Diagnostic ⚠\u{7}".to_owned());

    let screen = render(&model, 130, 32);

    assert!(screen.is_ascii(), "{screen:?}");
    for leaked in ["É", "ø", "☃", "☠", "✓", "λ", "⚠", "\u{1b}", "\u{7}"] {
        assert!(!screen.contains(leaked), "leaked {leaked:?}\n{screen}");
    }
    for sanitized in [
        "?lowen?Name",
        "Caf? ??",
        "Chronicle ??",
        "Output ??[31m",
        "Diagnostic ??",
    ] {
        assert!(
            screen.contains(sanitized),
            "missing {sanitized:?}\n{screen}"
        );
    }
}

#[test]
fn outbreak_sprites_use_only_unoccupied_architecture() {
    let baseline_model = live_model();
    let mut active_model = baseline_model.clone();
    active_model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let baseline = {
        let mut baseline_model = baseline_model;
        baseline_model.set_preferences(DisplayPreferences {
            character_set: CharacterSet::Ascii,
            ..DisplayPreferences::default()
        });
        render(&baseline_model, 130, 32)
    };
    let released_at = active_model.now();
    active_model.goblins_mut().release(released_at);
    let active = render(&active_model, 130, 32);

    assert!(active.contains("{g}"), "{active}");
    for (index, (before, after)) in baseline.chars().zip(active.chars()).enumerate() {
        if before.is_ascii_alphanumeric() {
            assert_eq!(before, after, "occupied text changed at character {index}");
        }
    }
}

#[test]
fn goblins_preserve_every_cell_covered_by_wide_guild_text() {
    let mut common = live_model();
    common.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Unicode,
        motion: Motion::Full,
        ..DisplayPreferences::default()
    });
    let (_, campaign) = common.domain_mut().campaigns.pop_first().unwrap();
    let mut campaign = campaign;
    campaign.label = "界  🧙  界  🧙".to_owned();
    common.domain_mut().campaigns.insert(
        questmancer::domain::WorkspaceId::new("goblin-fixture-0"),
        campaign.clone(),
    );

    let mut rare = common.clone();
    rare.domain_mut().campaigns.clear();
    rare.domain_mut().campaigns.insert(
        questmancer::domain::WorkspaceId::new("goblin-fixture-32"),
        campaign,
    );

    let mut outbreak = common.clone();
    let released_at = outbreak.now();
    outbreak.goblins_mut().release(released_at);

    let baseline = render_buffer(&common, 130, 32);
    let wide_rows = baseline
        .content
        .iter()
        .enumerate()
        .filter(|(_, cell)| cell.cell_width() > 1)
        .map(|(index, _)| u16::try_from(index / usize::from(baseline.area.width)).unwrap())
        .collect::<Vec<_>>();
    let mut protected = Vec::new();
    for y in &wide_rows {
        for x in 0..baseline.area.width {
            let cell = baseline.cell((x, *y)).unwrap();
            if cell.symbol() != " " {
                protected.extend((0..cell.cell_width()).map(|offset| (x + offset, *y)));
            }
        }
    }
    protected.sort_unstable();
    protected.dedup();
    assert!(
        baseline.content.iter().any(|cell| cell.symbol() == "界")
            && baseline.content.iter().any(|cell| cell.symbol() == "🧙"),
        "fixture must render both CJK and emoji wide graphemes"
    );
    assert!(!wide_rows.is_empty());
    assert!(!protected.is_empty());

    for (scenario, active) in [
        ("rare sighting", render_buffer(&rare, 130, 32)),
        ("outbreak", render_buffer(&outbreak, 130, 32)),
    ] {
        let changed = active
            .content
            .iter()
            .zip(&baseline.content)
            .any(|(after, before)| after != before);
        assert!(changed, "{scenario} must render goblins");
        for (x, y) in &protected {
            assert_eq!(
                active.cell((*x, *y)),
                baseline.cell((*x, *y)),
                "{scenario} changed wide grapheme cell ({x}, {y})"
            );
        }
    }
}

#[test]
fn reduced_motion_is_static_and_no_motion_has_notice_without_sprites() {
    let mut model = live_model();
    model.set_settings(RuntimeSettings {
        show_elapsed_time: false,
        ..RuntimeSettings::default()
    });
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        motion: Motion::Reduced,
        ..DisplayPreferences::default()
    });
    model.goblins_mut().release(Timestamp::from_millis(121_000));
    model.set_now(Timestamp::from_millis(121_000));
    let reduced_first = render(&model, 130, 32);
    model.set_now(Timestamp::from_millis(122_000));
    let reduced_later = render(&model, 130, 32);
    assert_eq!(reduced_first, reduced_later);
    assert!(reduced_first.contains("{g}"), "{reduced_first}");

    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        motion: Motion::None,
        ..DisplayPreferences::default()
    });
    let none = render(&model, 130, 32);
    assert!(none.contains("CREATURES DETECTED"), "{none}");
    assert!(!none.contains("{g}"), "{none}");
}

#[test]
fn full_motion_changes_at_no_more_than_four_frames_per_second() {
    let mut model = Model::new(View::Guild);
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        motion: Motion::Full,
        ..DisplayPreferences::default()
    });
    model.goblins_mut().release(Timestamp::from_millis(1_000));

    model.set_now(Timestamp::from_millis(1_000));
    let start = render(&model, 80, 24);
    model.set_now(Timestamp::from_millis(1_249));
    assert_eq!(render(&model, 80, 24), start);
    model.set_now(Timestamp::from_millis(1_250));
    assert_ne!(render(&model, 80, 24), start);
}

#[test]
fn goblins_are_ascii_ansi_and_tiny_terminal_safe() {
    let mut model = live_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
        motion: Motion::Full,
    });
    let released_at = model.now();
    model.goblins_mut().release(released_at);

    let screen = render(&model, 130, 32);
    assert!(screen.is_ascii(), "{screen:?}");
    assert!(screen.contains("{g}"), "{screen}");

    for (width, height) in [(0, 0), (1, 1), (2, 2), (3, 3), (4, 3), (8, 5)] {
        let _ = render(&model, width, height);
    }
}
