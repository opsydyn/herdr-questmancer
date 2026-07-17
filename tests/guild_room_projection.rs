use std::path::PathBuf;

use questmancer::{
    app::{GuildFocus, Model, View},
    domain::{
        Agent, AgentKey, Campaign, DomainState, GuildAttention, GuildSummons, Presence, Timestamp,
        WorkspaceId,
    },
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::guild_room_projection::{
        AdventurerRepresentation, GuildLandmark, GuildRoomMode, GuildRoomProjection, project,
    },
};
use ratatui::layout::Rect;

fn campaign(id: &str, label: &str, cwd: impl Into<PathBuf>) -> Campaign {
    Campaign {
        workspace_id: WorkspaceId::new(id),
        label: label.to_owned(),
        cwd: cwd.into(),
        party: Vec::new(),
    }
}

fn model_with_campaigns(campaigns: impl IntoIterator<Item = Campaign>) -> Model {
    let mut domain = DomainState::default();
    for campaign in campaigns {
        domain
            .campaigns
            .insert(campaign.workspace_id.clone(), campaign);
    }
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model
}

fn fixture_agent() -> Agent {
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000))
        .agents
        .into_values()
        .next()
        .expect("fixture contains an adventurer")
}

fn model_with_agent(mut agent: Agent) -> Model {
    agent.workspace_id = WorkspaceId::new("campaign-alpha");
    let agent_key = agent.key.clone();
    let mut owning_campaign = campaign("campaign-alpha", "Alpha", "/work/alpha");
    owning_campaign.party.push(agent_key.clone());
    let mut domain = DomainState::default();
    domain
        .campaigns
        .insert(agent.workspace_id.clone(), owning_campaign);
    domain.selected_agent = Some(agent_key.clone());
    domain.agents.insert(agent_key, agent);
    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    model
}

#[test]
fn presence_and_local_attention_derive_the_exact_truthful_station() {
    let since = Timestamp::from_millis(1_000);
    let cases = [
        (Presence::Exited, GuildAttention::Clear, None),
        (
            Presence::Blocked,
            GuildAttention::Clear,
            Some(AdventurerRepresentation::Projection {
                agent: AgentKey::new("agent-alpha"),
                station: GuildLandmark::CounselBell,
            }),
        ),
        (
            Presence::Done,
            GuildAttention::unread(GuildSummons::SpoilsReturned, since),
            Some(AdventurerRepresentation::Physical {
                agent: AgentKey::new("agent-alpha"),
                station: GuildLandmark::Spoils,
            }),
        ),
        (
            Presence::Idle,
            GuildAttention::Clear,
            Some(AdventurerRepresentation::Physical {
                agent: AgentKey::new("agent-alpha"),
                station: GuildLandmark::Hearth,
            }),
        ),
        (
            Presence::Working,
            GuildAttention::Clear,
            Some(AdventurerRepresentation::Token {
                agent: AgentKey::new("agent-alpha"),
                table: WorkspaceId::new("campaign-alpha"),
            }),
        ),
        (
            Presence::Done,
            GuildAttention::Read {
                summons: GuildSummons::SpoilsReturned,
                since,
            },
            Some(AdventurerRepresentation::Token {
                agent: AgentKey::new("agent-alpha"),
                table: WorkspaceId::new("campaign-alpha"),
            }),
        ),
        (
            Presence::Done,
            GuildAttention::unread(GuildSummons::CounselRequested, since),
            Some(AdventurerRepresentation::Token {
                agent: AgentKey::new("agent-alpha"),
                table: WorkspaceId::new("campaign-alpha"),
            }),
        ),
        (
            Presence::Done,
            GuildAttention::unread(GuildSummons::AdventurerDeparted, since),
            Some(AdventurerRepresentation::Token {
                agent: AgentKey::new("agent-alpha"),
                table: WorkspaceId::new("campaign-alpha"),
            }),
        ),
        (
            Presence::Unknown,
            GuildAttention::Clear,
            Some(AdventurerRepresentation::Token {
                agent: AgentKey::new("agent-alpha"),
                table: WorkspaceId::new("campaign-alpha"),
            }),
        ),
    ];

    for (presence, attention, expected) in cases {
        let mut agent = fixture_agent();
        agent.key = AgentKey::new("agent-alpha");
        agent.presence = presence;
        agent.attention = attention;
        let projection = project(&model_with_agent(agent), Rect::new(0, 0, 120, 30));

        assert_eq!(
            projection.adventurers,
            expected.into_iter().collect::<Vec<_>>(),
            "unexpected representation for {presence:?}"
        );
    }
}

#[test]
fn focused_agent_changes_only_room_presentation_not_station_or_geometry() {
    let mut agent = fixture_agent();
    agent.key = AgentKey::new("agent-alpha");
    agent.presence = Presence::Working;
    agent.attention = GuildAttention::Clear;
    agent.focused = false;
    let calm = project(&model_with_agent(agent.clone()), Rect::new(0, 0, 120, 30));

    agent.focused = true;
    let focused = project(&model_with_agent(agent), Rect::new(0, 0, 120, 30));

    assert_eq!(calm.adventurers, focused.adventurers);
    assert_eq!(
        stable_campaign_identity(&calm),
        stable_campaign_identity(&focused)
    );
    assert!(!calm.campaigns[0].illuminated);
    assert!(focused.campaigns[0].illuminated);
    let scrying_illumination = |projection: &GuildRoomProjection| {
        projection
            .landmarks
            .iter()
            .find(|landmark| landmark.landmark == GuildLandmark::Scrying)
            .expect("Scrying is a stable landmark")
            .illuminated
    };
    assert!(!scrying_illumination(&calm));
    assert!(scrying_illumination(&focused));
}

#[test]
fn room_mode_changes_at_the_exact_camera_widths() {
    let model = Model::new(View::Guild);
    let cases = [
        (79, GuildRoomMode::LandmarkCamera),
        (80, GuildRoomMode::CroppedRoom),
        (119, GuildRoomMode::CroppedRoom),
        (120, GuildRoomMode::WholeRoom),
    ];

    for (width, expected) in cases {
        assert_eq!(
            project(&model, Rect::new(0, 0, width, 24)).mode,
            expected,
            "unexpected room mode at width {width}"
        );
    }
}

#[test]
fn cropped_room_centres_selected_campaign_and_keeps_shared_landmarks() {
    let mut model = model_with_campaigns([
        campaign("alpha", "Alpha", "/work/alpha"),
        campaign("beta", "Beta", "/work/beta"),
        campaign("gamma", "Gamma", "/work/gamma"),
    ]);
    let mut agent = fixture_agent();
    agent.key = AgentKey::new("agent-beta");
    agent.workspace_id = WorkspaceId::new("beta");
    model.domain_mut().agents.insert(agent.key.clone(), agent);
    model.domain_mut().selected_agent = Some(AgentKey::new("agent-beta"));

    let projection = project(&model, Rect::new(0, 0, 100, 24));

    assert_eq!(projection.mode, GuildRoomMode::CroppedRoom);
    assert_eq!(projection.focused, GuildFocus::QuestWall);
    assert_eq!(projection.campaigns.len(), 3);
    let selected = projection
        .campaigns
        .iter()
        .find(|campaign| campaign.selected)
        .unwrap();
    assert_eq!(selected.workspace_id, WorkspaceId::new("beta"));
    assert!(
        projection
            .campaigns
            .iter()
            .filter(|campaign| !campaign.selected)
            .all(|campaign| !campaign.area.is_empty())
    );
    for required in [
        GuildLandmark::Door,
        GuildLandmark::QuestWall,
        GuildLandmark::Hearth,
        GuildLandmark::Scrying,
    ] {
        assert!(
            projection
                .landmarks
                .iter()
                .any(|landmark| landmark.landmark == required && !landmark.area.is_empty()),
            "missing {required:?}"
        );
    }
    assert_eq!(projection.breadcrumb, None);
    assert_projected_areas_fit_without_overlap(&projection, Rect::new(0, 0, 100, 24));
}

#[test]
fn landmark_camera_projects_one_focused_landmark_and_room_breadcrumb() {
    let mut model = model_with_campaigns([
        campaign("alpha", "Alpha", "/work/alpha"),
        campaign("beta", "Beta", "/work/beta"),
    ]);
    model.set_guild_focus(GuildFocus::Scrying);

    let projection = project(&model, Rect::new(0, 0, 79, 24));

    assert_eq!(projection.mode, GuildRoomMode::LandmarkCamera);
    assert_eq!(projection.focused, GuildFocus::Scrying);
    assert_eq!(
        projection.breadcrumb.as_deref(),
        Some("GREAT ROOM / SCRYING")
    );
    assert_eq!(
        projection
            .landmarks
            .iter()
            .filter(|landmark| !landmark.area.is_empty())
            .map(|landmark| landmark.landmark.clone())
            .collect::<Vec<_>>(),
        [GuildLandmark::Scrying]
    );
    assert!(
        projection
            .campaigns
            .iter()
            .all(|campaign| campaign.area.is_empty())
    );
}

#[test]
fn campaign_table_camera_preserves_all_identities_but_shows_only_selected_table() {
    let mut model = model_with_campaigns([
        campaign("alpha", "Alpha", "/work/alpha"),
        campaign("beta", "Beta", "/work/beta"),
    ]);
    let mut agent = fixture_agent();
    agent.key = AgentKey::new("agent-beta");
    agent.workspace_id = WorkspaceId::new("beta");
    let agent_key = agent.key.clone();
    model.domain_mut().agents.insert(agent_key.clone(), agent);
    model.domain_mut().selected_agent = Some(agent_key);
    model.set_guild_focus(GuildFocus::CampaignTables);

    let projection = project(&model, Rect::new(0, 0, 60, 18));

    assert_eq!(projection.campaigns.len(), 2);
    assert_eq!(
        projection
            .campaigns
            .iter()
            .filter(|campaign| !campaign.area.is_empty())
            .map(|campaign| campaign.workspace_id.as_str())
            .collect::<Vec<_>>(),
        ["beta"]
    );
    assert_eq!(
        projection.breadcrumb.as_deref(),
        Some("GREAT ROOM / CAMPAIGN TABLES")
    );
}

#[test]
fn campaign_labels_use_meaningful_label_then_checkout_then_workspace_id() {
    let model = model_with_campaigns([
        campaign("alpha", "  Ironmere  ", "/work/ignored-checkout"),
        campaign("blank", "", "/work/Moonfen"),
        campaign("space", " \t ", "/work/Saltwatch"),
        campaign("tilde-id", "~", PathBuf::new()),
    ]);

    let projection = project(&model, Rect::new(0, 0, 120, 30));
    let labels = projection
        .campaigns
        .iter()
        .map(|campaign| (campaign.workspace_id.as_str(), campaign.label.as_str()))
        .collect::<Vec<_>>();

    assert_eq!(
        labels,
        [
            ("alpha", "Ironmere"),
            ("blank", "Moonfen"),
            ("space", "Saltwatch"),
            ("tilde-id", "tilde-id"),
        ]
    );
}

#[test]
fn campaign_identity_and_geometry_are_deterministic_for_multiple_workspaces() {
    let model = model_with_campaigns([
        campaign("zeta", "Zeta", "/work/zeta"),
        campaign("alpha", "Alpha", "/work/alpha"),
        campaign("middle", "Middle", "/work/middle"),
    ]);
    let area = Rect::new(7, 11, 120, 30);

    let first = project(&model, area);
    let second = project(&model, area);

    assert_eq!(first, second);
    assert_eq!(
        first
            .campaigns
            .iter()
            .map(|campaign| campaign.workspace_id.as_str())
            .collect::<Vec<_>>(),
        ["alpha", "middle", "zeta"]
    );
    assert!(
        first
            .campaigns
            .iter()
            .map(|campaign| campaign.seal)
            .all(|seal| seal != 0)
    );
    assert_eq!(
        first
            .campaigns
            .iter()
            .map(|campaign| campaign.seal)
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        first.campaigns.len()
    );
    assert_projected_areas_fit_without_overlap(&first, area);
}

#[test]
fn zero_and_tiny_areas_are_safe_and_saturating() {
    let model = model_with_campaigns([
        campaign("alpha", "Alpha", "/work/alpha"),
        campaign("beta", "Beta", "/work/beta"),
    ]);

    for area in [
        Rect::new(0, 0, 0, 0),
        Rect::new(9, 13, 0, 1),
        Rect::new(9, 13, 1, 0),
        Rect::new(u16::MAX - 1, u16::MAX - 1, 1, 1),
    ] {
        let projection = project(&model, area);
        assert_projected_areas_fit_without_overlap(&projection, area);
    }
}

#[test]
fn selection_changes_only_campaign_presentation() {
    let mut domain = DomainState::default();
    let response: SuccessResponse<SessionSnapshotResult> =
        serde_json::from_str(include_str!("fixtures/herdr/session_snapshot.json")).unwrap();
    let mut alpha =
        DomainState::from_snapshot(&response.result.snapshot, Timestamp::from_millis(1_000))
            .agents
            .into_values()
            .next()
            .expect("fixture contains an adventurer");
    alpha.key = AgentKey::new("agent-alpha");
    alpha.workspace_id = WorkspaceId::new("alpha");
    let mut beta = alpha.clone();
    beta.key = AgentKey::new("agent-beta");
    beta.workspace_id = WorkspaceId::new("beta");

    for (id, label, agent) in [("alpha", "Alpha", &alpha), ("beta", "Beta", &beta)] {
        domain.campaigns.insert(
            WorkspaceId::new(id),
            Campaign {
                workspace_id: WorkspaceId::new(id),
                label: label.to_owned(),
                cwd: PathBuf::from(format!("/work/{id}")),
                party: vec![agent.key.clone()],
            },
        );
    }
    domain.agents.insert(alpha.key.clone(), alpha.clone());
    domain.agents.insert(beta.key.clone(), beta.clone());
    domain.selected_agent = Some(alpha.key.clone());

    let mut model = Model::new(View::Guild);
    model.replace_domain(domain);
    let area = Rect::new(0, 0, 120, 30);
    let alpha_selected = project(&model, area);

    model.domain_mut().selected_agent = Some(beta.key);
    let beta_selected = project(&model, area);

    let stable_identity =
        |projection: &questmancer::ui::guild_room_projection::GuildRoomProjection| {
            projection
                .campaigns
                .iter()
                .map(|campaign| {
                    (
                        campaign.workspace_id.clone(),
                        campaign.label.clone(),
                        campaign.seal,
                        campaign.area,
                    )
                })
                .collect::<Vec<_>>()
        };
    assert_eq!(
        stable_identity(&alpha_selected),
        stable_identity(&beta_selected)
    );
    assert_eq!(
        alpha_selected
            .campaigns
            .iter()
            .find(|campaign| campaign.selected)
            .map(|campaign| campaign.workspace_id.as_str()),
        Some("alpha")
    );
    assert_eq!(
        beta_selected
            .campaigns
            .iter()
            .find(|campaign| campaign.selected)
            .map(|campaign| campaign.workspace_id.as_str()),
        Some("beta")
    );
}

fn assert_projected_areas_fit_without_overlap(
    projection: &questmancer::ui::guild_room_projection::GuildRoomProjection,
    area: Rect,
) {
    let projected = projection
        .landmarks
        .iter()
        .map(|landmark| landmark.area)
        .chain(projection.campaigns.iter().map(|campaign| campaign.area))
        .collect::<Vec<_>>();

    for child in &projected {
        assert!(child.x >= area.x);
        assert!(child.y >= area.y);
        assert!(child.right() <= area.right());
        assert!(child.bottom() <= area.bottom());
    }
    for (index, left) in projected.iter().enumerate() {
        for right in projected.iter().skip(index + 1) {
            assert!(
                !rectangles_overlap(*left, *right),
                "projected areas overlap: {left:?} and {right:?}"
            );
        }
    }
}

fn stable_campaign_identity(
    projection: &GuildRoomProjection,
) -> Vec<(WorkspaceId, String, u64, Rect)> {
    projection
        .campaigns
        .iter()
        .map(|campaign| {
            (
                campaign.workspace_id.clone(),
                campaign.label.clone(),
                campaign.seal,
                campaign.area,
            )
        })
        .collect()
}

fn rectangles_overlap(left: Rect, right: Rect) -> bool {
    left.x < right.right()
        && right.x < left.right()
        && left.y < right.bottom()
        && right.y < left.bottom()
}
