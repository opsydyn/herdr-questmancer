#![cfg(feature = "storybook")]

use questmancer::storybook::{
    app::{Action, StorybookApp, reduce},
    catalogue::{catalogue, validate_catalogue},
    fixtures::{StoryContext, StoryFixture},
    ui,
};
use ratatui::{Terminal, backend::TestBackend};

#[test]
fn core_ux_room_studies_are_available_for_review() {
    for id in [
        "world.delve-walk",
        "world.delve-materials",
        "world.guild-materials",
    ] {
        assert!(
            catalogue().iter().any(|story| story.id.as_str() == id),
            "missing {id}"
        );
    }
}

#[test]
fn scrying_story_contains_settled_output_without_live_io() {
    let model = questmancer::storybook::fixtures::scrying_interaction_fixture(
        questmancer::storybook::fixtures::StoryContext::fixed(),
    );
    let preview = model.output_preview().expect("scrying story has output");
    assert!(
        !preview.loading,
        "a fixed story must not wait for a real socket"
    );
    assert_eq!(preview.text, "The runes resolve into a clean test report.");
    assert_eq!(preview.error, None);
}

#[test]
fn catalogue_contains_every_production_scene_interaction_once() {
    assert_eq!(catalogue().len(), 91);
    let titles = catalogue()
        .iter()
        .map(|story| story.title)
        .collect::<Vec<_>>();

    for title in [
        "Fit / Hall — Caster Party",
        "Fit / Delve — Caster Party",
        "Assets / Core World Masters",
        "Assets / Barbarian Poses",
        "Assets / Wizard Poses",
        "Assets / Ranger Poses",
        "Assets / Bard Poses",
        "Assets / Artificer Poses",
        "Assets / Testmender Poses",
        "Assets / Cleric Poses",
        "Assets / Paladin Poses",
        "Assets / Druid Poses",
        "Assets / Rogue Poses",
        "Assets / Pathseeker Poses",
        "Assets / Runewright Poses",
        "Assets / Mage Poses",
        "Assets / Sorcerer Poses",
        "Assets / Core Portrait Masters",
        "Assets / Goblin Easter Egg",
        "Assets / Librarian",
        "Asset / Native Artificer Card",
        "Asset / Native Barbarian Card",
        "Asset / Native Bard Card",
        "Asset / Native Cleric Card",
        "Asset / Native Druid Card",
        "Asset / Native Paladin Card",
        "Asset / Native Rogue Card",
        "Asset / Native Ranger Card",
        "Asset / Native Wizard Card",
        "Asset / Native Testmender Card",
        "Asset / Reserved Goblin Event Art",
        "Asset / Reserved Orc Event Art",
        "Interaction / Selected Adventurer",
        "Interaction / Counsel Parchment",
        "Interaction / Search Parchment",
        "Interaction / Scrying Parchment",
        "Interaction / Librarian's Ledger",
        "Interaction / Narrow Parchment",
    ] {
        assert_eq!(
            titles
                .iter()
                .filter(|candidate| **candidate == title)
                .count(),
            1,
            "missing or duplicated Storybook interaction: {title}"
        );
    }
}

#[test]
fn rat_royalty_gallery_shows_all_three_sprite_sizes_at_its_minimum() {
    let stories = catalogue();
    let index = stories
        .iter()
        .position(|story| story.id.as_str() == "asset.rat-royalty")
        .expect("Rat Royalty has an asset story");
    let mut app = StorybookApp::new(stories);
    app.select(index, stories);
    let _ = reduce(&mut app, Action::Inspect, stories);
    let mut terminal = Terminal::new(TestBackend::new(80, 28)).unwrap();
    terminal
        .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed(), None))
        .unwrap();
    let output: String = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    for label in ["World 16x24", "Roster 8x12", "Profile 24x32"] {
        assert!(output.contains(label), "missing {label} at 80x28");
    }
}

#[test]
fn owned_authoring_has_fixed_production_recovery_and_narrow_stories() {
    for id in [
        "interaction.owned-actions",
        "interaction.owned-empty-campaign",
        "interaction.owned-answer",
        "interaction.owned-answer-narrow",
        "interaction.owned-view-pending",
        "interaction.owned-indeterminate",
        "interaction.owned-answered",
        "interaction.owned-small",
    ] {
        assert!(
            catalogue().iter().any(|s| s.id.as_str() == id),
            "missing {id}"
        );
    }
}

#[test]
fn narrow_answer_keeps_its_action_title_while_the_focused_field_scrolls() {
    use questmancer::storybook::fixtures::{OwnedAuthoringState, owned_authoring_fixture};
    let mut model = owned_authoring_fixture(StoryContext::fixed(), OwnedAuthoringState::Narrow);
    model.authoring_mut().follow_focus = true;
    let mut terminal = Terminal::new(TestBackend::new(40, 20)).unwrap();
    terminal
        .draw(|f| questmancer::ui::owned_authoring::render(f, &model))
        .unwrap();
    let text = terminal
        .backend()
        .buffer()
        .content
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect::<String>();
    assert!(
        text.contains("Answer decision"),
        "the form loses its action context when editing"
    );
}

#[test]
fn every_production_asset_has_exactly_one_owner_with_variant_stories_allowed() {
    let report = validate_catalogue().expect("production Storybook coverage is complete");
    assert_eq!(
        report.owned(),
        questmancer::storybook::asset_inventory().len()
    );
    assert!(
        catalogue()
            .iter()
            .all(|story| story.owns.len() == 1 || !story.shows.is_empty())
    );
}

#[test]
fn every_production_story_renders_at_its_minimum_viewport() {
    let stories = catalogue();
    for (index, story) in stories.iter().enumerate() {
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        let backend = TestBackend::new(
            story.viewport.minimum_width,
            story.viewport.minimum_height.saturating_add(2),
        );
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed(), None))
            .unwrap();
    }
}

#[test]
fn scene_inspection_reaches_small_production_layouts_while_asset_galleries_keep_their_minimum() {
    let stories = catalogue();
    for (index, story) in stories.iter().enumerate() {
        let context = StoryContext::fixed();
        let is_scene = matches!(
            (story.build)(context),
            StoryFixture::SceneApplication(_) | StoryFixture::WorldFit(_)
        );
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        let _ = reduce(&mut app, Action::Inspect, stories);
        for (width, height) in [(64, 20), (30, 15), (24, 13), (16, 9)] {
            let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
            terminal
                .draw(|frame| ui::render(frame, &app, stories, &context, None))
                .unwrap();
            let buffer = terminal.backend().buffer();
            let first_row = (0..width)
                .map(|x| buffer.cell((x, 0)).unwrap().symbol())
                .collect::<String>();
            assert_eq!(
                first_row.starts_with("Needs "),
                !is_scene,
                "{} at {width}x{height}: inspect must reach the production scene",
                story.title
            );
        }
    }
}

#[test]
fn core_master_galleries_name_every_reviewed_archetype() {
    let stories = catalogue();
    for title in [
        "Assets / Core World Masters",
        "Assets / Core Portrait Masters",
    ] {
        let index = stories
            .iter()
            .position(|story| story.title == title)
            .expect("core gallery is catalogued");
        let mut app = StorybookApp::new(stories);
        app.select(index, stories);
        let backend = TestBackend::new(180, 38);
        let mut terminal = Terminal::new(backend).unwrap();
        terminal
            .draw(|frame| ui::render(frame, &app, stories, &StoryContext::fixed(), None))
            .unwrap();
        let buffer = terminal.backend().buffer();
        let screen = (0..38)
            .map(|y| {
                (0..180)
                    .map(|x| buffer.cell((x, y)).unwrap().symbol())
                    .collect::<String>()
            })
            .collect::<Vec<_>>()
            .join("\n");

        for class in [
            "Barbarian",
            "Bard",
            "Cleric",
            "Druid",
            "Paladin",
            "Ranger",
            "Rogue",
            "Wizard",
        ] {
            assert!(screen.contains(class), "{title} omits {class}");
        }
    }
}

#[test]
fn below_minimum_form_clears_scene_glyphs_behind_the_resize_message() {
    use questmancer::storybook::fixtures::{OwnedAuthoringState, owned_authoring_fixture};
    let model = owned_authoring_fixture(StoryContext::fixed(), OwnedAuthoringState::Small);
    let mut terminal = Terminal::new(TestBackend::new(27, 11)).unwrap();
    terminal
        .draw(|frame| {
            for cell in &mut frame.buffer_mut().content {
                cell.set_symbol("▀");
            }
            questmancer::ui::owned_authoring::render(frame, &model);
        })
        .unwrap();
    assert!(
        terminal
            .backend()
            .buffer()
            .content
            .iter()
            .all(|cell| cell.symbol() != "▀"),
        "scene glyphs leak into the status-only resize surface"
    );
}
