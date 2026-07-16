#![cfg(feature = "storybook")]

use questmancer::{
    app::{CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Modal, Motion, View},
    domain::{AgentKey, GuildSummons, Presence, WorkspaceId},
    storybook::fixtures::{
        FIXED_NOW, StoryContext, campaign_fixture, compatibility_fixture, delve_fixture,
        goblin_biscuit_id, goblin_chest_id, goblin_hand_id, goblin_scroll_id, guild_fixture,
        library_id, modal_fixture, undercroft_id, watchtower_id,
    },
    ui::{
        delve_scene::{DelveVariant, variant_for_campaign},
        goblins::{GoblinSighting, sighting_for_campaign},
    },
};

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
    let workspace_id = WorkspaceId::new("lexical-campaign");
    let party = vec![AgentKey::new("alpha"), AgentKey::new("zeta")];

    let campaign = campaign_fixture(workspace_id.clone(), "Lexical Campaign", party.clone());

    assert_eq!(campaign.workspace_id, workspace_id);
    assert_eq!(campaign.label, "Lexical Campaign");
    assert_eq!(
        campaign.cwd.to_string_lossy(),
        "/storybook/lexical-campaign"
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

    assert_eq!(model.view(), View::Delve);
    assert_eq!(model.preferences(), &preferences);
}
