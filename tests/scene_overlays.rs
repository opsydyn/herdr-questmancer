use std::{collections::BTreeMap, path::PathBuf};

use questmancer::{
    app::{ConnectionState, Model, OutputPreview, View},
    domain::{
        AdventurerPersona, Agent, AgentKey, Campaign, Chronicle, ChronicleEntry, ChronicleEvent,
        DomainState, GuildAttention, PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    interaction::reduce_action,
    scene::{
        SceneActorRegion, SceneFrame, pixel::PixelRect, presentation::ScenePresentation,
        stage::WorldScene,
    },
    ui::{
        input::Action,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use ratatui::{Terminal, backend::TestBackend, widgets::Paragraph};

fn model() -> Model {
    let key = AgentKey::new("codex");
    let workspace_id = WorkspaceId::new("questmancer");
    let mut agents = BTreeMap::new();
    agents.insert(
        key.clone(),
        Agent {
            key: key.clone(),
            pane_id: PaneId::new("w1:p1"),
            workspace_id: workspace_id.clone(),
            tab_id: TabId::new("w1:t1"),
            name: "codex".to_owned(),
            custom_status: None,
            presence: Presence::Working,
            presence_since: Timestamp::from_millis(1_000),
            attention: GuildAttention::Clear,
            focused: false,
            pane_revision: 1,
            persona: AdventurerPersona::for_key(PersonaKey::new("overlay-codex")),
        },
    );
    let mut campaigns = BTreeMap::new();
    campaigns.insert(
        workspace_id.clone(),
        Campaign {
            workspace_id,
            label: "Questmancer".to_owned(),
            cwd: PathBuf::from("/tmp/questmancer"),
            party: vec![key.clone()],
        },
    );
    let mut model = Model::new(View::Guild);
    model.set_connection(ConnectionState::Connected);
    model.set_now(Timestamp::from_millis(1_000));
    model.replace_domain(DomainState {
        campaigns,
        agents,
        selected_agent: Some(key),
        chronicle: Chronicle::default(),
    });
    model
}

fn render(model: &Model, width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            frame.render_widget(Paragraph::new("WORLD REMAINS"), frame.area());
            render_scene_overlays(frame, model, &ScenePresentation::from_model(model), None);
        })
        .unwrap();
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
fn counsel_uses_a_centered_parchment_without_replacing_the_world() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Counsel);
    for character in "use jsonb".chars() {
        let _ = reduce_action(&mut model, Action::TypeCharacter(character));
    }

    let screen = render(&model, 120, 36);

    assert!(screen.contains("WORLD REMAINS"));
    assert!(screen.contains("ISSUE COUNSEL"));
    assert!(screen.contains("use jsonb"));
    assert!(screen.contains("Enter send"));
    assert!(screen.contains("Esc cancel"));
}

#[test]
fn search_and_scrying_are_contextual_overlays() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Search);
    assert!(render(&model, 120, 36).contains("SEARCH THE GUILD"));

    model.dismiss_modal();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 1,
        text: "cargo test passed".to_owned(),
        loading: false,
        error: None,
    }));
    let _ = reduce_action(&mut model, Action::Refresh);
    let screen = render(&model, 120, 36);
    assert!(screen.contains("SCRYING"));
    assert!(screen.contains("cargo test passed"));
}

#[test]
fn librarian_ledger_is_the_single_responsive_help_surface() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::ToggleLedger);

    let wide = render(&model, 120, 36);
    assert!(wide.contains("LIBRARIAN'S LEDGER"));
    assert!(wide.contains("Welcome to the Guild"));
    assert!(wide.contains("Page 1 / 4"));
    assert!(wide.contains("Esc/? close"));

    let compact = render(&model, 64, 24);
    assert!(compact.contains("LIBRARIAN'S LEDGER"));
    assert!(compact.contains("Welcome to the Guild"));
}

#[test]
fn selected_adventurer_card_exposes_fantasy_and_system_identity() {
    let mut model = model();
    model.show_adventurer_card();
    model.set_now(Timestamp::from_millis(421_000));
    model
        .domain_mut()
        .agents
        .get_mut(&AgentKey::new("codex"))
        .unwrap()
        .custom_status = Some("Indexing the forgotten library".to_owned());
    let persona_name = model.selected_agent().unwrap().persona.name.clone();

    let screen = render(&model, 120, 36);

    assert!(screen.contains("ADVENTURER"));
    assert!(screen.contains(&persona_name));
    assert!(screen.contains("Agent: codex"));
    assert!(screen.contains("Campaign: Questmancer"));
    assert!(screen.contains("Working · 7m"));
    assert!(screen.contains("Indexing the forgotten library"));
}

#[test]
fn selected_adventurer_does_not_force_the_card_open() {
    let model = model();

    let screen = render(&model, 120, 36);

    assert!(!screen.contains("ADVENTURER"));
    assert!(model.selected_agent().is_some());
}

#[test]
fn every_visible_sprite_has_an_agent_state_nameplate() {
    let mut model = model();
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![SceneActorRegion {
            agent: AgentKey::new("codex"),
            bounds: PixelRect::new(10, 20, 8, 14),
        }],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen = (0..36)
        .map(|y| {
            (0..120)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(screen.contains("codex · WORKING 7m"));
}

#[test]
fn a_nameplate_never_paints_through_another_adventurer() {
    let mut model = model();
    let mut other = model.selected_agent().cloned().unwrap();
    other.key = AgentKey::new("other");
    other.name = "other".to_owned();
    model.domain_mut().agents.insert(other.key.clone(), other);
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![
            SceneActorRegion {
                agent: AgentKey::new("codex"),
                bounds: PixelRect::new(20, 20, 16, 24),
            },
            SceneActorRegion {
                agent: AgentKey::new("other"),
                bounds: PixelRect::new(20, 18, 16, 24),
            },
        ],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let row = |y| {
        (0..120)
            .map(|x| buffer.cell((x, y)).unwrap().symbol())
            .collect::<String>()
    };

    assert!(
        !row(9).contains("codex"),
        "the preferred label row belongs to the neighbouring adventurer"
    );
    assert!(
        row(22).contains("codex · WORKING 7m"),
        "the selected nameplate should use the first clear lane below the actor"
    );
}

#[test]
fn nameplate_truncation_shortens_the_name_and_keeps_the_state_word() {
    let mut model = model();
    let mut archivist = model.selected_agent().cloned().unwrap();
    archivist.key = AgentKey::new("archivist");
    archivist.name = "archive-mender-of-the-vaults".to_owned();
    model
        .domain_mut()
        .agents
        .insert(archivist.key.clone(), archivist);
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![SceneActorRegion {
            agent: AgentKey::new("archivist"),
            bounds: PixelRect::new(40, 20, 16, 24),
        }],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen = (0..36)
        .map(|y| {
            (0..120)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<Vec<_>>()
        .join("\n");

    assert!(
        screen.contains("archive-m… · WORKING"),
        "quiet nameplates must give up name characters before the state word"
    );
    assert!(
        !screen.contains("WO…"),
        "the presence badge must never be the truncated token"
    );
}

/// Two nameplates side by side used to be placed flush, because the collision
/// test is exclusive and touching rectangles do not intersect. They read as one
/// run of text: `codex - WORKING 2member-car... - WORKING`.
#[test]
fn neighbouring_nameplates_never_touch() {
    let mut model = model();
    let mut other = model.selected_agent().cloned().unwrap();
    other.key = AgentKey::new("ember");
    other.name = "ember".to_owned();
    model.domain_mut().agents.insert(other.key.clone(), other);
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors: vec![
            SceneActorRegion {
                agent: AgentKey::new("codex"),
                bounds: PixelRect::new(20, 20, 16, 24),
            },
            SceneActorRegion {
                agent: AgentKey::new("ember"),
                bounds: PixelRect::new(60, 20, 16, 24),
            },
        ],
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(120, 36);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();

    for y in 0..36 {
        let row = (0..120)
            .map(|x| buffer.cell((x, y)).unwrap().symbol())
            .collect::<String>();
        // Any two labels on one row must have blank between them.
        assert!(
            !row.contains("WORKINGember") && !row.contains("member"),
            "nameplates ran together on row {y}: {row}"
        );
    }
}

/// A crowded party must degrade to shorter nameplates rather than losing them.
/// Six adventurers in a compact Hall used to render two labels.
#[test]
fn a_crowded_party_keeps_a_nameplate_for_every_adventurer() {
    let mut model = model();
    let template = model.selected_agent().cloned().unwrap();
    let mut actors = Vec::new();
    for index in 0..6 {
        let mut agent = template.clone();
        agent.key = AgentKey::new(format!("agent-{index}"));
        agent.name = format!("adventurer-number-{index}");
        model.domain_mut().agents.insert(agent.key.clone(), agent);
        actors.push(SceneActorRegion {
            agent: AgentKey::new(format!("agent-{index}")),
            bounds: PixelRect::new(2 + index * 18, 40, 16, 24),
        });
    }
    model.set_now(Timestamp::from_millis(421_000));
    let scene = SceneFrame {
        world: WorldScene::GuildHall,
        next_frame_in: None,
        actors,
        interactables: Vec::new(),
    };
    let backend = TestBackend::new(112, 40);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| render_scene_identity_labels(frame, &model, &scene))
        .unwrap();
    let buffer = terminal.backend().buffer();
    let screen = (0..40)
        .map(|y| {
            (0..112)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
        })
        .collect::<String>();

    // Every adventurer is working, so each leaves either a full badge or,
    // where there is no room for one, its bare state glyph.
    let badges = screen.matches("WORKING").count();
    let glyphs = screen.matches('\u{bb}').count();
    assert!(
        badges + glyphs >= 6,
        "expected a nameplate for all six adventurers, found {badges} badges and {glyphs} glyphs"
    );
}

#[test]
fn command_ribbon_expires_after_three_seconds() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::Next);
    assert!(model.command_ribbon_visible());
    assert!(render(&model, 120, 36).contains("[1] Guild"));

    model.set_now(Timestamp::from_millis(4_001));
    assert!(!model.command_ribbon_visible());
    assert!(!render(&model, 120, 36).contains("[1] Guild"));
}

#[test]
fn overlays_are_safe_at_zero_and_minimum_viewports() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::ToggleLedger);
    for (width, height) in [(0, 0), (1, 1), (40, 18), (80, 24)] {
        let _ = render(&model, width, height);
    }
}

/// The Chronicle recorded seven event types, persisted them and replayed them
/// on startup, and only one — returned spoils — ever reached a human, as a
/// count in a sidebar token. This renders the record itself.
#[test]
fn the_chronicle_parchment_shows_recorded_events_over_the_world() {
    let mut model = model();
    let selected = model.selected_agent_key().cloned().unwrap();
    {
        let domain = model.domain_mut();
        for (millis, event, summary) in [
            (
                1_000,
                ChronicleEvent::AdventurerJoined,
                "codex joined the guild",
            ),
            (
                2_000,
                ChronicleEvent::CounselRequested,
                "codex requested counsel",
            ),
            (
                3_000,
                ChronicleEvent::SpoilsReturned,
                "codex returned with spoils",
            ),
        ] {
            domain.chronicle.append(ChronicleEntry::new(
                Timestamp::from_millis(millis),
                Some(selected.clone()),
                None,
                None,
                0,
                event,
                summary,
            ));
        }
    }
    model.replace_domain(model.domain().clone());
    model.set_now(Timestamp::from_millis(63_000));

    let _ = reduce_action(&mut model, Action::OpenChronicle);
    let rendered = render(&model, 100, 26);

    assert!(
        rendered.contains("CHRONICLE"),
        "the Chronicle parchment must be titled:\n{rendered}"
    );
    for summary in [
        "codex joined the guild",
        "codex requested counsel",
        "codex returned with spoils",
    ] {
        assert!(
            rendered.contains(summary),
            "the Chronicle must show {summary:?}:\n{rendered}"
        );
    }
    assert!(
        rendered.contains("WORLD REMAINS"),
        "the Chronicle is a parchment over the world, not a replacement"
    );
    assert!(
        rendered.contains("Esc close"),
        "the way out must be visible"
    );
}

/// An empty Chronicle says so rather than rendering a blank parchment.
#[test]
fn an_empty_chronicle_says_so() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::OpenChronicle);
    let rendered = render(&model, 100, 26);
    assert!(
        rendered.contains("no Chronicle yet"),
        "an empty Chronicle must explain itself:\n{rendered}"
    );
}

/// The keyring page is the binding table rendered, so a binding cannot ship
/// without appearing here.
#[test]
fn the_ledger_shows_the_whole_keyring() {
    let mut model = model();
    let _ = reduce_action(&mut model, Action::ToggleLedger);
    let _ = reduce_action(&mut model, Action::Next);
    let _ = reduce_action(&mut model, Action::Next);
    let rendered = render(&model, 130, 30);

    assert!(
        rendered.contains("Keyring"),
        "the keyring page must be reachable in the Ledger:\n{rendered}"
    );
    for keys in ["Tab", "!", "n / N", "c", "s", "wheel", "Esc", "q"] {
        assert!(
            rendered.contains(keys),
            "the keyring must show {keys:?}:\n{rendered}"
        );
    }
}

/// Scrying asks Herdr for `output_preview_lines` — eighty by default — and the
/// parchment could show about fourteen. The rest were fetched, held in memory
/// and unreachable by any key.
#[test]
fn scrying_can_be_scrolled_to_reach_output_below_the_fold() {
    let mut model = model();
    let text = (0..60)
        .map(|index| format!("output line {index}"))
        .collect::<Vec<_>>()
        .join("\n");
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 1,
        text,
        loading: false,
        error: None,
    }));
    let _ = reduce_action(&mut model, Action::Refresh);

    let top = render(&model, 100, 26);
    assert!(
        top.contains("output line 0"),
        "the top must start at the top"
    );
    assert!(
        !top.contains("output line 40"),
        "line 40 cannot already be visible, or the test proves nothing"
    );
    assert!(
        top.contains("scroll"),
        "a scrollable parchment must say it can be scrolled:\n{top}"
    );

    for _ in 0..40 {
        let _ = reduce_action(&mut model, Action::ScrollDown);
    }
    let scrolled = render(&model, 100, 26);
    assert!(
        scrolled.contains("output line 40"),
        "scrolling must reach output that was below the fold:\n{scrolled}"
    );
    assert!(!scrolled.contains("output line 0"));
}

/// Scrolling past the end must not run the offset away, or coming back costs
/// as many presses as were wasted.
#[test]
fn scrolling_stops_at_the_end_of_the_text() {
    let mut model = model();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 1,
        text: "one\ntwo\nthree".to_owned(),
        loading: false,
        error: None,
    }));
    let _ = reduce_action(&mut model, Action::Refresh);

    for _ in 0..50 {
        let _ = reduce_action(&mut model, Action::ScrollDown);
    }
    assert_eq!(
        model.reading_scroll(),
        2,
        "three lines means a last index of 2"
    );

    let _ = reduce_action(&mut model, Action::ScrollUp);
    assert_eq!(model.reading_scroll(), 1, "one press must undo one press");
}

/// Reopening a parchment starts at the top rather than wherever it was left.
#[test]
fn a_reopened_parchment_starts_at_the_top() {
    let mut model = model();
    model.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("w1:p1"),
        revision: 1,
        text: (0..30)
            .map(|i| i.to_string())
            .collect::<Vec<_>>()
            .join("\n"),
        loading: false,
        error: None,
    }));
    let _ = reduce_action(&mut model, Action::Refresh);
    for _ in 0..5 {
        let _ = reduce_action(&mut model, Action::ScrollDown);
    }
    assert_eq!(model.reading_scroll(), 5);

    let _ = reduce_action(&mut model, Action::Dismiss);
    let _ = reduce_action(&mut model, Action::Refresh);
    assert_eq!(model.reading_scroll(), 0);
}
