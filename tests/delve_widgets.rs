use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion},
    domain::{Agent, DomainState, Keepsake, PaneId, Presence, Timestamp, WorkspaceId},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::{
        persona::compose_profile_adventurer_for_palette,
        pixel::{ColorRole, Palette, pack},
        theatre::{TheatreFrame, TheatrePose},
        widgets::{render_adventurer_card, render_chamber},
    },
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect, style::Color};

fn agent() -> Agent {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(0))
        .agents
        .into_values()
        .next()
        .unwrap()
}

fn preferences(character_set: CharacterSet) -> DisplayPreferences {
    DisplayPreferences {
        motion: Motion::Full,
        character_set,
        color_mode: ColorMode::Xterm256,
    }
}

fn render_chamber_colours(preferences: DisplayPreferences) -> Vec<Color> {
    let agent = agent();
    let backend = TestBackend::new(28, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_chamber(
                frame,
                Rect::new(0, 0, 28, 10),
                &agent,
                theatre(TheatrePose::Delving, 2, false, "DELVING"),
                false,
                &preferences,
            );
        })
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| cell.fg)
        .collect()
}

fn render_chamber_styles(
    agent: &Agent,
    theatre: TheatreFrame,
    preferences: DisplayPreferences,
) -> Vec<(Color, Color)> {
    let backend = TestBackend::new(28, 10);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_chamber(
                frame,
                Rect::new(0, 0, 28, 10),
                agent,
                theatre,
                false,
                &preferences,
            );
        })
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(|cell| (cell.fg, cell.bg))
        .collect()
}

fn theatre(
    pose: TheatrePose,
    animation_frame: u8,
    focused: bool,
    label: &'static str,
) -> TheatreFrame {
    TheatreFrame {
        pose,
        animation_frame,
        focused,
        label,
    }
}

fn render_chamber_at(
    agent: &Agent,
    theatre: TheatreFrame,
    selected: bool,
    preferences: DisplayPreferences,
    width: u16,
    height: u16,
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_chamber(
                frame,
                Rect::new(0, 0, width, height),
                agent,
                theatre,
                selected,
                &preferences,
            );
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

fn render_profile_at(
    agent: &Agent,
    theatre: TheatreFrame,
    preferences: DisplayPreferences,
    width: u16,
    height: u16,
) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            render_adventurer_card(
                frame,
                Rect::new(0, 0, width, height),
                agent,
                theatre,
                &preferences,
            );
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
fn full_chamber_communicates_every_pose_without_relying_on_colour() {
    let preferences = preferences(CharacterSet::Unicode);
    let mut agent = agent();
    agent.focused = false;
    let cases = [
        (
            Presence::Working,
            theatre(TheatrePose::Delving, 2, false, "DELVING"),
            "[>]",
            "DELVING",
            "TOOL/RUNE",
        ),
        (
            Presence::Blocked,
            theatre(TheatrePose::SeekingCounsel, 1, false, "COUNSEL REQUESTED"),
            "[!]",
            "COUNSEL REQUESTED",
            "SIGNAL LANTERN",
        ),
        (
            Presence::Done,
            theatre(TheatrePose::SpoilsUnopened, 4, false, "SPOILS RETURNED"),
            "[+]",
            "SPOILS RETURNED",
            "CHEST SPARKLE",
        ),
        (
            Presence::Done,
            theatre(TheatrePose::VictoryRecorded, 0, false, "VICTORY RECORDED"),
            "[+]",
            "VICTORY RECORDED",
            "VICTORY LEDGER",
        ),
        (
            Presence::Idle,
            theatre(TheatrePose::Resting, 3, false, "RESTING"),
            "[~]",
            "RESTING",
            "CAMPFIRE",
        ),
        (
            Presence::Exited,
            theatre(TheatrePose::Departed, 0, false, "DEPARTED"),
            "[x]",
            "DEPARTED",
            "EMPTY CHAMBER",
        ),
        (
            Presence::Unknown,
            theatre(TheatrePose::Unknown, 0, false, "UNKNOWN"),
            "[?]",
            "UNKNOWN",
            "UNKNOWN",
        ),
    ];

    for (presence, theatre, marker, label, scene_marker) in cases {
        agent.presence = presence;
        let screen = render_chamber_at(&agent, theatre, false, preferences, 28, 10);

        assert!(
            screen.contains("Codex"),
            "missing name for {label}:\n{screen}"
        );
        assert!(
            screen.contains(marker),
            "missing {marker} for {label}:\n{screen}"
        );
        assert!(screen.contains(label), "missing {label}:\n{screen}");
        assert!(
            screen.contains(scene_marker),
            "missing scene marker {scene_marker} for {label}:\n{screen}"
        );
        assert!(
            screen.contains("CHAMBER"),
            "missing chamber setting for {label}:\n{screen}"
        );
    }
}

#[test]
fn focused_chamber_keeps_state_and_adds_live_lantern() {
    let mut agent = agent();
    agent.presence = Presence::Working;
    let screen = render_chamber_at(
        &agent,
        theatre(TheatrePose::Delving, 1, true, "DELVING"),
        false,
        preferences(CharacterSet::Unicode),
        28,
        10,
    );

    assert!(screen.contains("[>] DELVING"));
    assert!(screen.contains("LIVE"));
    assert!(screen.contains("(*)"));
}

#[test]
fn selection_lights_the_lamp_without_claiming_focus() {
    let agent = agent();
    let screen = render_chamber_at(
        &agent,
        theatre(TheatrePose::Delving, 1, false, "DELVING"),
        true,
        preferences(CharacterSet::Unicode),
        28,
        10,
    );

    assert!(screen.contains("[>] DELVING"));
    assert!(screen.contains("(*)"));
    assert!(!screen.contains("LIVE"));
}

#[test]
fn unicode_chamber_places_the_packed_seated_figure_in_six_scene_rows() {
    let agent = agent();
    let screen = render_chamber_at(
        &agent,
        theatre(TheatrePose::Delving, 1, false, "DELVING"),
        false,
        preferences(CharacterSet::Unicode),
        28,
        10,
    );

    let rows = screen.lines().collect::<Vec<_>>();
    assert_eq!(rows.len(), 10);
    let scene = rows[2..8].join("\n");
    assert!(
        scene
            .chars()
            .filter(|glyph| matches!(glyph, '▀' | '▄' | '█'))
            .count()
            > 15,
        "six-row chamber scene did not contain the packed seated figure:\n{screen}"
    );
}

#[test]
fn compact_unicode_chamber_keeps_the_counsel_seated_sprite_visible() {
    let agent = agent();
    let screen = render_chamber_at(
        &agent,
        theatre(TheatrePose::SeekingCounsel, 1, false, "COUNSEL REQUESTED"),
        false,
        preferences(CharacterSet::Unicode),
        14,
        6,
    );

    assert!(
        screen.contains("[!] COUNSEL"),
        "compact state marker and readable label prefix disappeared:\n{screen}"
    );
    assert!(
        screen.chars().any(|glyph| matches!(glyph, '▀' | '▄' | '█')),
        "compact counsel chamber lost its seated sprite:\n{screen}"
    );
}

#[test]
fn unicode_scene_composes_distinct_victory_and_empty_chamber_props() {
    let agent = agent();
    let preferences = preferences(CharacterSet::Unicode);
    let victory_colour = Color::Indexed(51);
    let departed_colour = Color::Indexed(244);
    let done = theatre(TheatrePose::VictoryRecorded, 0, false, "VICTORY RECORDED");
    let exited = theatre(TheatrePose::Departed, 0, false, "DEPARTED");

    let prop_mask = |pose, colour| {
        let styles = render_chamber_styles(&agent, pose, preferences);
        assert!(
            styles
                .iter()
                .any(|(foreground, background)| *foreground == colour || *background == colour),
            "pose {:?} did not render its semantic state prop",
            pose.pose
        );
        styles
            .into_iter()
            .map(|(foreground, background)| foreground == colour || background == colour)
            .collect::<Vec<_>>()
    };
    let done_prop = prop_mask(done, victory_colour);
    let exited_prop = prop_mask(exited, departed_colour);
    assert_ne!(
        done_prop, exited_prop,
        "victory banner and empty chamber arch share a silhouette"
    );

    let figure = |pose| {
        render_chamber_at(&agent, pose, false, preferences, 28, 10)
            .lines()
            .skip(2)
            .take(6)
            .map(|row| row.chars().skip(17).take(10).collect::<String>())
            .collect::<Vec<_>>()
            .join("\n")
    };
    let done_figure = figure(done);
    let exited_figure = figure(exited);
    assert!(
        exited_figure
            .chars()
            .any(|glyph| matches!(glyph, '▀' | '▄' | '█')),
        "departed chamber did not leave a visible empty rest"
    );
    assert_ne!(done_figure, exited_figure);
}

#[test]
fn profile_card_shows_the_independent_full_figure_and_actionable_details() {
    let mut agent = agent();
    agent.persona.appearance.keepsake = Keepsake::TinyFamiliar;
    let screen = render_profile_at(
        &agent,
        theatre(TheatrePose::SeekingCounsel, 1, true, "COUNSEL REQUESTED"),
        preferences(CharacterSet::Unicode),
        40,
        20,
    );

    assert!(screen.contains(&agent.persona.name));
    assert!(screen.contains("Site: w1"));
    assert!(screen.contains("[!] COUNSEL REQUESTED"));
    assert!(screen.contains("Keepsake:"));
    assert!(screen.contains("Keepsake: Familiar"));
    assert!(screen.contains("Gear:"));
    assert!(screen.contains("LIVE"));

    let rows = screen.lines().collect::<Vec<_>>();
    let rendered_figure = rows[1..17]
        .iter()
        .map(|row| row.chars().skip(1).take(16).collect::<String>())
        .collect::<Vec<_>>();
    let expected_figure = pack(
        &compose_profile_adventurer_for_palette(&agent.persona, Palette::Xterm256),
        &Palette::Xterm256,
        ColorRole::DarkStone,
    )
    .lines
    .into_iter()
    .map(|line| {
        line.spans
            .into_iter()
            .map(|span| span.content.into_owned())
            .collect::<String>()
    })
    .collect::<Vec<_>>();
    assert_eq!(rendered_figure, expected_figure);
    let figure = rows[1..17].join("\n");
    let packed_pixels = figure
        .chars()
        .filter(|glyph| matches!(glyph, '▀' | '▄' | '█'))
        .count();
    assert!(
        packed_pixels > 40,
        "full profile figure was not rendered:\n{screen}"
    );
    assert!(
        rows[1..9]
            .iter()
            .any(|row| row.chars().any(|glyph| matches!(glyph, '▀' | '▄' | '█'))),
        "profile head/torso missing:\n{screen}"
    );
    assert!(
        rows[9..17]
            .iter()
            .any(|row| row.chars().any(|glyph| matches!(glyph, '▀' | '▄' | '█'))),
        "profile legs/footwear missing:\n{screen}"
    );
}

#[test]
fn minimum_profile_boundary_never_drops_the_adventurer_name() {
    let agent = agent();
    let screen = render_profile_at(
        &agent,
        theatre(TheatrePose::Resting, 0, false, "RESTING"),
        preferences(CharacterSet::Unicode),
        34,
        18,
    );

    assert!(
        screen.contains(&agent.persona.name),
        "minimum profile boundary dropped the adventurer name:\n{screen}"
    );
}

#[test]
fn ascii_widgets_use_labelled_silhouettes_and_no_block_glyphs() {
    let agent = agent();
    let preferences = preferences(CharacterSet::Ascii);
    let blocked = theatre(TheatrePose::SeekingCounsel, 1, false, "COUNSEL REQUESTED");

    let chamber = render_chamber_at(&agent, blocked, false, preferences, 28, 10);
    assert!(chamber.contains("ADVENT [!]"));
    assert!(chamber.contains("SIGNAL LANTERN"));
    assert!(chamber.contains("[!] COUNSEL REQUESTED"));

    let profile = render_profile_at(&agent, blocked, preferences, 40, 20);
    assert!(profile.contains("ADVENTURER PROFILE"));
    assert!(profile.contains("[!] COUNSEL REQUESTED"));
    assert!(profile.contains("Keepsake:"));

    for screen in [chamber, profile] {
        assert!(
            !screen.chars().any(|glyph| matches!(glyph, '▀' | '▄' | '█')),
            "ASCII projection emitted a packed block glyph:\n{screen}"
        );
        assert!(
            screen.is_ascii(),
            "ASCII projection emitted a non-ASCII glyph:\n{screen}"
        );
    }
}

#[test]
fn ascii_presentation_sanitizes_domain_text_in_full_and_compact_widgets() {
    let mut agent = agent();
    agent.name = "Café\n主机\t\u{7}".to_owned();
    agent.persona.name = "héllø\nroot".to_owned();
    agent.workspace_id = WorkspaceId::new("sité\n一");
    agent.pane_id = PaneId::new("pane\tß");
    agent.custom_status = Some("naïve\n状态\u{1b}".to_owned());
    let preferences = preferences(CharacterSet::Ascii);
    let working = theatre(TheatrePose::Delving, 0, false, "DELVING");

    let screens = [
        render_chamber_at(&agent, working, false, preferences, 60, 10),
        render_chamber_at(&agent, working, false, preferences, 24, 4),
        render_profile_at(&agent, working, preferences, 60, 20),
        render_profile_at(&agent, working, preferences, 30, 6),
    ];

    for screen in &screens {
        assert!(screen.is_ascii(), "ASCII widget leaked Unicode:\n{screen}");
        assert!(
            screen
                .lines()
                .flat_map(str::chars)
                .all(|glyph| glyph == ' ' || glyph.is_ascii_graphic()),
            "ASCII widget leaked a control character:\n{screen}"
        );
        assert!(
            screen.contains("Caf?"),
            "name lost readable placeholder:\n{screen}"
        );
    }
    assert!(screens[0].contains("na?ve ???"));
    assert!(screens[1].contains("na?ve ???"));
    assert!(screens[2].contains("h?ll? root"));
    assert!(screens[2].contains("Site: sit? ?"));
    assert!(screens[2].contains("Pane: pane ?"));
    assert!(screens[2].contains("Status: na?ve ???"));
    assert!(screens[3].contains("h?ll? root"));
    assert!(screens[3].contains("Site: sit? ?"));
    assert!(screens[3].contains("Pane: pane ?"));
    assert!(screens[3].contains("Status: na?ve ???"));
}

#[test]
fn unicode_presentation_preserves_printable_text_but_neutralizes_controls() {
    let mut agent = agent();
    agent.name = "Café\nMüller\u{1b}".to_owned();
    agent.persona.name = "héllø-root".to_owned();
    agent.workspace_id = WorkspaceId::new("sité-é");
    agent.pane_id = PaneId::new("pane-ß");
    agent.custom_status = Some("naïve\trésumé".to_owned());
    let preferences = preferences(CharacterSet::Unicode);
    let working = theatre(TheatrePose::Delving, 0, false, "DELVING");

    let chamber = render_chamber_at(&agent, working, false, preferences, 60, 10);
    assert!(chamber.contains("Café Müller?"));
    assert!(chamber.contains("naïve résumé"));

    let profile = render_profile_at(&agent, working, preferences, 60, 20);
    assert!(profile.contains("Café Müller?"));
    assert!(profile.contains("héllø-root"));
    assert!(profile.contains("Site: sité-é"));
    assert!(profile.contains("Pane: pane-ß"));
    assert!(profile.contains("Status: naïve résumé"));
}

#[test]
fn colour_mode_selects_xterm_or_ansi_without_domain_ui_coupling() {
    assert_eq!(ColorMode::default(), ColorMode::Xterm256);

    let xterm = render_chamber_colours(preferences(CharacterSet::Unicode));
    assert!(
        xterm
            .iter()
            .any(|colour| matches!(colour, Color::Indexed(_)))
    );

    let mut ansi_preferences = preferences(CharacterSet::Unicode);
    ansi_preferences.color_mode = ColorMode::Ansi16;
    let ansi = render_chamber_colours(ansi_preferences);
    assert!(ansi.iter().any(|colour| *colour != Color::Reset));
    assert!(
        ansi.iter()
            .all(|colour| !matches!(colour, Color::Indexed(_) | Color::Rgb(_, _, _)))
    );
}

#[test]
fn zero_and_tiny_widgets_are_safe_and_tiny_cards_keep_the_state_actionable() {
    let agent = agent();
    let preferences = preferences(CharacterSet::Ascii);
    let exited = theatre(TheatrePose::Departed, 0, false, "DEPARTED");

    assert_eq!(
        render_chamber_at(&agent, exited, false, preferences, 0, 0),
        ""
    );
    assert_eq!(render_profile_at(&agent, exited, preferences, 0, 0), "");
    let _ = render_chamber_at(&agent, exited, false, preferences, 1, 1);
    let _ = render_profile_at(&agent, exited, preferences, 1, 1);

    let chamber = render_chamber_at(&agent, exited, false, preferences, 18, 4);
    let profile = render_profile_at(&agent, exited, preferences, 18, 4);
    for screen in [chamber, profile] {
        assert!(
            screen.contains("Codex"),
            "tiny card lost identity:\n{screen}"
        );
        assert!(
            screen.contains("[x] DEPARTED"),
            "tiny card lost actionable state:\n{screen}"
        );
    }
}

#[test]
fn ascii_fallback_keeps_each_action_marker_explicit() {
    let agent = agent();
    let preferences = preferences(CharacterSet::Ascii);
    let cases = [
        (TheatrePose::SeekingCounsel, "COUNSEL REQUESTED", "[!]"),
        (TheatrePose::SpoilsUnopened, "SPOILS RETURNED", "[+]"),
        (TheatrePose::Resting, "RESTING", "[~]"),
        (TheatrePose::Departed, "DEPARTED", "[x]"),
    ];

    for (pose, label, marker) in cases {
        let screen = render_chamber_at(
            &agent,
            theatre(pose, 2, false, label),
            false,
            preferences,
            28,
            10,
        );
        assert!(screen.contains(marker), "missing {marker}:\n{screen}");
        assert!(screen.contains(label), "missing {label}:\n{screen}");
    }
}

#[test]
fn injected_frames_make_runes_and_chest_sparkle_deterministic() {
    let agent = agent();
    let preferences = preferences(CharacterSet::Unicode);
    let render = |animation_frame| {
        render_chamber_at(
            &agent,
            theatre(
                TheatrePose::SpoilsUnopened,
                animation_frame,
                false,
                "SPOILS RETURNED",
            ),
            false,
            preferences,
            28,
            10,
        )
    };

    assert_eq!(render(4), render(4));
    assert_eq!(render(0), render(10));
    assert_eq!(render(9), render(11));
    assert_eq!(render(0).matches('^').count(), 0);
    for animation_frame in 1..=8 {
        let stable_same_rune_phase = if animation_frame % 2 == 0 {
            render(10)
        } else {
            render(9)
        };
        assert_ne!(
            render(animation_frame),
            stable_same_rune_phase,
            "frame {animation_frame} did not contain deterministic chest sparkle"
        );
        assert_eq!(
            render(animation_frame).matches('^').count(),
            1,
            "frame {animation_frame} did not contain exactly one chest sparkle marker"
        );
    }

    let done_seen = render_chamber_at(
        &agent,
        theatre(TheatrePose::VictoryRecorded, 0, false, "VICTORY RECORDED"),
        false,
        preferences,
        28,
        10,
    );
    assert_eq!(done_seen.matches('^').count(), 0);
}

#[test]
fn wider_chamber_includes_custom_status_when_space_allows() {
    let agent = agent();
    let screen = render_chamber_at(
        &agent,
        theatre(TheatrePose::Delving, 0, false, "DELVING"),
        false,
        preferences(CharacterSet::Unicode),
        44,
        10,
    );

    assert!(screen.contains("which schema?"));
}
