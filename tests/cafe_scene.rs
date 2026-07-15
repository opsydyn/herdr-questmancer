use std::collections::BTreeMap;

#[allow(dead_code)]
mod support;

use proptest::prelude::*;
use questmancer::{
    domain::{Agent, AgentKey, Campaign, WorkspaceId},
    ui::cafe_scene::{BayVariant, layout_bays, variant_for_workspace},
};
use ratatui::layout::Rect;

fn site(id: &str, agent_ids: &[&str]) -> Campaign {
    Campaign {
        workspace_id: WorkspaceId::new(id),
        label: id.to_owned(),
        cwd: "/tmp".into(),
        party: agent_ids.iter().map(|id| AgentKey::new(*id)).collect(),
    }
}

#[test]
fn variants_are_deterministic_and_cover_all_authored_variants() {
    let ids = (0..128)
        .map(|index| WorkspaceId::new(format!("workspace-{index}")))
        .collect::<Vec<_>>();
    let variants = ids.iter().map(variant_for_workspace).collect::<Vec<_>>();

    for id in &ids {
        assert_eq!(variant_for_workspace(id), variant_for_workspace(id));
    }
    assert!(variants.contains(&BayVariant::WallRow));
    assert!(variants.contains(&BayVariant::CornerBooth));
    assert!(variants.contains(&BayVariant::BackRoomLab));
}

#[test]
fn bays_are_sorted_by_workspace_id_and_seats_fit_the_scene() {
    let sites = BTreeMap::from([
        (WorkspaceId::new("zeta"), site("zeta", &["z1", "z2"])),
        (WorkspaceId::new("alpha"), site("alpha", &["a1"])),
    ]);
    let agents = BTreeMap::new();
    let area = Rect::new(0, 0, 120, 30);
    let bays = layout_bays(&sites, &agents, area, None);

    assert_eq!(
        bays.iter().map(|bay| &bay.workspace_id).collect::<Vec<_>>(),
        [&WorkspaceId::new("alpha"), &WorkspaceId::new("zeta")]
    );
    for bay in bays {
        for seat in bay.seats {
            assert!(seat.x >= bay.rect.x);
            assert!(seat.y >= bay.rect.y);
            assert!(u32::from(seat.x) + u32::from(seat.width) <= u32::from(bay.rect.right()));
            assert!(u32::from(seat.y) + u32::from(seat.height) <= u32::from(bay.rect.bottom()));
        }
    }
}

#[test]
fn tiny_scene_exposes_bays_without_duplicate_seats() {
    let sites = BTreeMap::from([
        (
            WorkspaceId::new("alpha"),
            site("alpha", &["a1", "a2", "a3"]),
        ),
        (WorkspaceId::new("beta"), site("beta", &["b1", "b2"])),
    ]);
    let agents = BTreeMap::new();
    let bays = layout_bays(&sites, &agents, Rect::new(0, 0, 1, 1), None);

    assert_eq!(bays.len(), 2);
    assert!(bays.iter().all(|bay| bay.seats.is_empty()));
}

#[test]
fn overflowing_workspace_is_split_into_connected_bays_without_losing_agents() {
    let keys = (0..11)
        .map(|index| AgentKey::new(format!("a{index}")))
        .collect::<Vec<_>>();
    let workspace = WorkspaceId::new("overflow");
    let sites = BTreeMap::from([(
        workspace.clone(),
        Campaign {
            workspace_id: workspace.clone(),
            label: "overflow".into(),
            cwd: "/tmp".into(),
            party: keys.clone(),
        },
    )]);
    let template = support::fixture_domain()
        .agents
        .values()
        .next()
        .unwrap()
        .clone();
    let agents = keys
        .iter()
        .cloned()
        .map(|key| {
            let mut agent = template.clone();
            agent.key = key.clone();
            (key, agent)
        })
        .collect::<BTreeMap<_, _>>();
    let bays = layout_bays(&sites, &agents, Rect::new(0, 0, 120, 40), None);
    assert!(bays.len() > 1);
    let assigned = bays
        .iter()
        .flat_map(|bay| bay.agent_keys.iter().cloned())
        .collect::<Vec<_>>();
    assert_eq!(assigned.len(), keys.len());
    assert_eq!(
        assigned
            .iter()
            .collect::<std::collections::BTreeSet<_>>()
            .len(),
        keys.len()
    );
}

#[allow(clippy::similar_names)]
fn overlaps(
    left: questmancer::ui::cafe_scene::SeatAnchor,
    right: questmancer::ui::cafe_scene::SeatAnchor,
) -> bool {
    let x = |seat: &questmancer::ui::cafe_scene::SeatAnchor| {
        (u32::from(seat.x), u32::from(seat.x) + u32::from(seat.width))
    };
    let y = |seat: &questmancer::ui::cafe_scene::SeatAnchor| {
        (
            u32::from(seat.y),
            u32::from(seat.y) + u32::from(seat.height),
        )
    };
    let (left_x0, left_x1) = x(&left);
    let (left_y0, left_y1) = y(&left);
    let (right_x0, right_x1) = x(&right);
    let (right_y0, right_y1) = y(&right);
    left_x0 < right_x1 && right_x0 < left_x1 && left_y0 < right_y1 && right_y0 < left_y1
}

proptest! {
    #[test]
    fn generated_bays_are_stable_and_non_overlapping(
        workspace in "[a-z0-9_-]{1,16}",
        count in 0usize..=12,
        width in 0u16..=8,
        height in 0u16..=8,
    ) {
        let workspace_id = WorkspaceId::new(workspace);
        let mut site_agents = Vec::new();
        let mut agents = BTreeMap::new();
        let template = support::fixture_domain().agents.values().next().unwrap().clone();
        for index in 0..count {
            let mut agent: Agent = template.clone();
            agent.key = AgentKey::new(format!("agent-{index}"));
            agent.workspace_id = workspace_id.clone();
            site_agents.push(agent.key.clone());
            agents.insert(agent.key.clone(), agent);
        }
        let sites = BTreeMap::from([(workspace_id.clone(), Campaign {
            workspace_id: workspace_id.clone(),
            label: workspace_id.to_string(),
            cwd: "/tmp".into(),
            party: site_agents,
        })]);
        let area = Rect::new(0, 0, width, height);
        let first = layout_bays(&sites, &agents, area, None);
        let second = layout_bays(&sites, &agents, area, None);
        prop_assert_eq!(&first, &second);
        prop_assume!(!first.is_empty());
        let seats = &first[0].seats;
        for (index, seat) in seats.iter().enumerate() {
            for other in seats.iter().skip(index + 1) {
                prop_assert!(!overlaps(*seat, *other));
            }
        }
    }
}
