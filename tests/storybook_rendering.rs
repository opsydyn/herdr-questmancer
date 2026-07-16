#![cfg(feature = "storybook")]

use questmancer::{
    domain::{AdventurerPersona, PersonaKey},
    storybook::{
        AssetId,
        app::{Action, StorybookApp, reduce},
        asset_inventory,
        catalogue::{Category, Story, StoryId, Viewport, catalogue},
        fixtures::{AtlasContent, StoryContext, StoryFixture, guild_fixture},
        ui as storybook_ui,
    },
    ui::{
        persona::compose_chamber_adventurer_for_palette,
        pixel::pack,
        theatre::{TheatreFrame, TheatrePose},
    },
};
use ratatui::{
    Terminal,
    backend::TestBackend,
    buffer::{Buffer, Cell},
    layout::Rect,
};

fn render_storybook(app: &StorybookApp, stories: &[Story], width: u16, height: u16) -> String {
    let backend = TestBackend::new(width, height);
    let mut terminal = Terminal::new(backend).unwrap();
    terminal
        .draw(|frame| {
            storybook_ui::render(frame, app, stories, &StoryContext::fixed());
        })
        .unwrap();
    terminal
        .backend()
        .buffer()
        .content()
        .iter()
        .map(Cell::symbol)
        .collect::<String>()
}

#[allow(
    clippy::trivially_copy_pass_by_ref,
    reason = "StoryBuilder requires a borrowed StoryContext"
)]
fn application_fixture(context: &StoryContext) -> StoryFixture {
    StoryFixture::Application(guild_fixture(context))
}

fn application_story() -> Story {
    Story::new(
        StoryId::new("scene.guild-hall"),
        "Guild Hall",
        Category::FullScenes,
        "The production Guild Hall with deterministic adventurers.",
        Viewport::new(130, 32, 60, 18),
        application_fixture,
        &[],
        &[],
    )
}

#[test]
fn wide_shell_shows_catalogue_canvas_and_coverage() {
    let stories = catalogue();
    let app = StorybookApp::new(stories);
    let screen = render_storybook(&app, stories, 140, 40);
    assert!(screen.contains("QUESTMANCER STORYBOOK"));
    assert!(screen.contains("STORIES"));
    assert!(screen.contains("PRODUCTION CANVAS"));
    assert!(screen.contains("COVERAGE"));
    assert!(screen.contains("offline fixture realm"));
}

#[test]
fn medium_shell_stacks_evidence_below_the_catalogue() {
    let stories = catalogue();
    let app = StorybookApp::new(stories);
    let screen = render_storybook(&app, stories, 100, 36);
    assert!(screen.contains("STORIES"));
    assert!(screen.contains("PRODUCTION CANVAS"));
    assert!(screen.contains("COVERAGE"));
    assert!(screen.contains("Classes and Gear"));
}

#[test]
fn narrow_shell_uses_a_one_line_story_selector() {
    let stories = catalogue();
    let app = StorybookApp::new(stories);
    let screen = render_storybook(&app, stories, 79, 24);
    assert!(screen.contains("1/15 Classes and Gear"));
    assert!(screen.contains("PRODUCTION CANVAS"));
    assert!(!screen.contains("STORIES"));
    assert!(!screen.contains("COVERAGE"));
}

#[test]
fn tiny_canvas_explains_the_selected_story_minimum() {
    let stories = catalogue();
    let app = StorybookApp::new(stories);
    let screen = render_storybook(&app, stories, 48, 12);
    assert!(screen.contains("This story needs at least 60x18."));
    assert!(screen.contains("Canvas available: 46x7."));
}

#[test]
fn inspection_hides_catalogue_chrome() {
    let stories = catalogue();
    let mut app = StorybookApp::new(stories);
    reduce(&mut app, Action::Inspect, stories);
    let screen = render_storybook(&app, stories, 120, 36);
    assert!(!screen.contains("STORIES"));
    assert!(screen.contains("[esc] catalogue"));
}

#[test]
fn help_overlay_lists_every_navigation_and_exit_key() {
    let stories = catalogue();
    let mut app = StorybookApp::new(stories);
    reduce(&mut app, Action::ToggleHelp, stories);
    let screen = render_storybook(&app, stories, 120, 36);
    for key in [
        "j/down", "k/up", "h/left", "l/right", "enter", "?", "esc", "q", "ctrl-c",
    ] {
        assert!(screen.contains(key), "missing {key}: {screen}");
    }
}

#[test]
fn application_story_blits_production_render_without_overwriting_shell() {
    let stories = [application_story()];
    let app = StorybookApp::new(&stories);
    let screen = render_storybook(&app, &stories, 140, 40);
    assert!(screen.contains("QUESTMANCER STORYBOOK"));
    assert!(screen.contains("STORIES"));
    assert!(screen.contains("COVERAGE"));
    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    assert!(screen.contains("Forgotten Library"));
}

#[test]
fn inspection_application_story_uses_the_full_production_renderer() {
    let stories = [application_story()];
    let mut app = StorybookApp::new(&stories);
    reduce(&mut app, Action::Inspect, &stories);
    let screen = render_storybook(&app, &stories, 130, 33);
    assert!(screen.contains("QUESTMANCER'S GUILD HALL"));
    assert!(screen.contains("PARTY ROSTER"));
    assert!(screen.contains("CALLS FOR COUNSEL"));
    assert!(screen.contains("[esc] catalogue"));
}

#[test]
fn blit_clips_at_the_target_edge_and_preserves_the_requested_offset() {
    let mut source = Buffer::empty(Rect::new(0, 0, 4, 2));
    source.cell_mut((0, 0)).unwrap().set_symbol("A");
    source.cell_mut((1, 0)).unwrap().set_symbol("B");
    source.cell_mut((2, 0)).unwrap().set_symbol("C");
    let mut target = Buffer::empty(Rect::new(0, 0, 5, 3));

    storybook_ui::blit(&source, &mut target, Rect::new(3, 2, 4, 2));

    assert_eq!(target.cell((3, 2)).unwrap().symbol(), "A");
    assert_eq!(target.cell((4, 2)).unwrap().symbol(), "B");
    assert!(!target.content().iter().any(|cell| cell.symbol() == "C"));
}

#[test]
fn class_atlas_uses_production_profile_canvases() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.classes")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("class atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 11);
    for tile in &atlas.tiles {
        let AtlasContent::Pixel { canvas, .. } = &tile.content else {
            panic!("class tiles must contain production pixel canvases");
        };
        assert_eq!((canvas.width(), canvas.height()), (16, 32));
        assert!(canvas.pixels().iter().any(Option::is_some));
    }
}

#[test]
fn pose_atlas_uses_all_seven_production_theatre_poses() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.poses")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("pose atlas must be an asset atlas");
    };
    assert_eq!(atlas.tiles.len(), 7);
}

#[test]
fn pixel_atlases_are_packed_through_the_production_packer() {
    for story in catalogue() {
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("atlas catalogue entries must build asset atlases");
        };
        for tile in atlas.tiles {
            let AtlasContent::Pixel {
                canvas,
                palette,
                background,
                packed,
            } = tile.content
            else {
                panic!("Task 4 atlas tiles must contain production pixel content");
            };
            assert_eq!(packed, pack(&canvas, &palette, background));
        }
    }
}

#[test]
fn every_atlas_builder_matches_its_canonical_asset_family() {
    let inventory = asset_inventory();
    for story in catalogue() {
        let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
            panic!("atlas catalogue entries must build asset atlases");
        };
        let expected = inventory
            .iter()
            .filter(|asset| asset_belongs_to_story(**asset, story.id.as_str()))
            .map(|asset| asset.label())
            .collect::<Vec<_>>();
        let actual = atlas
            .tiles
            .iter()
            .map(|tile| tile.label)
            .collect::<Vec<_>>();
        assert_eq!(actual, expected, "{}", story.id.as_str());
    }
}

#[test]
fn pose_atlas_uses_the_exact_production_pose_and_frame_mapping() {
    let story = catalogue()
        .iter()
        .find(|story| story.id.as_str() == "atlas.poses")
        .unwrap();
    let StoryFixture::AssetAtlas(atlas) = (story.build)(&StoryContext::fixed()) else {
        panic!("pose atlas must be an asset atlas");
    };
    let persona = AdventurerPersona::for_key(PersonaKey::new("storybook-pose-atlas"));
    let poses = asset_inventory()
        .into_iter()
        .filter_map(|asset| match asset {
            AssetId::Pose(pose) => Some((asset.label(), pose)),
            _ => None,
        });

    for (tile, (label, pose)) in atlas.tiles.iter().zip(poses) {
        let AtlasContent::Pixel {
            canvas, palette, ..
        } = &tile.content
        else {
            panic!("pose tiles must contain production pixel canvases");
        };
        let animation_frame = if pose == TheatrePose::SpoilsUnopened {
            4
        } else {
            0
        };
        let expected = compose_chamber_adventurer_for_palette(
            &persona,
            TheatreFrame {
                pose,
                animation_frame,
                focused: false,
                label,
            },
            *palette,
        );
        assert_eq!(canvas, &expected, "{label}");
    }
}

fn asset_belongs_to_story(asset: AssetId, story_id: &str) -> bool {
    matches!(
        (story_id, asset),
        ("atlas.classes", AssetId::Class(_))
            | ("atlas.ancestries", AssetId::Ancestry(_))
            | ("atlas.body-proportions", AssetId::BodyProportions(_))
            | ("atlas.head-shapes", AssetId::HeadShape(_))
            | ("atlas.skin-tones", AssetId::SkinTone(_))
            | ("atlas.hair-shapes", AssetId::HairShape(_))
            | ("atlas.hair-tones", AssetId::HairTone(_))
            | ("atlas.face-details", AssetId::FaceDetail(_))
            | ("atlas.garb", AssetId::Garb(_))
            | ("atlas.legwear", AssetId::Legwear(_))
            | ("atlas.footwear", AssetId::Footwear(_))
            | ("atlas.keepsakes", AssetId::Keepsake(_))
            | ("atlas.accent-tones", AssetId::AccentTone(_))
            | ("atlas.palette-roles", AssetId::ColorRole(_))
            | ("atlas.poses", AssetId::Pose(_))
    )
}
