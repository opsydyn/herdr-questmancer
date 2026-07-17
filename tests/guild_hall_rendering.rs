use questmancer::{
    app::{
        CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Model, Motion, OutputPreview,
        RuntimeSettings, View,
    },
    command::CommandResult,
    domain::{
        AgentKey, ChronicleEntry, ChronicleEvent, DomainState, Epithet, GuildAttention,
        GuildSummons, PaneId, Presence, Timestamp, WorkspaceId,
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
    layout::Rect,
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

fn wide_room_model() -> Model {
    let mut model = live_model();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 7,
        text: "schema choice?".to_owned(),
        loading: false,
        error: None,
    }));
    let template = model.domain().agents.values().next().unwrap().clone();
    let workspace_id = template.workspace_id.clone();
    let fixtures = [
        (
            "agent-counsel",
            "Aster Counsel",
            Presence::Blocked,
            GuildAttention::Clear,
            false,
        ),
        (
            "agent-hearth",
            "Bran Hearth",
            Presence::Idle,
            GuildAttention::Clear,
            false,
        ),
        (
            "agent-spoils",
            "Cora Spoils",
            Presence::Done,
            GuildAttention::unread(
                GuildSummons::SpoilsReturned,
                Timestamp::from_millis(120_500),
            ),
            false,
        ),
        (
            "agent-token",
            "Dain Token",
            Presence::Working,
            GuildAttention::Clear,
            true,
        ),
    ];

    model.domain_mut().agents.clear();
    let mut party = Vec::new();
    for (key, name, presence, attention, focused) in fixtures {
        let mut agent = template.clone();
        agent.key = AgentKey::new(key);
        name.clone_into(&mut agent.persona.name);
        agent.presence = presence;
        agent.attention = attention;
        agent.focused = focused;
        agent.custom_status = None;
        party.push(agent.key.clone());
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-token"));
    model
        .domain_mut()
        .campaigns
        .get_mut(&workspace_id)
        .unwrap()
        .party = party;
    model
}

fn crowded_owner_model() -> Model {
    let mut model = live_model();
    let template = model.domain().agents.values().next().unwrap().clone();
    let workspace_id = template.workspace_id.clone();
    model.domain_mut().agents.clear();
    let mut party = Vec::new();

    let mut insert = |key: String, name: String, presence: Presence, attention: GuildAttention| {
        let mut agent = template.clone();
        agent.key = AgentKey::new(key.clone());
        agent.pane_id = PaneId::new(format!("pane-{key}"));
        agent.name = format!("pane-{key}");
        agent.persona.name = name;
        agent.presence = presence;
        agent.attention = attention;
        agent.focused = false;
        agent.custom_status = None;
        party.push(agent.key.clone());
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    };

    for suffix in ["A", "B", "C"] {
        insert(
            format!("counsel-{suffix}"),
            format!("CNS-{suffix}"),
            Presence::Blocked,
            GuildAttention::Clear,
        );
        insert(
            format!("hearth-{suffix}"),
            format!("HTH-{suffix}"),
            Presence::Idle,
            GuildAttention::Clear,
        );
        insert(
            format!("spoils-{suffix}"),
            format!("SPL-{suffix}"),
            Presence::Done,
            GuildAttention::unread(
                GuildSummons::SpoilsReturned,
                Timestamp::from_millis(120_500),
            ),
        );
    }
    for index in 0..17 {
        insert(
            format!("token-{index:02}"),
            format!("TKN-{index:02}"),
            Presence::Working,
            GuildAttention::Clear,
        );
    }

    let selected = AgentKey::new("token-00");
    model.domain_mut().selected_agent = Some(selected);
    model
        .domain_mut()
        .campaigns
        .get_mut(&workspace_id)
        .unwrap()
        .party = party;
    model
}

fn overflowing_owner_model(tokens: bool, total: usize) -> Model {
    let mut model = live_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let template = model.domain().agents.values().next().unwrap().clone();
    let workspace_id = template.workspace_id.clone();
    model.domain_mut().agents.clear();
    let mut party = Vec::new();

    for index in 0..total {
        let prefix = if tokens { "TOK" } else { "ADV" };
        let key = AgentKey::new(format!("overflow-{prefix}-{index:03}"));
        let mut agent = template.clone();
        agent.key = key.clone();
        agent.pane_id = PaneId::new(format!("pane-{prefix}-{index:03}"));
        agent.name = format!("pane-{prefix}-{index:03}");
        agent.persona.name = format!("{prefix}-{index:03}");
        agent.presence = if tokens {
            Presence::Working
        } else {
            Presence::Done
        };
        agent.attention = if tokens {
            GuildAttention::Clear
        } else {
            GuildAttention::unread(
                GuildSummons::SpoilsReturned,
                Timestamp::from_millis(120_500),
            )
        };
        agent.focused = tokens;
        agent.custom_status = None;
        party.push(key.clone());
        model.domain_mut().agents.insert(key, agent);
    }
    model.domain_mut().selected_agent = None;
    model
        .domain_mut()
        .campaigns
        .get_mut(&workspace_id)
        .unwrap()
        .party = party;
    model
}

fn six_table_token_overflow_model(total: usize) -> Model {
    let mut model = live_model();
    model.set_preferences(DisplayPreferences {
        character_set: CharacterSet::Ascii,
        ..DisplayPreferences::default()
    });
    let campaign_template = model.domain().campaigns.values().next().unwrap().clone();
    let agent_template = model.domain().agents.values().next().unwrap().clone();
    model.domain_mut().campaigns.clear();
    model.domain_mut().agents.clear();
    model.domain_mut().selected_agent = None;

    for (campaign_index, label) in ["ALPHA", "BRAVO", "CEDAR", "DELTA", "EMBER", "FJORD"]
        .into_iter()
        .enumerate()
    {
        let workspace_id = WorkspaceId::new(format!("campaign-{campaign_index}"));
        let mut campaign = campaign_template.clone();
        campaign.workspace_id = workspace_id.clone();
        label.clone_into(&mut campaign.label);
        campaign.party.clear();
        if campaign_index == 0 {
            for index in 0..total {
                let key = AgentKey::new(format!("narrow-token-{index:03}"));
                let mut agent = agent_template.clone();
                agent.key = key.clone();
                agent.workspace_id = workspace_id.clone();
                agent.pane_id = PaneId::new(format!("narrow-pane-{index:03}"));
                agent.name = format!("narrow-pane-{index:03}");
                agent.persona.name = format!("TOK-{index:03}");
                agent.presence = Presence::Working;
                agent.attention = GuildAttention::Clear;
                agent.focused = false;
                agent.custom_status = None;
                campaign.party.push(key.clone());
                model.domain_mut().agents.insert(key, agent);
            }
        }
        model.domain_mut().campaigns.insert(workspace_id, campaign);
    }
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

fn area_rows(buffer: &Buffer, area: Rect) -> Vec<String> {
    let inner = Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    );
    (inner.y..inner.bottom())
        .map(|y| {
            (inner.x..inner.right())
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
                .trim_end()
                .to_owned()
        })
        .collect()
}

fn row_text(buffer: &Buffer, width: u16, y: u16) -> String {
    (0..width)
        .map(|x| buffer.cell((x, y)).unwrap().symbol())
        .collect()
}

fn assert_room_border_precedes_footer(buffer: &Buffer, width: u16, height: u16) {
    let footer_y = (0..height)
        .find(|y| row_text(buffer, width, *y).contains("[1] Guild Hall"))
        .expect("footer action row");
    let border = (0..footer_y)
        .rev()
        .map(|y| row_text(buffer, width, y))
        .find(|row| row.starts_with('└') && row.ends_with('┘'));
    assert!(
        border.is_some(),
        "room bottom border was erased before footer row {footer_y}"
    );
}

fn complete_overflow_count(screen: &str, noun: &str) -> usize {
    let words = screen.split_whitespace().collect::<Vec<_>>();
    let noun_index = words
        .iter()
        .position(|word| *word == noun)
        .unwrap_or_else(|| panic!("missing complete overflow noun {noun}:\n{screen}"));
    words
        .get(noun_index.saturating_sub(1))
        .and_then(|count| count.strip_prefix('+'))
        .and_then(|count| count.parse().ok())
        .unwrap_or_else(|| panic!("missing complete overflow count before {noun}:\n{screen}"))
}

fn complete_overflow_in_rows(rows: &[String], nouns: &[&str]) -> (usize, String) {
    for (index, row) in rows.iter().enumerate() {
        let row = row.trim();
        for noun in nouns {
            if row == *noun
                && let Some(count) = index
                    .checked_sub(1)
                    .and_then(|previous| rows[previous].trim().strip_prefix('+'))
                    .and_then(|count| count.parse().ok())
            {
                return (count, (*noun).to_owned());
            }
            if let Some(count) = row
                .strip_suffix(noun)
                .map(str::trim_end)
                .and_then(|prefix| prefix.strip_prefix('+'))
                .and_then(|count| count.parse().ok())
            {
                return (count, (*noun).to_owned());
            }
        }
    }
    panic!("missing complete overflow count and noun {nouns:?}: {rows:?}");
}

#[test]
fn wide_guild_hall_renders_every_operational_region() {
    let screen = render(&live_model(), 130, 32);

    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    for landmark in [
        "GUILD DOOR",
        "QUEST WALL",
        "CAMPAIGN TABLE: webmaster",
        "COUNSEL BELL",
        "HEARTH",
        "CHRONICLE LECTERN",
        "SCRYING ALCOVE",
        "SPOILS DESK",
    ] {
        assert!(screen.contains(landmark), "missing {landmark}:\n{screen}");
    }
    assert!(screen.contains("Elowen"));
    assert!(screen.contains("requests counsel"));
    assert!(screen.contains("blocked 2m"));
    assert!(screen.contains("which schema should I"));
    assert!(screen.contains("use?"));
}

#[test]
fn wide_guild_is_one_great_room() {
    let model = wide_room_model();

    for (width, height) in [(120, 40), (160, 50)] {
        let screen = render(&model, width, height);

        for landmark in [
            "GUILD DOOR",
            "QUEST WALL",
            "COUNSEL BELL",
            "HEARTH",
            "CHRONICLE LECTERN",
            "SCRYING ALCOVE",
            "SPOILS DESK",
        ] {
            assert!(
                screen.contains(landmark),
                "missing {landmark} at {width}x{height}:\n{screen}"
            );
        }
        assert!(screen.contains("CAMPAIGN BANNER: webmaster"), "{screen}");
        assert!(screen.contains("CAMPAIGN TABLE: webmaster"), "{screen}");
        assert!(screen.contains("SELECTED LAMP"), "{screen}");
        assert!(screen.contains("schema choice?"), "{screen}");
        for name in ["Aster Counsel", "Bran Hearth", "Cora Spoils", "Dain Token"] {
            assert_eq!(
                screen.matches(name).count(),
                1,
                "{name} did not have one visible representation at {width}x{height}:\n{screen}"
            );
        }
        for old_panel in [
            "QUEST BOARD",
            "PARTY ROSTER",
            "CALLS FOR COUNSEL",
            "SCRYING TABLE",
            "SPOILS VAULT",
        ] {
            assert!(
                !screen.contains(old_panel),
                "old panel {old_panel} survived at {width}x{height}:\n{screen}"
            );
        }
    }
}

#[test]
fn crowded_wide_room_preserves_every_final_representation_and_bottom_architecture() {
    let model = crowded_owner_model();
    let buffer = render_buffer(&model, 120, 40);
    let screen = render(&model, 120, 40);

    for suffix in ["A", "B", "C"] {
        for prefix in ["CNS", "HTH", "SPL"] {
            let name = format!("{prefix}-{suffix}");
            assert_eq!(
                screen.matches(&name).count(),
                1,
                "{name} lost or duplicated in final frame:\n{screen}"
            );
        }
    }
    for index in 0..17 {
        let name = format!("TKN-{index:02}");
        assert_eq!(
            screen.matches(&name).count(),
            1,
            "{name} lost or duplicated in final frame:\n{screen}"
        );
    }
    assert!(screen.contains("└────────┘"), "{screen}");
    assert_room_border_precedes_footer(&buffer, 120, 40);
}

#[test]
fn spoils_action_copy_requires_an_actionable_selected_pane() {
    let spoils_rows = |model: &Model| {
        let projection = ui::render_projection_for(model, Rect::new(0, 0, 120, 40));
        let area = projection
            .guild_room
            .as_ref()
            .unwrap()
            .landmarks
            .iter()
            .find(|landmark| landmark.landmark == ui::guild_room_projection::GuildLandmark::Spoils)
            .unwrap()
            .area;
        area_rows(&render_buffer(model, 120, 40), area)
    };

    let mut missing = live_model();
    missing.set_reviewr_available(true);
    missing.domain_mut().selected_agent = None;
    let missing_rows = spoils_rows(&missing);
    assert!(!missing_rows.iter().any(|row| row.contains("Reviewr ready")));
    assert!(
        !missing_rows
            .iter()
            .any(|row| row.contains("Inspect spoils"))
    );

    let mut managed = live_model();
    managed.set_reviewr_available(true);
    managed.set_managed_pane_id(Some(PaneId::new("w1:p1")));
    let managed_rows = spoils_rows(&managed);
    assert!(!managed_rows.iter().any(|row| row.contains("Reviewr ready")));
    assert!(
        !managed_rows
            .iter()
            .any(|row| row.contains("Inspect spoils"))
    );

    let mut actionable = live_model();
    actionable.set_reviewr_available(true);
    let actionable_rows = spoils_rows(&actionable);
    assert!(
        actionable_rows
            .iter()
            .any(|row| row.trim() == "REVIEWR READY"),
        "{actionable_rows:?}"
    );
    assert!(
        actionable_rows
            .iter()
            .any(|row| row.trim() == "[v] Inspect spoils"),
        "{actionable_rows:?}"
    );
}

#[test]
fn six_minimum_wide_campaign_tables_keep_distinct_labels_and_seals() {
    let mut model = live_model();
    let template = model.domain().campaigns.values().next().unwrap().clone();
    model.domain_mut().campaigns.clear();
    model.domain_mut().agents.clear();
    model.domain_mut().selected_agent = None;
    for (index, label) in ["ALPHA", "BRAVO", "CEDAR", "DELTA", "EMBER", "FJORD"]
        .into_iter()
        .enumerate()
    {
        let workspace_id = WorkspaceId::new(format!("campaign-{index}"));
        let mut campaign = template.clone();
        campaign.workspace_id = workspace_id.clone();
        campaign.label = label.to_owned();
        campaign.party.clear();
        model.domain_mut().campaigns.insert(workspace_id, campaign);
    }

    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 120, 40));
    let room = projection.guild_room.as_ref().unwrap();
    let buffer = render_buffer(&model, 120, 40);
    assert_eq!(room.campaigns.len(), 6);
    for campaign in &room.campaigns {
        let rows = area_rows(&buffer, campaign.area);
        assert!(
            rows.iter().any(|row| row.trim() == campaign.label),
            "missing distinct table label {} in {rows:?}",
            campaign.label
        );
        let seal = format!("#{:04X}", campaign.seal & 0xFFFF);
        assert!(
            rows.iter().any(|row| row.trim() == seal),
            "missing table seal {seal} for {} in {rows:?}",
            campaign.label
        );
    }
}

#[test]
fn long_wide_footer_diagnostics_wrap_without_erasing_the_room() {
    let message = "Persistence failed while writing /very/long/questmancer/state/history/for/the/current/guild/session.json because the destination directory is read only; the previous valid state remains intact and no Chronicle entry was discarded.";

    for persistence in [false, true] {
        let mut model = wide_room_model();
        if persistence {
            model.set_persistence_diagnostic(message.to_owned());
        } else {
            model.set_action_feedback(message.to_owned());
        }
        let buffer = render_buffer(&model, 120, 40);
        let screen = render(&model, 120, 40);
        let normalized = screen.split_whitespace().collect::<Vec<_>>().join(" ");
        assert!(normalized.contains(message), "{screen}");
        assert!(screen.contains("HEARTH"), "{screen}");
        assert!(screen.contains("└────────┘"), "{screen}");
        assert_room_border_precedes_footer(&buffer, 120, 40);
    }
}

#[test]
fn minimum_wide_scrying_uses_a_complete_short_furniture_caption() {
    let model = live_model();
    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 120, 40));
    let area = projection
        .guild_room
        .as_ref()
        .unwrap()
        .landmarks
        .iter()
        .find(|landmark| landmark.landmark == ui::guild_room_projection::GuildLandmark::Scrying)
        .unwrap()
        .area;
    let rows = area_rows(&render_buffer(&model, 120, 40), area);

    assert!(
        rows.iter().any(|row| row.trim() == "MIRROR / CANDLES"),
        "{rows:?}"
    );
    assert!(!rows.iter().any(|row| row.ends_with("BOO")), "{rows:?}");
}

#[test]
fn spoils_diagnostic_and_single_returnee_do_not_overwrite_each_other_or_the_room_wall() {
    let mut model = model_with_presence(
        Presence::Done,
        GuildAttention::unread(
            GuildSummons::SpoilsReturned,
            Timestamp::from_millis(120_500),
        ),
    );
    let _ = reduce_action(&mut model, Action::InspectSpoils);

    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 120, 40));
    let spoils = projection
        .guild_room
        .as_ref()
        .unwrap()
        .landmarks
        .iter()
        .find(|landmark| landmark.landmark == ui::guild_room_projection::GuildLandmark::Spoils)
        .unwrap()
        .area;
    let buffer = render_buffer(&model, 120, 40);
    let screen = render(&model, 120, 40);
    let rows = area_rows(&buffer, spoils);
    let normalized = rows
        .join(" ")
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ");
    let diagnostic = "The spoils cannot be inspected here: Reviewr is unavailable.";

    assert_eq!(screen.matches("Elowen Typeweaver").count(), 1, "{screen}");
    assert_eq!(screen.matches("completed 2m").count(), 1, "{screen}");
    assert!(normalized.contains(diagnostic), "{rows:?}");
    assert!(
        rows.iter()
            .filter(|row| row.contains("Elowen Typeweaver") || row.contains("completed 2m"))
            .all(|row| !row.contains("Reviewr") && !row.contains("spoils cannot")),
        "identity/status was mixed with diagnostic copy: {rows:?}"
    );
    assert_room_border_precedes_footer(&buffer, 120, 40);
}

#[test]
fn true_token_and_adventurer_overflow_use_complete_owner_wide_counts() {
    const TOTAL: usize = 240;
    for (tokens, noun) in [(true, "TOKENS"), (false, "ADVENTURERS")] {
        let model = overflowing_owner_model(tokens, TOTAL);
        let screen = render(&model, 120, 40);
        let overflow = complete_overflow_count(&screen, noun);
        let individually_visible = if tokens {
            screen.matches("TOK-").count()
        } else {
            screen.matches("ADV").count().saturating_sub(1)
        };

        assert!(overflow > 0, "fixture did not overflow {noun}:\n{screen}");
        assert_eq!(
            individually_visible + overflow,
            TOTAL,
            "visible plus explicit {noun} overflow must describe every owner: {screen}"
        );
    }
}

#[test]
fn reviewr_discovery_race_clears_stale_copy_and_preserves_the_spoils_returnee() {
    let mut model = model_with_presence(
        Presence::Done,
        GuildAttention::unread(
            GuildSummons::SpoilsReturned,
            Timestamp::from_millis(120_500),
        ),
    );
    let _ = reduce_action(&mut model, Action::InspectSpoils);
    assert!(matches!(
        model.notice(),
        Some(questmancer::app::Notice::IntegrationDiagnostic(_))
    ));

    apply_command_result(
        &mut model,
        CommandResult::ReviewrAvailable(true),
        Timestamp::from_millis(122_000),
    );
    let screen = render(&model, 120, 40);

    assert!(screen.contains("REVIEWR READY"), "{screen}");
    assert!(screen.contains("[v] Inspect spoils"), "{screen}");
    assert_eq!(screen.matches("Elowen Typeweaver").count(), 1, "{screen}");
    assert_eq!(screen.matches("completed 2m").count(), 1, "{screen}");
    assert!(!screen.contains("Reviewr is unavailable"), "{screen}");
}

#[test]
fn spoils_copy_budget_never_sacrifices_all_returnee_identity_rows() {
    let mut model = model_with_presence(
        Presence::Done,
        GuildAttention::unread(
            GuildSummons::SpoilsReturned,
            Timestamp::from_millis(120_500),
        ),
    );
    model.set_reviewr_available(true);
    model.set_integration_diagnostic(
        "Reviewr accepted the request but its selected provider returned a detailed actionable integration warning for this pane.".to_owned(),
    );

    let screen = render(&model, 120, 40);

    assert!(screen.contains("REVIEWR READY"), "{screen}");
    assert!(screen.contains("[v] Inspect spoils"), "{screen}");
    assert_eq!(screen.matches("Elowen Typeweaver").count(), 1, "{screen}");
    assert_eq!(screen.matches("completed 2m").count(), 1, "{screen}");
}

#[test]
fn six_narrow_tables_use_a_complete_multirow_token_overflow_summary() {
    const TOTAL: usize = 240;
    let model = six_table_token_overflow_model(TOTAL);
    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 120, 40));
    let table = projection
        .guild_room
        .as_ref()
        .unwrap()
        .campaigns
        .iter()
        .find(|campaign| campaign.workspace_id == WorkspaceId::new("campaign-0"))
        .unwrap();
    assert_eq!(table.area.width.saturating_sub(2), 10);
    let rows = area_rows(&render_buffer(&model, 120, 40), table.area);
    let (overflow, noun) = complete_overflow_in_rows(&rows, &["TOKENS"]);
    let individually_visible = rows.join("\n").matches("TOK-").count();

    assert_eq!(noun, "TOKENS");
    assert_eq!(individually_visible + overflow, TOTAL, "{rows:?}");
}

#[test]
fn narrow_physical_overflow_keeps_a_complete_semantic_noun_and_count() {
    const TOTAL: usize = 240;
    let model = overflowing_owner_model(false, TOTAL);
    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 120, 40));
    let spoils = projection
        .guild_room
        .as_ref()
        .unwrap()
        .landmarks
        .iter()
        .find(|landmark| landmark.landmark == ui::guild_room_projection::GuildLandmark::Spoils)
        .unwrap();
    let rows = area_rows(&render_buffer(&model, 120, 40), spoils.area);
    let (overflow, noun) = complete_overflow_in_rows(&rows, &["ADVENTURERS", "AGENTS"]);
    let noun_prefix = usize::from(noun.starts_with("ADV"));
    let individually_visible = rows
        .join("\n")
        .matches("ADV")
        .count()
        .saturating_sub(noun_prefix);

    assert!(matches!(noun.as_str(), "ADVENTURERS" | "AGENTS"));
    assert_eq!(individually_visible + overflow, TOTAL, "{rows:?}");
}

#[test]
fn scrying_caption_uses_inner_width_at_the_twenty_five_cell_boundary() {
    let model = live_model();
    let projection = ui::render_projection_for(&model, Rect::new(0, 0, 125, 40));
    let area = projection
        .guild_room
        .as_ref()
        .unwrap()
        .landmarks
        .iter()
        .find(|landmark| landmark.landmark == ui::guild_room_projection::GuildLandmark::Scrying)
        .unwrap()
        .area;
    assert_eq!(area.width, 25, "boundary fixture drifted: {area:?}");
    let rows = area_rows(&render_buffer(&model, 125, 40), area);

    assert!(
        rows.iter().any(|row| row.trim() == "MIRROR / CANDLES"),
        "{rows:?}"
    );
    assert!(!rows.iter().any(|row| row.contains("BOOK")), "{rows:?}");
}

#[test]
fn empty_wide_guild_still_renders_a_furnished_room() {
    let screen = render(&Model::new(View::Guild), 160, 40);

    for fixture in [
        "QUEST WALL",
        "MAPS / COMMISSIONS",
        "HEARTH",
        "MUGS / BEDROLLS",
        "The hearth is warm. The guild awaits its next commission.",
    ] {
        assert!(screen.contains(fixture), "missing {fixture}:\n{screen}");
    }
}

#[test]
fn unavailable_reviewr_leaves_a_quiet_furnished_spoils_desk() {
    let screen = render(&live_model(), 160, 40);

    assert!(screen.contains("SPOILS DESK"), "{screen}");
    assert!(screen.contains("LEDGER / LOCKBOX / MUG"), "{screen}");
    assert!(!screen.contains("Reviewr is unavailable."), "{screen}");
    assert!(!screen.contains("SPOILS VAULT"), "{screen}");
}

#[test]
fn failed_output_clouds_only_the_furnished_scrying_alcove() {
    let mut model = live_model();
    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            message: "pane vanished".to_owned(),
        },
        Timestamp::from_millis(122_000),
    );

    let screen = render(&model, 160, 40);
    assert!(screen.contains("SCRYING ALCOVE"), "{screen}");
    assert!(screen.contains("MIRROR / CANDLES / BOOKS"), "{screen}");
    assert!(screen.contains("The scrying pool has clouded."), "{screen}");
    assert!(screen.contains("load output failed:"), "{screen}");
    assert!(screen.contains("pane vanished"), "{screen}");
    assert!(!screen.contains("SCRYING TABLE"), "{screen}");
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
fn offline_connection_diagnostics_render_in_connection_theatre() {
    let startup = bootstrap_model(Model::new(View::Guild), None);
    let startup_screen = render(&startup, 130, 32);
    for fragment in [
        "OFFLINE / door closed",
        "Cause: offline: launch",
        "from Herdr to connect to",
        "the live session",
    ] {
        assert!(
            startup_screen.contains(fragment),
            "missing {fragment}:\n{startup_screen}"
        );
    }

    let mut disconnected = live_model();
    apply_connection_update(
        &mut disconnected,
        ConnectionUpdate::Disconnected("socket closed by peer".to_owned()),
        Timestamp::from_millis(122_000),
    );
    let disconnected_screen = render(&disconnected, 130, 32);
    assert!(
        disconnected_screen.contains("Cause: socket"),
        "{disconnected_screen}"
    );
    assert!(
        disconnected_screen.contains("socket closed by"),
        "{disconnected_screen}"
    );
    assert!(
        disconnected_screen.contains("peer"),
        "{disconnected_screen}"
    );
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

        model.cycle_region();
    }
}

#[test]
fn load_output_failure_is_visible_only_at_the_scrying_table() {
    let mut model = live_model();
    apply_command_result(
        &mut model,
        CommandResult::OutputFailed {
            pane_id: PaneId::new("w1:p1"),
            message: "pane vanished".to_owned(),
        },
        Timestamp::from_millis(122_000),
    );

    for title in [
        "QUEST BOARD",
        "PARTY ROSTER",
        "CALLS FOR COUNSEL",
        "CHRONICLE",
    ] {
        let screen = render(&model, 79, 24);
        assert!(screen.contains(title), "missing {title}\n{screen}");
        assert!(
            !screen.contains("load output failed: pane vanished"),
            "output failure leaked into {title}\n{screen}"
        );
        model.cycle_region();
    }

    let scrying = render(&model, 79, 24);
    assert!(scrying.contains("SCRYING TABLE"), "{scrying}");
    assert!(
        scrying.contains("load output failed: pane vanished"),
        "{scrying}"
    );
}

#[test]
fn wide_room_renders_reviewr_diagnostic_once_at_the_spoils_vault() {
    let mut model = live_model();
    let _ = reduce_action(&mut model, Action::InspectSpoils);

    let screen = render(&model, 130, 32);
    assert!(
        screen.contains("The spoils cannot be inspected"),
        "{screen}"
    );
    assert_eq!(
        screen.matches("Reviewr is unavailable.").count(),
        1,
        "{screen}"
    );
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
