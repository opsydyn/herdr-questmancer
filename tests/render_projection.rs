#![cfg(feature = "storybook")]

use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, Region},
    domain::Presence,
    storybook::fixtures::{
        StoryContext, connected_delves_fixture, guild_populated_fixture, library_delve_fixture,
    },
    ui::{
        ChamberPresentation, GuildRegion, PersonaRenderMode, persona_render_mode_for_chamber,
        render_projection_for, theatre::TheatrePose, widgets::chamber_presentation,
    },
};
use ratatui::layout::Rect;

#[test]
fn chamber_projection_uses_the_production_full_and_compact_boundaries() {
    assert_eq!(
        chamber_presentation(Rect::new(0, 0, 28, 10)),
        ChamberPresentation::Full
    );
    assert_eq!(
        chamber_presentation(Rect::new(0, 0, 27, 10)),
        ChamberPresentation::CompactScene
    );
    assert_eq!(
        chamber_presentation(Rect::new(0, 0, 28, 9)),
        ChamberPresentation::CompactScene
    );
    assert_eq!(
        chamber_presentation(Rect::new(0, 0, 14, 7)),
        ChamberPresentation::Text
    );
    assert_eq!(
        chamber_presentation(Rect::new(0, 0, 14, 8)),
        ChamberPresentation::CompactScene
    );
}

#[test]
fn persona_projection_is_exact_at_unicode_ascii_and_departed_boundaries() {
    for character_set in [CharacterSet::Unicode, CharacterSet::Ascii] {
        assert_eq!(
            persona_render_mode_for_chamber(
                ChamberPresentation::Text,
                TheatrePose::Delving,
                character_set,
            ),
            PersonaRenderMode::None
        );
        assert_eq!(
            persona_render_mode_for_chamber(
                ChamberPresentation::CompactScene,
                TheatrePose::Delving,
                character_set,
            ),
            match character_set {
                CharacterSet::Unicode => PersonaRenderMode::Full,
                CharacterSet::Ascii => PersonaRenderMode::Silhouette,
            }
        );
        assert_eq!(
            persona_render_mode_for_chamber(
                ChamberPresentation::Full,
                TheatrePose::Departed,
                character_set,
            ),
            PersonaRenderMode::None
        );
    }
}

#[test]
fn departed_projection_never_claims_persona_art() {
    let mut model = connected_delves_fixture(&StoryContext::fixed());
    for agent in model.domain_mut().agents.values_mut() {
        agent.presence = Presence::Exited;
    }

    for (width, height) in [(130, 36), (60, 36), (60, 18)] {
        for character_set in [CharacterSet::Unicode, CharacterSet::Ascii] {
            model.set_preferences(DisplayPreferences {
                motion: Motion::Full,
                character_set,
                color_mode: ColorMode::Xterm256,
            });
            let projection = render_projection_for(&model, Rect::new(0, 0, width, height));
            assert!(
                projection
                    .visible_agents
                    .iter()
                    .all(|agent| agent.persona == PersonaRenderMode::None),
                "Departed {character_set:?} projection at {width}x{height} claimed persona art"
            );
        }
    }
}

#[test]
fn intermediate_delve_size_projects_a_full_chamber_missed_by_endpoints() {
    let model = library_delve_fixture(&StoryContext::fixed());
    let reference = render_projection_for(&model, Rect::new(0, 0, 130, 36));
    let minimum = render_projection_for(&model, Rect::new(0, 0, 60, 18));
    let intermediate = render_projection_for(&model, Rect::new(0, 0, 60, 36));

    assert!(
        reference
            .visible_agents
            .iter()
            .all(|agent| { agent.chamber == Some(ChamberPresentation::Text) })
    );
    assert!(
        minimum
            .visible_agents
            .iter()
            .all(|agent| { agent.chamber == Some(ChamberPresentation::Text) })
    );
    assert!(
        intermediate
            .visible_agents
            .iter()
            .any(|agent| { agent.chamber == Some(ChamberPresentation::Full) })
    );
}

#[test]
fn unicode_full_chambers_project_full_personas_while_ascii_projects_silhouettes() {
    let mut unicode = library_delve_fixture(&StoryContext::fixed());
    let unicode_projection = render_projection_for(&unicode, Rect::new(0, 0, 60, 36));
    assert!(
        unicode_projection
            .visible_agents
            .iter()
            .any(|agent| { agent.persona == PersonaRenderMode::Full })
    );

    unicode.set_preferences(DisplayPreferences {
        motion: Motion::Full,
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
    });
    let ascii_projection = render_projection_for(&unicode, Rect::new(0, 0, 60, 36));
    assert!(
        ascii_projection
            .visible_agents
            .iter()
            .all(|agent| { agent.persona != PersonaRenderMode::Full })
    );
    assert!(
        ascii_projection
            .visible_agents
            .iter()
            .any(|agent| { agent.persona == PersonaRenderMode::Silhouette })
    );
}

#[test]
fn guild_projection_owns_profile_visibility_at_responsive_boundaries() {
    let mut model = guild_populated_fixture(&StoryContext::fixed());
    let wide = render_projection_for(&model, Rect::new(0, 0, 120, 36));
    assert!(wide.guild_regions.contains(&GuildRegion::AdventurerProfile));
    assert!(wide.guild_profile_agent.is_some());

    let medium = render_projection_for(&model, Rect::new(0, 0, 80, 24));
    assert!(
        medium
            .guild_regions
            .contains(&GuildRegion::AdventurerProfile)
    );
    assert!(medium.guild_profile_agent.is_some());

    model.set_region(Region::QuestBoard);
    let focused = render_projection_for(&model, Rect::new(0, 0, 79, 24));
    assert_eq!(focused.guild_regions, [GuildRegion::QuestBoard].into());
    assert!(focused.guild_profile_agent.is_none());
}

#[test]
fn projected_chamber_rectangles_are_the_exact_rendered_agent_regions() {
    for (model, width, height) in [
        (connected_delves_fixture(&StoryContext::fixed()), 60, 18),
        (connected_delves_fixture(&StoryContext::fixed()), 60, 36),
        (connected_delves_fixture(&StoryContext::fixed()), 130, 36),
    ] {
        let projection = render_projection_for(&model, Rect::new(0, 0, width, height));
        let buffer = questmancer::storybook::ui::render_application_buffer(&model, width, height);
        for projected in &projection.visible_agents {
            let area = projected
                .chamber_area
                .expect("every projected Delve agent must retain its chamber rectangle");
            let name = &model.domain().agents.get(&projected.key).unwrap().name;
            assert!(
                rect_text(&buffer, area).contains(name),
                "projected chamber {area:?} did not contain {name} at {width}x{height}"
            );
        }
    }
}

#[test]
fn connected_and_active_delve_regions_match_the_rendered_variant_and_selection() {
    let mut model = connected_delves_fixture(&StoryContext::fixed());
    for (width, expected_multiple) in [(80, false), (130, true)] {
        let projection = render_projection_for(&model, Rect::new(0, 0, width, 36));
        assert_eq!(projection.delve_regions.len() > 1, expected_multiple);
        let buffer = questmancer::storybook::ui::render_application_buffer(&model, width, 36);
        for delve in &projection.delve_regions {
            assert!(
                rect_text(&buffer, delve.area).contains(variant_marker(delve.variant)),
                "projected {:?} architecture was not rendered in {:?}:\n{}",
                delve.variant,
                delve.area,
                rect_text(&buffer, delve.area)
            );
        }
    }

    let initial_workspace = model.selected_agent().unwrap().workspace_id.clone();
    let selected_key = model
        .domain()
        .agents
        .values()
        .find(|agent| agent.workspace_id != initial_workspace)
        .unwrap()
        .key
        .clone();
    model.domain_mut().selected_agent = Some(selected_key);
    let selected = render_projection_for(&model, Rect::new(0, 0, 80, 36));
    assert_eq!(selected.delve_regions.len(), 1);
    let selected_workspace = model.selected_agent().unwrap().workspace_id.clone();
    assert_eq!(selected.delve_regions[0].workspace_id, selected_workspace);
    assert!(selected.delve_regions[0].active);
}

fn rect_text(buffer: &ratatui::buffer::Buffer, area: Rect) -> String {
    (area.y..area.bottom())
        .flat_map(|y| (area.x..area.right()).map(move |x| buffer.cell((x, y)).unwrap().symbol()))
        .collect()
}

const fn variant_marker(variant: questmancer::ui::delve_scene::DelveVariant) -> &'static str {
    use questmancer::ui::delve_scene::DelveVariant;
    match variant {
        DelveVariant::ForgottenLibrary => "READING ALCOVE / CONNECTING ARCH",
        DelveVariant::MossyUndercroft => "CAMP JUNCTION / DESCENDING PASSAGE",
        DelveVariant::OldWatchtower => "STAIR / NARROW LANDING",
    }
}
