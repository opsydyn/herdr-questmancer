use std::collections::BTreeMap;

#[allow(dead_code)]
mod support;

use proptest::prelude::*;
use questmancer::{
    domain::{Agent, AgentKey, Campaign, WorkspaceId},
    ui::delve_scene::{DelveVariant, layout_delves, variant_for_campaign},
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
    let variants = ids.iter().map(variant_for_campaign).collect::<Vec<_>>();

    for id in &ids {
        assert_eq!(variant_for_campaign(id), variant_for_campaign(id));
    }
    assert!(variants.contains(&DelveVariant::ForgottenLibrary));
    assert!(variants.contains(&DelveVariant::MossyUndercroft));
    assert!(variants.contains(&DelveVariant::OldWatchtower));
}

#[test]
fn delves_are_sorted_non_overlapping_and_chambers_fit_the_scene() {
    let sites = BTreeMap::from([
        (WorkspaceId::new("zeta"), site("zeta", &["z1", "z2"])),
        (WorkspaceId::new("alpha"), site("alpha", &["a1"])),
    ]);
    let agents = BTreeMap::new();
    let area = Rect::new(0, 0, 120, 30);
    let delves = layout_delves(&sites, &agents, area, None);

    assert_eq!(
        delves
            .iter()
            .map(|delve| &delve.workspace_id)
            .collect::<Vec<_>>(),
        [&WorkspaceId::new("alpha"), &WorkspaceId::new("zeta")]
    );
    for (index, left) in delves.iter().enumerate() {
        for right in delves.iter().skip(index + 1) {
            let overlaps = left.rect.x < right.rect.right()
                && right.rect.x < left.rect.right()
                && left.rect.y < right.rect.bottom()
                && right.rect.y < left.rect.bottom();
            assert!(!overlaps, "Delves must not overlap: {left:?} and {right:?}");
        }
        for chamber in &left.chambers {
            assert!(chamber.x >= left.rect.x);
            assert!(chamber.y >= left.rect.y);
            assert!(
                u32::from(chamber.x) + u32::from(chamber.width) <= u32::from(left.rect.right())
            );
            assert!(
                u32::from(chamber.y) + u32::from(chamber.height) <= u32::from(left.rect.bottom())
            );
        }
    }
}

#[test]
fn connected_layout_allocates_complete_compact_chambers_when_room_allows() {
    let workspace = WorkspaceId::new("connected");
    let template = support::fixture_domain()
        .agents
        .into_values()
        .next()
        .unwrap();
    let agents = ["a1", "a2"]
        .into_iter()
        .map(|id| {
            let mut agent = template.clone();
            agent.key = AgentKey::new(id);
            agent.workspace_id = workspace.clone();
            (agent.key.clone(), agent)
        })
        .collect::<BTreeMap<_, _>>();
    let campaign = Campaign {
        workspace_id: workspace.clone(),
        label: "Connected".to_owned(),
        cwd: "/tmp/connected".into(),
        party: agents.keys().cloned().collect(),
    };
    let delves = layout_delves(
        &BTreeMap::from([(workspace, campaign)]),
        &agents,
        Rect::new(0, 0, 120, 30),
        None,
    );

    assert_eq!(delves.len(), 1);
    assert_eq!(delves[0].chambers.len(), 2);
    assert!(
        delves[0].chambers.iter().all(|chamber| chamber.height == 8),
        "connected chambers with sufficient room must allocate all eight rows: {:?}",
        delves[0].chambers
    );
}

#[test]
fn tiny_scene_exposes_delves_without_duplicate_chambers() {
    let sites = BTreeMap::from([
        (
            WorkspaceId::new("alpha"),
            site("alpha", &["a1", "a2", "a3"]),
        ),
        (WorkspaceId::new("beta"), site("beta", &["b1", "b2"])),
    ]);
    let agents = BTreeMap::new();
    let delves = layout_delves(&sites, &agents, Rect::new(0, 0, 1, 1), None);

    assert_eq!(delves.len(), 2);
    assert!(delves.iter().all(|delve| delve.chambers.is_empty()));
}

#[test]
fn overflowing_campaign_is_split_into_connected_delves_without_losing_adventurers() {
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
    let delves = layout_delves(&sites, &agents, Rect::new(0, 0, 120, 40), None);
    assert!(delves.len() > 1);
    let assigned = delves
        .iter()
        .flat_map(|delve| delve.adventurers.iter().cloned())
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
    left: questmancer::ui::delve_scene::ChamberAnchor,
    right: questmancer::ui::delve_scene::ChamberAnchor,
) -> bool {
    let x = |chamber: &questmancer::ui::delve_scene::ChamberAnchor| {
        (
            u32::from(chamber.x),
            u32::from(chamber.x) + u32::from(chamber.width),
        )
    };
    let y = |chamber: &questmancer::ui::delve_scene::ChamberAnchor| {
        (
            u32::from(chamber.y),
            u32::from(chamber.y) + u32::from(chamber.height),
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
    fn generated_delves_are_stable_and_non_overlapping(
        workspace in "[a-z0-9_-]{1,16}",
        count in 0usize..=12,
        x in 0u16..=40,
        y in 0u16..=40,
        width in 0u16..=160,
        height in 0u16..=60,
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
        let area = Rect::new(x, y, width, height);
        let first = layout_delves(&sites, &agents, area, None);
        let second = layout_delves(&sites, &agents, area, None);
        prop_assert_eq!(&first, &second);
        prop_assume!(!first.is_empty());
        for (delve_index, delve) in first.iter().enumerate() {
            for other_delve in first.iter().skip(delve_index + 1) {
                let delve_overlaps = delve.rect.x < other_delve.rect.right()
                    && other_delve.rect.x < delve.rect.right()
                    && delve.rect.y < other_delve.rect.bottom()
                    && other_delve.rect.y < delve.rect.bottom();
                prop_assert!(!delve_overlaps, "Delves overlap: {delve:?} and {other_delve:?}");
            }
            for (index, chamber) in delve.chambers.iter().enumerate() {
                prop_assert!(chamber.x >= delve.rect.x, "left escape: {chamber:?} from {delve:?}");
                prop_assert!(chamber.y >= delve.rect.y, "top escape: {chamber:?} from {delve:?}");
                prop_assert!(chamber.x.saturating_add(chamber.width) <= delve.rect.right(), "right escape: {chamber:?} from {delve:?}");
                prop_assert!(chamber.y.saturating_add(chamber.height) <= delve.rect.bottom(), "bottom escape: {chamber:?} from {delve:?}");
                for other in delve.chambers.iter().skip(index + 1) {
                    prop_assert!(!overlaps(*chamber, *other), "Chambers overlap in {delve:?}: {chamber:?} and {other:?}");
                }
            }
        }
    }
}
