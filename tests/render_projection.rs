#![cfg(feature = "storybook")]

use questmancer::{
    app::{CharacterSet, ColorMode, DisplayPreferences, Motion, Region},
    storybook::fixtures::{
        StoryContext, connected_delves_fixture, guild_populated_fixture, library_delve_fixture,
    },
    ui::{
        ChamberPresentation, GuildRegion, PersonaRenderMode, render_projection_for,
        widgets::chamber_presentation,
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
        chamber_presentation(Rect::new(0, 0, 13, 5)),
        ChamberPresentation::Text
    );
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
            .all(|agent| { agent.chamber == Some(ChamberPresentation::CompactScene) })
    );
    assert!(
        minimum
            .visible_agents
            .iter()
            .all(|agent| { agent.chamber == Some(ChamberPresentation::CompactScene) })
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
    let mut unicode = connected_delves_fixture(&StoryContext::fixed());
    let unicode_projection = render_projection_for(&unicode, Rect::new(0, 0, 130, 36));
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
    let ascii_projection = render_projection_for(&unicode, Rect::new(0, 0, 130, 36));
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
