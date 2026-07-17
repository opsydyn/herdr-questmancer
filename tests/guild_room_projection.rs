use std::path::PathBuf;

use questmancer::{
    app::{Model, View},
    domain::{AgentKey, Campaign, DomainState, Timestamp, WorkspaceId},
    herdr::protocol::{SessionSnapshotResult, SuccessResponse},
    ui::guild_room_projection::{GuildRoomMode, project},
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

fn rectangles_overlap(left: Rect, right: Rect) -> bool {
    left.x < right.right()
        && right.x < left.right()
        && left.y < right.bottom()
        && right.y < left.bottom()
}
