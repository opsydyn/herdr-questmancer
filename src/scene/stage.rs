use crate::{
    app::Motion,
    domain::{AgentKey, GuildSummons, Presence, Timestamp, WorkspaceId},
    scene::{
        pixel::PixelSize,
        snapshot::{SceneAgent, SceneConnection, SceneSnapshot},
    },
};

pub const COMPLETION_THEATRE_MS: i64 = 3_000;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum WorldScene {
    GuildHall,
    Delve,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SceneCamera {
    WholeRoom,
    Focused { anchor: CameraAnchor },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum CameraAnchor {
    Door,
    CampaignTable(WorkspaceId),
    CounselBell,
    Hearth,
    Spoils,
    DelveParty(WorkspaceId),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TruthfulStation {
    CampaignToken(WorkspaceId),
    CounselBell,
    SpoilsBench,
    Hearth,
    DelveActive(WorkspaceId),
    DelveGate(WorkspaceId),
    DelveExit(WorkspaceId),
    DelveCamp(WorkspaceId),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ScenePose {
    Working,
    SeekingCounsel,
    ReturningWithSpoils,
    Settled,
    Resting,
    Unknown,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ActorPlacement {
    pub agent: AgentKey,
    pub station: TruthfulStation,
    pub pose: ScenePose,
    pub focused: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum SceneEffect {
    FreshSpoils {
        agent: AgentKey,
        since: Timestamp,
    },
    RecentDeparture {
        workspace_id: WorkspaceId,
        since: Timestamp,
    },
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SceneCadence {
    EventDriven,
    Fps(u8),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ScenePlan {
    pub world: WorldScene,
    pub camera: SceneCamera,
    pub actors: Vec<ActorPlacement>,
    pub effects: Vec<SceneEffect>,
    pub cadence: SceneCadence,
}

impl ScenePlan {
    #[must_use]
    pub fn project(snapshot: &SceneSnapshot, viewport: PixelSize) -> Self {
        project_for_world(snapshot, viewport, automatic_world(snapshot))
    }
}

#[must_use]
pub(crate) fn project_for_world(
    snapshot: &SceneSnapshot,
    viewport: PixelSize,
    world: WorldScene,
) -> ScenePlan {
    let agents = sorted_agents(snapshot);
    let actors = agents
        .iter()
        .filter_map(|agent| placement_for(agent, snapshot.now, world))
        .collect::<Vec<_>>();
    let effects = agents
        .iter()
        .filter_map(|agent| effect_for(agent, snapshot.now))
        .collect::<Vec<_>>();
    let camera = camera_for(&agents, snapshot.now, viewport, world);
    let cadence = cadence_for(snapshot.motion, &actors, &effects);

    ScenePlan {
        world,
        camera,
        actors,
        effects,
        cadence,
    }
}

fn automatic_world(snapshot: &SceneSnapshot) -> WorldScene {
    if snapshot.connection != SceneConnection::Connected
        || snapshot
            .agents
            .iter()
            .any(|agent| agent.presence == Presence::Blocked)
        || snapshot
            .agents
            .iter()
            .any(|agent| has_fresh_spoils(agent, snapshot.now))
    {
        return WorldScene::GuildHall;
    }

    if snapshot
        .agents
        .iter()
        .any(|agent| agent.focused && agent.presence == Presence::Working)
        || snapshot
            .agents
            .iter()
            .any(|agent| agent.presence == Presence::Working)
    {
        WorldScene::Delve
    } else {
        WorldScene::GuildHall
    }
}

fn sorted_agents(snapshot: &SceneSnapshot) -> Vec<&SceneAgent> {
    let mut agents = snapshot.agents.iter().collect::<Vec<_>>();
    agents.sort_by(|left, right| left.key.cmp(&right.key));
    agents
}

fn placement_for(agent: &SceneAgent, now: Timestamp, world: WorldScene) -> Option<ActorPlacement> {
    let (station, pose) = match (world, agent.presence) {
        (WorldScene::GuildHall, Presence::Working) => (
            TruthfulStation::CampaignToken(agent.workspace_id.clone()),
            ScenePose::Working,
        ),
        (WorldScene::GuildHall, Presence::Unknown) => (
            TruthfulStation::CampaignToken(agent.workspace_id.clone()),
            ScenePose::Unknown,
        ),
        (WorldScene::GuildHall, Presence::Blocked) => {
            (TruthfulStation::CounselBell, ScenePose::SeekingCounsel)
        }
        (WorldScene::GuildHall, Presence::Done) => (
            TruthfulStation::SpoilsBench,
            if has_fresh_spoils(agent, now) {
                ScenePose::ReturningWithSpoils
            } else {
                ScenePose::Settled
            },
        ),
        (WorldScene::GuildHall, Presence::Idle) => (TruthfulStation::Hearth, ScenePose::Resting),
        (WorldScene::Delve, Presence::Working) => (
            TruthfulStation::DelveActive(agent.workspace_id.clone()),
            ScenePose::Working,
        ),
        (WorldScene::Delve, Presence::Blocked) => (
            TruthfulStation::DelveGate(agent.workspace_id.clone()),
            ScenePose::SeekingCounsel,
        ),
        (WorldScene::Delve, Presence::Done) => (
            TruthfulStation::DelveExit(agent.workspace_id.clone()),
            if has_fresh_spoils(agent, now) {
                ScenePose::ReturningWithSpoils
            } else {
                ScenePose::Settled
            },
        ),
        (WorldScene::Delve, Presence::Idle) => (
            TruthfulStation::DelveCamp(agent.workspace_id.clone()),
            ScenePose::Resting,
        ),
        (WorldScene::Delve, Presence::Unknown) => (
            TruthfulStation::DelveActive(agent.workspace_id.clone()),
            ScenePose::Unknown,
        ),
        (_, Presence::Exited) => return None,
    };

    Some(ActorPlacement {
        agent: agent.key.clone(),
        station,
        pose,
        focused: agent.focused,
    })
}

fn effect_for(agent: &SceneAgent, now: Timestamp) -> Option<SceneEffect> {
    let transition = agent.transition?;
    if !is_fresh(transition.since, now) {
        return None;
    }
    match transition.summons {
        GuildSummons::SpoilsReturned => Some(SceneEffect::FreshSpoils {
            agent: agent.key.clone(),
            since: transition.since,
        }),
        GuildSummons::AdventurerDeparted => Some(SceneEffect::RecentDeparture {
            workspace_id: agent.workspace_id.clone(),
            since: transition.since,
        }),
        GuildSummons::CounselRequested => None,
    }
}

fn camera_for(
    agents: &[&SceneAgent],
    now: Timestamp,
    viewport: PixelSize,
    world: WorldScene,
) -> SceneCamera {
    if viewport.width >= 120 && viewport.height >= 72 {
        return SceneCamera::WholeRoom;
    }

    let target = agents
        .iter()
        .copied()
        .find(|agent| agent.presence == Presence::Blocked)
        .or_else(|| {
            agents
                .iter()
                .copied()
                .find(|agent| has_fresh_spoils(agent, now))
        })
        .or_else(|| {
            agents
                .iter()
                .copied()
                .find(|agent| agent.focused && agent.presence != Presence::Exited)
        });
    let anchor = target.map_or(CameraAnchor::Door, |agent| anchor_for(agent, world));
    SceneCamera::Focused { anchor }
}

fn anchor_for(agent: &SceneAgent, world: WorldScene) -> CameraAnchor {
    if world == WorldScene::Delve {
        return CameraAnchor::DelveParty(agent.workspace_id.clone());
    }
    match agent.presence {
        Presence::Working | Presence::Unknown => {
            CameraAnchor::CampaignTable(agent.workspace_id.clone())
        }
        Presence::Blocked => CameraAnchor::CounselBell,
        Presence::Done => CameraAnchor::Spoils,
        Presence::Idle => CameraAnchor::Hearth,
        Presence::Exited => CameraAnchor::Door,
    }
}

fn cadence_for(motion: Motion, actors: &[ActorPlacement], effects: &[SceneEffect]) -> SceneCadence {
    match motion {
        Motion::None => SceneCadence::EventDriven,
        Motion::Reduced => {
            if actors.iter().any(|actor| actor.pose == ScenePose::Resting) {
                SceneCadence::Fps(1)
            } else {
                SceneCadence::EventDriven
            }
        }
        Motion::Full => {
            let effect_fps = if effects
                .iter()
                .any(|effect| matches!(effect, SceneEffect::FreshSpoils { .. }))
            {
                8
            } else {
                0
            };
            let actor_fps = actors
                .iter()
                .map(|actor| match actor.pose {
                    ScenePose::ReturningWithSpoils => 8,
                    ScenePose::Working => 6,
                    ScenePose::SeekingCounsel => 2,
                    ScenePose::Resting => 1,
                    ScenePose::Settled | ScenePose::Unknown => 0,
                })
                .max()
                .unwrap_or(0);
            match effect_fps.max(actor_fps).min(8) {
                0 => SceneCadence::EventDriven,
                fps => SceneCadence::Fps(fps),
            }
        }
    }
}

fn has_fresh_spoils(agent: &SceneAgent, now: Timestamp) -> bool {
    agent.transition.is_some_and(|transition| {
        transition.summons == GuildSummons::SpoilsReturned && is_fresh(transition.since, now)
    })
}

fn is_fresh(since: Timestamp, now: Timestamp) -> bool {
    now >= since && now.as_millis().saturating_sub(since.as_millis()) < COMPLETION_THEATRE_MS
}

#[cfg(test)]
mod tests {
    use crate::domain::{AdventurerPersona, PersonaKey};

    use super::*;

    fn agent(key: &str, presence: Presence) -> SceneAgent {
        SceneAgent {
            key: AgentKey::new(key),
            workspace_id: WorkspaceId::new("workspace"),
            name: key.to_owned(),
            custom_status: None,
            presence,
            presence_since: Timestamp::from_millis(0),
            transition: None,
            focused: false,
            persona: AdventurerPersona::for_key(PersonaKey::new(format!("persona-{key}"))),
        }
    }

    #[test]
    fn forced_delve_maps_blocked_agents_to_the_sealed_gate() {
        let snapshot = SceneSnapshot {
            connection: SceneConnection::Connected,
            campaigns: Vec::new(),
            agents: vec![agent("blocked", Presence::Blocked)],
            motion: Motion::None,
            now: Timestamp::from_millis(1_000),
        };

        let plan = project_for_world(&snapshot, PixelSize::new(80, 50), WorldScene::Delve);

        assert_eq!(
            plan.actors,
            vec![ActorPlacement {
                agent: AgentKey::new("blocked"),
                station: TruthfulStation::DelveGate(WorkspaceId::new("workspace")),
                pose: ScenePose::SeekingCounsel,
                focused: false,
            }]
        );
    }
}
