#![cfg(feature = "storybook")]

use std::{collections::HashSet, fs, path::Path};

use questmancer::{
    app::{GuildFocus, Motion, OutputPreview, View},
    domain::{
        AdventurerClass, AdventurerPersona, AgentKey, Ancestry, PaneId, PersonaKey, Presence,
        Timestamp, WorkspaceId,
    },
    scene::{
        SceneFrame,
        assets::{
            AssetError, IndexedPaletteEntry,
            adventurer::{compact_adventurer_animation_frame, compact_adventurer_frame},
            indexed_sprite,
        },
        pixel::{PixelSize, Rgb, RgbBuffer},
        render_scene, render_scene_for_story,
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot},
        stage::{ScenePlan, ScenePose, TruthfulStation, WorldScene},
    },
    storybook::{
        AssetId, SceneFirstAsset,
        app::{Action, StorybookApp, reduce},
        catalogue::catalogue,
        fixtures::{AtlasContent, StoryContext, StoryFixture, guild_fixture},
        ui as storybook_ui,
    },
};
use ratatui::{Terminal, backend::TestBackend, style::Color};

fn scene_agent(key: &str, presence: Presence) -> SceneAgent {
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: WorkspaceId::new(format!("workspace-{key}")),
        name: key.to_owned(),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(1_000),
        transition: None,
        focused: false,
        persona: AdventurerPersona::for_key(PersonaKey::new(format!("scene-{key}"))),
    }
}

fn snapshot() -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: Vec::new(),
        agents: vec![
            scene_agent("working", Presence::Working),
            scene_agent("resting", Presence::Idle),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(2_000),
    }
}

fn rust_sources(root: &Path, paths: &mut Vec<std::path::PathBuf>) {
    for entry in fs::read_dir(root).unwrap() {
        let path = entry.unwrap().path();
        if path.is_dir() {
            rust_sources(&path, paths);
        } else if path.extension().is_some_and(|extension| extension == "rs") {
            paths.push(path);
        }
    }
}

#[test]
fn indexed_assets_reject_ragged_rows_duplicate_keys_and_unknown_glyphs_exactly() {
    let red = Rgb::new(200, 10, 20);
    assert_eq!(
        indexed_sprite(
            &["aa", "a"],
            &[IndexedPaletteEntry {
                key: 'a',
                colour: Some(red),
            }],
        ),
        Err(AssetError::RaggedRows {
            row: 1,
            expected: 2,
            actual: 1,
        })
    );
    assert_eq!(
        indexed_sprite(
            &["a"],
            &[
                IndexedPaletteEntry {
                    key: 'a',
                    colour: Some(red),
                },
                IndexedPaletteEntry {
                    key: 'a',
                    colour: None,
                },
            ],
        ),
        Err(AssetError::DuplicatePaletteKey { key: 'a' })
    );
    assert_eq!(
        indexed_sprite(
            &["az"],
            &[IndexedPaletteEntry {
                key: 'a',
                colour: Some(red)
            }]
        ),
        Err(AssetError::UnknownGlyph {
            glyph: 'z',
            row: 0,
            column: 1,
        })
    );
}

#[test]
fn indexed_assets_reserve_dot_for_transparency() {
    let frame = indexed_sprite(
        &[".a"],
        &[IndexedPaletteEntry {
            key: 'a',
            colour: Some(Rgb::new(1, 2, 3)),
        }],
    )
    .unwrap();
    assert_eq!(frame.size(), PixelSize::new(2, 1));
    assert_eq!(frame.pixels(), &[None, Some(Rgb::new(1, 2, 3))]);
    assert_eq!(
        indexed_sprite(
            &["."],
            &[IndexedPaletteEntry {
                key: '.',
                colour: Some(Rgb::new(1, 2, 3)),
            }],
        ),
        Err(AssetError::ReservedTransparencyKey)
    );
}

#[test]
fn every_compact_pose_and_ancestry_class_pair_has_an_eight_by_fourteen_silhouette() {
    for &ancestry in Ancestry::ALL {
        for &class in AdventurerClass::ALL {
            let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!(
                "compact-{ancestry:?}-{class:?}"
            )));
            persona.ancestry = ancestry;
            persona.class = class;
            for pose in [
                ScenePose::Working,
                ScenePose::SeekingCounsel,
                ScenePose::ReturningWithSpoils,
                ScenePose::Settled,
                ScenePose::Resting,
                ScenePose::Unknown,
            ] {
                for alternate in [false, true] {
                    let frame = compact_adventurer_frame(&persona, pose, alternate);
                    assert_eq!(frame.size(), PixelSize::new(8, 14));
                    assert!(frame.pixels().iter().any(Option::is_some));
                }
            }
        }
    }

    let persona = AdventurerPersona::for_key(PersonaKey::new("compact-animation"));
    let frames = (0..3)
        .map(|frame| compact_adventurer_animation_frame(&persona, ScenePose::Working, frame))
        .collect::<Vec<_>>();
    assert_ne!(frames[0].pixels(), frames[1].pixels());
    assert_ne!(frames[1].pixels(), frames[2].pixels());
}

#[test]
fn persona_identity_changes_pixels_without_changing_the_station_contract() {
    let first = AdventurerPersona::for_key(PersonaKey::new("compact-first"));
    let second = AdventurerPersona::for_key(PersonaKey::new("compact-second"));
    let first_frame = compact_adventurer_frame(&first, ScenePose::Working, false);
    let second_frame = compact_adventurer_frame(&second, ScenePose::Working, false);
    assert_eq!(first_frame.size(), second_frame.size());
    assert_ne!(first_frame.pixels(), second_frame.pixels());

    let first_snapshot = snapshot();
    let mut second_snapshot = first_snapshot.clone();
    second_snapshot.agents[0].persona = second;
    let first_plan = ScenePlan::project(&first_snapshot, PixelSize::new(120, 72));
    let second_plan = ScenePlan::project(&second_snapshot, PixelSize::new(120, 72));
    assert_eq!(first_plan.actors, second_plan.actors);
}

#[test]
fn render_scene_reuses_the_target_and_paints_a_deterministic_continuous_room() {
    let mut target = RgbBuffer::filled(1, 1, Rgb::new(255, 0, 255));
    let first = render_scene(&snapshot(), PixelSize::new(120, 72), &mut target);
    assert_eq!(
        first,
        SceneFrame {
            world: WorldScene::Delve,
            next_frame_in: None
        }
    );
    assert_eq!(target.size(), PixelSize::new(120, 72));
    let pixels = target.pixels().to_vec();
    let colours = pixels.iter().copied().collect::<HashSet<_>>();
    assert!(colours.len() >= 12, "calibration room needs material depth");
    assert!(pixels.iter().all(|pixel| *pixel != Rgb::new(255, 0, 255)));
    let capacity = target.capacity();

    let second = render_scene(&snapshot(), PixelSize::new(120, 72), &mut target);
    assert_eq!(first, second);
    assert_eq!(target.capacity(), capacity);
    assert_eq!(pixels, target.pixels());
}

#[test]
fn story_override_reprojects_truthful_stations_for_the_requested_world() {
    let mut value = snapshot();
    value.agents.truncate(1);
    let viewport = PixelSize::new(120, 72);
    let automatic_plan = ScenePlan::project(&value, viewport);
    assert!(matches!(
        automatic_plan.actors[0].station,
        TruthfulStation::DelveActive(_)
    ));

    let mut automatic_pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let automatic = render_scene_for_story(&value, None, viewport, &mut automatic_pixels);
    let mut guild_pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let guild = render_scene_for_story(
        &value,
        Some(WorldScene::GuildHall),
        viewport,
        &mut guild_pixels,
    );
    assert_eq!(automatic.world, WorldScene::Delve);
    assert_eq!(guild.world, WorldScene::GuildHall);
    assert_ne!(automatic_pixels.pixels(), guild_pixels.pixels());
    assert_ne!(automatic_pixels.get(37, 50), guild_pixels.get(37, 50));
}

#[test]
#[allow(
    clippy::too_many_lines,
    reason = "the exhaustive SceneFirstAsset ownership table is intentionally visible"
)]
fn scene_first_stories_have_exhaustive_ownership_and_render_rgb_half_blocks() {
    assert_eq!(
        SceneFirstAsset::ALL,
        &[
            SceneFirstAsset::CalibrationRoom,
            SceneFirstAsset::CompactAdventurers,
            SceneFirstAsset::GuildHallEmpty,
            SceneFirstAsset::GuildHallMixedParty,
            SceneFirstAsset::GuildHallCounselRequested,
            SceneFirstAsset::GuildHallSpoilsReturned,
            SceneFirstAsset::GuildHallReconnecting,
            SceneFirstAsset::GuildHallMinimumViewport,
            SceneFirstAsset::DelveActiveParty,
            SceneFirstAsset::DelveMixedStates,
            SceneFirstAsset::DelveSealedGate,
            SceneFirstAsset::DelveReconnecting,
            SceneFirstAsset::DelveMinimumViewport,
            SceneFirstAsset::MotionFull,
            SceneFirstAsset::MotionReduced,
            SceneFirstAsset::MotionNone,
            SceneFirstAsset::MinimumViewport,
        ]
    );
    let stories = catalogue();
    for (id, asset) in [
        (
            "scenes.rgb-calibration-room",
            SceneFirstAsset::CalibrationRoom,
        ),
        (
            "atlas.compact-scene-adventurers",
            SceneFirstAsset::CompactAdventurers,
        ),
        ("scenes.guild-hall-empty", SceneFirstAsset::GuildHallEmpty),
        (
            "scenes.guild-hall-mixed-party",
            SceneFirstAsset::GuildHallMixedParty,
        ),
        (
            "scenes.guild-hall-counsel-requested",
            SceneFirstAsset::GuildHallCounselRequested,
        ),
        (
            "scenes.guild-hall-spoils-returned",
            SceneFirstAsset::GuildHallSpoilsReturned,
        ),
        (
            "scenes.guild-hall-reconnecting",
            SceneFirstAsset::GuildHallReconnecting,
        ),
        (
            "scenes.guild-hall-minimum-viewport",
            SceneFirstAsset::GuildHallMinimumViewport,
        ),
        (
            "scenes.delve-active-party",
            SceneFirstAsset::DelveActiveParty,
        ),
        (
            "scenes.delve-mixed-states",
            SceneFirstAsset::DelveMixedStates,
        ),
        ("scenes.delve-sealed-gate", SceneFirstAsset::DelveSealedGate),
        (
            "scenes.delve-reconnecting",
            SceneFirstAsset::DelveReconnecting,
        ),
        (
            "scenes.delve-minimum-viewport",
            SceneFirstAsset::DelveMinimumViewport,
        ),
        (
            "scenes.scene-first-motion-full",
            SceneFirstAsset::MotionFull,
        ),
        (
            "scenes.scene-first-motion-reduced",
            SceneFirstAsset::MotionReduced,
        ),
        (
            "scenes.scene-first-motion-none",
            SceneFirstAsset::MotionNone,
        ),
        (
            "scenes.scene-first-minimum-viewport",
            SceneFirstAsset::MinimumViewport,
        ),
    ] {
        let index = stories
            .iter()
            .position(|story| story.id.as_str() == id)
            .unwrap();
        let story = &stories[index];
        assert_eq!(story.owns, &[AssetId::SceneFirst(asset)]);
        let fixture = (story.build)(&StoryContext::fixed());
        match asset {
            SceneFirstAsset::CalibrationRoom
            | SceneFirstAsset::GuildHallEmpty
            | SceneFirstAsset::GuildHallMixedParty
            | SceneFirstAsset::GuildHallCounselRequested
            | SceneFirstAsset::GuildHallSpoilsReturned
            | SceneFirstAsset::GuildHallReconnecting
            | SceneFirstAsset::GuildHallMinimumViewport
            | SceneFirstAsset::DelveActiveParty
            | SceneFirstAsset::DelveMixedStates
            | SceneFirstAsset::DelveSealedGate
            | SceneFirstAsset::DelveReconnecting
            | SceneFirstAsset::DelveMinimumViewport
            | SceneFirstAsset::MotionFull
            | SceneFirstAsset::MotionReduced
            | SceneFirstAsset::MotionNone
            | SceneFirstAsset::MinimumViewport => {
                assert!(matches!(fixture, StoryFixture::PixelScene(_)));
            }
            SceneFirstAsset::CompactAdventurers => {
                assert!(matches!(fixture, StoryFixture::AssetAtlas(_)));
            }
        }

        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        reduce(&mut app, Action::Inspect, stories);
        let backend = TestBackend::new(
            story.viewport.reference_width,
            story.viewport.reference_height.saturating_add(1),
        );
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| storybook_ui::render(frame, &app, stories, &StoryContext::fixed()))
            .unwrap();
        let cells = terminal.backend().buffer().content();
        assert!(cells.iter().any(|cell| {
            cell.symbol() == "▀"
                && matches!(cell.fg, Color::Rgb(_, _, _))
                && matches!(cell.bg, Color::Rgb(_, _, _))
        }));
    }

    for asset in SceneFirstAsset::ALL {
        assert_eq!(
            stories
                .iter()
                .filter(|story| story.owns.contains(&AssetId::SceneFirst(*asset)))
                .count(),
            1,
            "{asset:?} must have exactly one Storybook owner"
        );
    }
}

#[test]
fn six_fixed_guild_hall_stories_use_direct_snapshots_and_unique_scene_first_ownership() {
    let stories = catalogue();
    let expected = [
        ("scenes.guild-hall-empty", "Guild Hall Empty"),
        ("scenes.guild-hall-mixed-party", "Guild Hall Mixed Party"),
        (
            "scenes.guild-hall-counsel-requested",
            "Guild Hall Counsel Requested",
        ),
        (
            "scenes.guild-hall-spoils-returned",
            "Guild Hall Spoils Returned",
        ),
        ("scenes.guild-hall-reconnecting", "Guild Hall Reconnecting"),
        (
            "scenes.guild-hall-minimum-viewport",
            "Guild Hall Minimum Viewport",
        ),
    ];
    let mut owners = HashSet::new();
    for (id, title) in expected {
        let story = stories
            .iter()
            .find(|story| story.id.as_str() == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(story.title, title);
        assert_eq!(story.owns.len(), 1);
        assert!(matches!(story.owns[0], AssetId::SceneFirst(_)));
        assert!(owners.insert(story.owns[0]), "duplicate owner for {id}");
        let StoryFixture::PixelScene(fixture) = (story.build)(&StoryContext::fixed()) else {
            panic!("{id} must derive directly from a SceneSnapshot fixture");
        };
        assert_eq!(fixture.world_override, Some(WorldScene::GuildHall));
    }
    assert_eq!(owners.len(), expected.len());
}

#[test]
fn five_fixed_delve_stories_use_direct_snapshots_and_unique_scene_first_ownership() {
    let stories = catalogue();
    let expected = [
        ("scenes.delve-active-party", "Delve Active Party"),
        ("scenes.delve-mixed-states", "Delve Mixed States"),
        ("scenes.delve-sealed-gate", "Delve Sealed Gate"),
        ("scenes.delve-reconnecting", "Delve Reconnecting"),
        ("scenes.delve-minimum-viewport", "Delve Minimum Viewport"),
    ];
    let mut owners = HashSet::new();
    for (id, title) in expected {
        let story = stories
            .iter()
            .find(|story| story.id.as_str() == id)
            .unwrap_or_else(|| panic!("missing {id}"));
        assert_eq!(story.title, title);
        assert_eq!(story.owns.len(), 1);
        assert!(matches!(story.owns[0], AssetId::SceneFirst(_)));
        assert!(owners.insert(story.owns[0]), "duplicate owner for {id}");
        let StoryFixture::PixelScene(fixture) = (story.build)(&StoryContext::fixed()) else {
            panic!("{id} must derive directly from a SceneSnapshot fixture");
        };
        assert_eq!(fixture.world_override, Some(WorldScene::Delve));
    }
    assert_eq!(owners.len(), expected.len());

    let sealed = stories
        .iter()
        .find(|story| story.id.as_str() == "scenes.delve-sealed-gate")
        .expect("sealed gate story exists");
    let StoryFixture::PixelScene(sealed) = (sealed.build)(&StoryContext::fixed()) else {
        unreachable!();
    };
    assert!(
        sealed
            .snapshot
            .agents
            .iter()
            .any(|agent| agent.presence == Presence::Blocked)
    );
}

#[test]
fn compact_adventurer_atlas_contains_each_authored_frame_exactly_once() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.compact-scene-adventurers")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("compact scene adventurers must be an asset atlas");
    };
    let expected = [
        ("Working", ScenePose::Working, 0),
        ("Seeking counsel", ScenePose::SeekingCounsel, 0),
        ("Returning with spoils", ScenePose::ReturningWithSpoils, 0),
        ("Settled", ScenePose::Settled, 0),
        ("Resting", ScenePose::Resting, 0),
        ("Unknown", ScenePose::Unknown, 0),
        ("Working alternate", ScenePose::Working, 1),
        ("Walking alternate", ScenePose::Working, 2),
    ];
    assert_eq!(atlas.tiles.len(), expected.len());

    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-compact-scene-atlas"));
    let mut counts = [0_u8; 8];
    for tile in &atlas.tiles {
        let AtlasContent::RgbSprite { frame, .. } = &tile.content else {
            panic!("{} must contain an RGB sprite frame", tile.label);
        };
        let matching = expected
            .iter()
            .enumerate()
            .filter(|(_, (label, pose, animation_frame))| {
                tile.label == *label
                    && frame
                        == &compact_adventurer_animation_frame(&persona, *pose, *animation_frame)
            })
            .map(|(index, _)| index)
            .collect::<Vec<_>>();
        assert_eq!(matching.len(), 1, "unexpected atlas tile {}", tile.label);
        counts[matching[0]] += 1;
    }
    assert_eq!(counts, [1; 8]);
}

#[test]
fn compact_adventurer_atlas_is_not_the_calibration_room_fixture_or_render() {
    let stories = catalogue();
    let calibration_index = stories
        .iter()
        .position(|story| story.id.as_str() == "scenes.rgb-calibration-room")
        .unwrap();
    let atlas_index = stories
        .iter()
        .position(|story| story.id.as_str() == "atlas.compact-scene-adventurers")
        .unwrap();
    let context = StoryContext::fixed();
    assert_ne!(
        (stories[calibration_index].build)(&context),
        (stories[atlas_index].build)(&context)
    );

    let render = |index| {
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        reduce(&mut app, Action::Inspect, stories);
        let backend = TestBackend::new(120, 37);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| storybook_ui::render(frame, &app, stories, &context))
            .unwrap();
        terminal.backend().buffer().clone()
    };
    assert_ne!(render(calibration_index), render(atlas_index));
}

#[test]
fn pixel_scene_dispatch_uses_logical_half_block_dimensions_without_the_application_renderer() {
    let source = fs::read_to_string("src/storybook/ui.rs").unwrap();
    let arm = source
        .split("StoryFixture::PixelScene")
        .nth(1)
        .unwrap()
        .split("pub fn render_application_buffer")
        .next()
        .unwrap();
    assert!(arm.contains("PixelSize::new(area.width, area.height.saturating_mul(2))"));
    assert!(arm.contains("flush_rgb(frame.buffer_mut(), area, &buffer, Rgb::BLACK)"));
    assert!(!arm.contains("render_application_buffer"));
    assert!(!arm.contains("crate::ui::render"));
    assert!(!arm.contains("Terminal::new"));
}

#[test]
fn calibration_story_has_dense_rgb_output_at_the_review_matrix() {
    let stories = catalogue();
    let index = stories
        .iter()
        .position(|story| story.id.as_str() == "scenes.rgb-calibration-room")
        .unwrap();
    let mut app = StorybookApp::new(stories);
    app.select(index, stories);
    reduce(&mut app, Action::Inspect, stories);

    for (width, height) in [(160, 45), (120, 36), (80, 24)] {
        let backend = TestBackend::new(width, height);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| storybook_ui::render(frame, &app, stories, &StoryContext::fixed()))
            .unwrap();
        let rgb_half_blocks = terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .filter(|cell| {
                cell.symbol() == "▀"
                    && matches!(cell.fg, Color::Rgb(_, _, _))
                    && matches!(cell.bg, Color::Rgb(_, _, _))
            })
            .count();
        assert!(
            rgb_half_blocks > usize::from(width) * usize::from(height) / 2,
            "calibration room is too sparse at {width}x{height}"
        );
    }
}

#[test]
fn equal_snapshots_ignore_legacy_interaction_fields_and_render_identical_rgb_pixels() {
    let context = StoryContext::fixed();
    let baseline = guild_fixture(&context);
    let mut changed = baseline.clone();
    changed.switch_to(View::Delve);
    changed.select_last_agent();
    changed.set_guild_focus(GuildFocus::Door);
    changed.toggle_help();
    changed.set_output_preview(Some(OutputPreview {
        pane_id: PaneId::new("legacy-preview"),
        revision: 99,
        text: "legacy interaction state".to_owned(),
        loading: true,
        error: None,
    }));
    changed.set_reviewr_available(true);
    changed.set_reviewr_availability_diagnostic("legacy diagnostic".to_owned());

    let baseline_snapshot = SceneSnapshot::from_model(&baseline);
    let changed_snapshot = SceneSnapshot::from_model(&changed);
    assert_eq!(baseline_snapshot, changed_snapshot);
    let mut baseline_pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let mut changed_pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    render_scene(
        &baseline_snapshot,
        PixelSize::new(120, 72),
        &mut baseline_pixels,
    );
    render_scene(
        &changed_snapshot,
        PixelSize::new(120, 72),
        &mut changed_pixels,
    );
    assert_eq!(baseline_pixels.pixels(), changed_pixels.pixels());
}

#[test]
fn scene_core_is_terminal_free_and_excludes_legacy_mutable_ui_dependencies() {
    let forbidden = [
        "GuildFocus",
        "Modal",
        "OutputPreview",
        "Reviewr",
        "selected_agent",
        "reduce_action",
        "AgentCommand",
        "ratatui",
        "crossterm",
    ];
    let mut paths = Vec::new();
    rust_sources(Path::new("src/scene"), &mut paths);
    for path in paths {
        let source = fs::read_to_string(&path).unwrap();
        for needle in forbidden {
            assert!(
                !source.contains(needle),
                "{} contains {needle}",
                path.display()
            );
        }
    }
}
