use questmancer::{
    app::Motion,
    domain::{Timestamp, WorkspaceId},
    scene::{
        heraldry::{CampaignCrest, CrestCharge, CrestField, table_pennants},
        snapshot::{SceneCampaign, SceneConnection, SceneSnapshot},
    },
};

fn snapshot(count: usize) -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: (0..count)
            .map(|i| SceneCampaign {
                workspace_id: WorkspaceId::new(format!("workspace-{i}")),
                label: format!("Campaign {i}"),
                variant_seed: 0,
            })
            .collect(),
        agents: Vec::new(),
        motion: Motion::None,
        now: Timestamp::from_millis(0),
    }
}

#[test]
fn crests_follow_workspace_keys_not_names_order_time_or_connection() {
    let mut scene = snapshot(8);
    let expected: Vec<_> = scene
        .campaigns
        .iter()
        .map(|c| {
            (
                c.workspace_id.clone(),
                CampaignCrest::for_workspace(&c.workspace_id),
            )
        })
        .collect();
    scene.campaigns.reverse();
    scene.now = Timestamp::from_millis(99_000);
    scene.connection = SceneConnection::Offline;
    for campaign in &mut scene.campaigns {
        campaign.label = "Renamed campaign".into();
        campaign.variant_seed = u64::MAX;
        assert_eq!(
            CampaignCrest::for_workspace(&campaign.workspace_id),
            expected
                .iter()
                .find(|(id, _)| *id == campaign.workspace_id)
                .unwrap()
                .1
        );
    }
}

#[test]
fn all_charges_remain_distinct_without_colour_and_have_ascii_names() {
    let charges = [
        CrestCharge::Lozenge,
        CrestCharge::Cross,
        CrestCharge::Saltire,
        CrestCharge::Bars,
    ];
    let mut frames = Vec::new();
    for charge in charges {
        let frame = CampaignCrest {
            field: CrestField::Azure,
            charge,
        }
        .frame();
        assert_eq!((frame.size().width, frame.size().height), (7, 8));
        assert!(frame.pixels().iter().flatten().count() > 40);
        assert!(charge.symbol(true).is_ascii());
        assert!(charge.name().is_ascii());
        assert!(
            !frames.contains(&frame),
            "shape must distinguish charges without colour"
        );
        frames.push(frame);
    }
    let identities: std::collections::HashSet<_> = (0..128)
        .map(|i| {
            let crest = CampaignCrest::for_workspace(&WorkspaceId::new(format!("campaign-{i}")));
            (crest.field.name(), crest.charge.name())
        })
        .collect();
    assert_eq!(
        identities.len(),
        16,
        "fixture keys exercise every field/charge pairing"
    );
}

#[test]
fn shared_table_pennants_are_separate_or_omitted_as_a_whole_set() {
    for count in 0..=12 {
        let scene = snapshot(count);
        let before = scene.clone();
        let pennants = table_pennants(&scene);
        assert_eq!(scene, before, "projection does not mutate live facts");
        for side in 0..2 {
            let expected: Vec<_> = scene
                .campaigns
                .iter()
                .skip(side)
                .step_by(2)
                .map(|c| c.workspace_id.clone())
                .collect();
            let actual: Vec<_> = pennants
                .iter()
                .filter(|p| (p.origin.x < 80) == (side == 0))
                .map(|p| p.workspace.clone())
                .collect();
            assert_eq!(
                actual,
                if expected.len() <= 4 {
                    expected
                } else {
                    Vec::new()
                }
            );
        }
        for (i, left) in pennants.iter().enumerate() {
            for right in pennants.iter().skip(i + 1) {
                assert!((left.origin.x - right.origin.x).abs() >= 7);
            }
        }
    }
}
