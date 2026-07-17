#![cfg(feature = "storybook")]

use questmancer::{
    app::{
        CharacterSet, ColorMode, ConnectionState, ConnectionStateKind, DisplayPreferences,
        GuildFocus, Modal, Motion, View,
    },
    domain::{AgentKey, GuildSummons, Presence, WorkspaceId},
    storybook::fixtures::{
        FIXED_NOW, StoryContext, campaign_fixture, campaign_token_fixture, compatibility_fixture,
        counsel_projection_fixture, delve_fixture, goblin_biscuit_id, goblin_chest_id,
        goblin_hand_id, goblin_scroll_id, great_room_fixture, great_room_one_campaign_fixture,
        great_room_reviewr_unavailable_fixture, great_room_scrying_failed_fixture,
        guild_connecting_fixture, guild_fixture, guild_incompatible_fixture,
        hearth_adventurer_fixture, library_id, modal_fixture, spoils_adventurer_fixture,
        undercroft_id, watchtower_id,
    },
    ui::{
        delve_scene::{DelveVariant, variant_for_campaign},
        goblins::{GoblinSighting, sighting_for_campaign},
        guild_room_projection::{
            AdventurerRepresentation, GuildLandmark, GuildLandmarkKind, GuildRoomMode,
            TruthfulStationKind, project,
        },
    },
};
use ratatui::layout::Rect;

#[test]
fn fixtures_are_value_deterministic() {
    let context = StoryContext::fixed();
    assert_eq!(guild_fixture(&context), guild_fixture(&context));
    assert_eq!(delve_fixture(&context), delve_fixture(&context));
    assert_eq!(guild_fixture(&context).view(), View::Guild);
    assert_eq!(delve_fixture(&context).view(), View::Delve);
    assert_eq!(guild_fixture(&context).modal(), &Modal::None);
}

#[test]
fn fixed_guild_fixtures_cover_every_connection_state() {
    let context = StoryContext::fixed();
    let cases = [
        (
            questmancer::storybook::fixtures::guild_disconnected_fixture(&context),
            ConnectionState::Offline,
        ),
        (
            guild_connecting_fixture(&context),
            ConnectionState::Connecting,
        ),
        (guild_fixture(&context), ConnectionState::Connected),
        (
            questmancer::storybook::fixtures::guild_reconnecting_fixture(&context),
            ConnectionState::Reconnecting { attempt: 3 },
        ),
        (
            guild_incompatible_fixture(&context),
            ConnectionState::Incompatible {
                expected: 17,
                actual: 16,
            },
        ),
    ];

    let mut kinds = Vec::new();
    for (model, expected) in cases {
        assert_eq!(model.connection(), &expected);
        assert_eq!(model.now(), FIXED_NOW);
        kinds.push(model.connection().kind());
    }
    assert_eq!(kinds, ConnectionStateKind::ALL);
}

#[test]
fn fixed_workspace_ids_lock_authored_variants() {
    assert_eq!(
        variant_for_campaign(&library_id()),
        DelveVariant::ForgottenLibrary
    );
    assert_eq!(
        variant_for_campaign(&undercroft_id()),
        DelveVariant::MossyUndercroft
    );
    assert_eq!(
        variant_for_campaign(&watchtower_id()),
        DelveVariant::OldWatchtower
    );
    assert_eq!(
        sighting_for_campaign(&goblin_chest_id()),
        Some(GoblinSighting::ChestEyes)
    );
    assert_eq!(
        sighting_for_campaign(&goblin_hand_id()),
        Some(GoblinSighting::ChronicleHand)
    );
    assert_eq!(
        sighting_for_campaign(&goblin_scroll_id()),
        Some(GoblinSighting::RaftersScroll)
    );
    assert_eq!(
        sighting_for_campaign(&goblin_biscuit_id()),
        Some(GoblinSighting::StolenBiscuit)
    );
}

#[test]
fn campaign_fixture_preserves_the_authored_party_order() {
    let workspace_id = WorkspaceId::new("reversed-campaign");
    let party = vec![AgentKey::new("zeta"), AgentKey::new("alpha")];
    let mut lexical = party.clone();
    lexical.sort();
    assert_ne!(
        party, lexical,
        "fixture input must expose accidental sorting"
    );

    let campaign = campaign_fixture(workspace_id.clone(), "Reversed Campaign", party.clone());

    assert_eq!(campaign.workspace_id, workspace_id);
    assert_eq!(campaign.label, "Reversed Campaign");
    assert_eq!(
        campaign.cwd.to_string_lossy(),
        "/storybook/reversed-campaign"
    );
    assert_eq!(campaign.party, party);
}

#[test]
fn guild_fixture_contains_the_complete_application_state() {
    let model = guild_fixture(&StoryContext::fixed());
    let domain = model.domain();
    let presences = domain
        .agents
        .values()
        .map(|agent| agent.presence)
        .collect::<Vec<_>>();

    for presence in [
        Presence::Working,
        Presence::Blocked,
        Presence::Done,
        Presence::Idle,
        Presence::Exited,
    ] {
        assert_eq!(
            presences
                .iter()
                .filter(|actual| **actual == presence)
                .count(),
            1
        );
    }
    assert_eq!(domain.agents.len(), 5);
    assert_eq!(domain.campaigns.len(), 3);
    assert_eq!(domain.chronicle.entries().len(), 5);
    assert_eq!(model.connection(), &ConnectionState::Connected);
    assert_eq!(model.now(), FIXED_NOW);

    let selected = model.selected_agent().expect("a selected adventurer");
    assert_eq!(selected.presence, Presence::Blocked);
    assert_eq!(
        selected.attention.summons(),
        Some(GuildSummons::CounselRequested)
    );

    let preview = model.output_preview().expect("a fixed output preview");
    assert_eq!(preview.pane_id, selected.pane_id);
    assert_eq!(preview.revision, selected.pane_revision);
    assert_eq!(
        preview.text,
        "Checking the local schema...\nAwaiting counsel at the sealed gate."
    );
    assert!(!preview.loading);
    assert_eq!(preview.error, None);
}

#[test]
fn modal_fixtures_use_the_public_editing_paths() {
    assert_eq!(modal_fixture(Modal::Help).modal(), &Modal::Help);
    assert_eq!(
        modal_fixture(Modal::Counsel {
            draft: String::new()
        })
        .modal(),
        &Modal::Counsel {
            draft: "Use the local schema".to_owned()
        }
    );
    assert_eq!(
        modal_fixture(Modal::Search {
            query: String::new()
        })
        .modal(),
        &Modal::Search {
            query: "Elowen".to_owned()
        }
    );
    assert_eq!(modal_fixture(Modal::None).modal(), &Modal::None);
}

#[test]
fn compatibility_fixture_applies_exact_preferences_to_a_delve() {
    let preferences = DisplayPreferences {
        motion: Motion::Reduced,
        character_set: CharacterSet::Ascii,
        color_mode: ColorMode::Ansi16,
    };

    let model = compatibility_fixture(preferences);

    assert_eq!(model.view(), View::Guild);
    assert_eq!(model.preferences(), &preferences);
}

#[test]
fn great_room_fixture_freezes_authored_campaigns_and_personas() {
    let context = StoryContext::fixed();
    let first = great_room_fixture(&context);
    let second = great_room_fixture(&context);

    assert_eq!(first, second);
    assert_eq!(first.now(), FIXED_NOW);
    assert_eq!(first.view(), View::Guild);
    assert_eq!(
        first
            .domain()
            .campaigns
            .values()
            .map(|campaign| campaign.label.as_str())
            .collect::<Vec<_>>(),
        ["Ironmere", "Saltwatch", "Moonfen"]
    );
    assert_eq!(
        great_room_one_campaign_fixture(&context)
            .domain()
            .campaigns
            .len(),
        1
    );
}

#[test]
fn great_room_failure_fixtures_preserve_actionable_semantics() {
    let reviewr = great_room_reviewr_unavailable_fixture(&StoryContext::fixed());
    assert!(!reviewr.reviewr_available());
    assert_eq!(reviewr.status_message(), Some("Reviewr is unavailable."));

    let scrying = great_room_scrying_failed_fixture(&StoryContext::fixed());
    assert_eq!(scrying.guild_focus(), GuildFocus::Scrying);
    assert_eq!(
        scrying
            .output_preview()
            .and_then(|preview| preview.error.as_deref()),
        Some("The scrying pool could not read this pane.")
    );
}

#[test]
fn truthful_station_fixtures_project_each_authored_representation() {
    let context = StoryContext::fixed();
    let cases = [
        (
            campaign_token_fixture(&context),
            "campaign token",
            GuildFocus::CampaignTables,
        ),
        (
            counsel_projection_fixture(&context),
            "counsel projection",
            GuildFocus::CounselBell,
        ),
        (
            hearth_adventurer_fixture(&context),
            "hearth adventurer",
            GuildFocus::Hearth,
        ),
        (
            spoils_adventurer_fixture(&context),
            "spoils adventurer",
            GuildFocus::Spoils,
        ),
    ];

    let mut kinds = Vec::new();
    for (model, label, focus) in cases {
        assert_eq!(model.guild_focus(), focus, "{label}");
        let projection = project(&model, Rect::new(0, 0, 78, 26));
        let representation = projection.adventurers.first().expect(label);
        kinds.push(
            representation
                .truthful_station_kind()
                .expect("authored fixture must use a truthful station"),
        );
        match (label, representation) {
            ("campaign token", AdventurerRepresentation::Token { .. })
            | (
                "counsel projection",
                AdventurerRepresentation::Projection {
                    station: GuildLandmark::CounselBell,
                    ..
                },
            )
            | (
                "hearth adventurer",
                AdventurerRepresentation::Physical {
                    station: GuildLandmark::Hearth,
                    ..
                },
            )
            | (
                "spoils adventurer",
                AdventurerRepresentation::Physical {
                    station: GuildLandmark::Spoils,
                    ..
                },
            ) => {}
            _ => panic!("{label} projected as {representation:?}"),
        }
    }
    assert_eq!(kinds, TruthfulStationKind::ALL);
}

#[test]
fn production_projection_enumerates_every_authored_landmark_and_room_mode() {
    let model = great_room_fixture(&StoryContext::fixed());
    let wide = project(&model, Rect::new(0, 0, 120, 36));
    let mut landmark_kinds = wide
        .landmarks
        .iter()
        .map(|landmark| landmark.landmark.kind())
        .collect::<Vec<_>>();
    landmark_kinds.extend(
        wide.campaigns
            .iter()
            .map(|_| GuildLandmarkKind::CampaignTable),
    );
    landmark_kinds.sort_unstable();
    landmark_kinds.dedup();
    assert_eq!(landmark_kinds, GuildLandmarkKind::ALL);

    let projected_modes =
        [120_u16, 80, 79].map(|width| project(&model, Rect::new(0, 0, width, 36)).mode);
    assert_eq!(projected_modes, GuildRoomMode::ALL);
}
