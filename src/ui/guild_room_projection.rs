use ratatui::layout::{Constraint, Layout, Rect};

use crate::{
    app::Model,
    domain::{AgentKey, Campaign, WorkspaceId},
};

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GuildRoomMode {
    WholeRoom,
    CroppedRoom,
    LandmarkCamera,
}

#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum GuildLandmark {
    Door,
    QuestWall,
    CampaignTable(WorkspaceId),
    CounselBell,
    Hearth,
    Chronicle,
    Scrying,
    Spoils,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum AdventurerRepresentation {
    Physical {
        agent: AgentKey,
        station: GuildLandmark,
    },
    Token {
        agent: AgentKey,
        table: WorkspaceId,
    },
    Projection {
        agent: AgentKey,
        station: GuildLandmark,
    },
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedLandmark {
    pub landmark: GuildLandmark,
    pub area: Rect,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedCampaignTable {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub seal: u64,
    pub area: Rect,
    pub selected: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildRoomProjection {
    pub mode: GuildRoomMode,
    pub landmarks: Vec<ProjectedLandmark>,
    pub campaigns: Vec<ProjectedCampaignTable>,
    pub adventurers: Vec<AdventurerRepresentation>,
}

#[must_use]
pub fn project(model: &Model, area: Rect) -> GuildRoomProjection {
    let mode = mode_for(area);
    let [upper_room, central_room, lower_room] = Layout::vertical([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 4),
    ])
    .areas(area);
    let [door, quest_wall, chronicle] = Layout::horizontal([
        Constraint::Ratio(1, 5),
        Constraint::Ratio(3, 5),
        Constraint::Ratio(1, 5),
    ])
    .areas(upper_room);
    let [counsel_bell, campaign_room, scrying] = Layout::horizontal([
        Constraint::Ratio(1, 5),
        Constraint::Ratio(3, 5),
        Constraint::Ratio(1, 5),
    ])
    .areas(central_room);
    let [hearth, spoils] =
        Layout::horizontal([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)]).areas(lower_room);

    let landmarks = vec![
        ProjectedLandmark {
            landmark: GuildLandmark::Door,
            area: door,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::QuestWall,
            area: quest_wall,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::CounselBell,
            area: counsel_bell,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::Hearth,
            area: hearth,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::Chronicle,
            area: chronicle,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::Scrying,
            area: scrying,
        },
        ProjectedLandmark {
            landmark: GuildLandmark::Spoils,
            area: spoils,
        },
    ];

    let selected_workspace = model.selected_agent().map(|agent| &agent.workspace_id);
    let campaign_areas = campaign_areas(campaign_room, model.domain().campaigns.len());
    let campaigns = model
        .domain()
        .campaigns
        .values()
        .zip(campaign_areas)
        .map(|(campaign, campaign_area)| ProjectedCampaignTable {
            workspace_id: campaign.workspace_id.clone(),
            label: campaign_label(campaign),
            seal: campaign_seal(&campaign.workspace_id),
            area: campaign_area,
            selected: selected_workspace == Some(&campaign.workspace_id),
        })
        .collect();

    GuildRoomProjection {
        mode,
        landmarks,
        campaigns,
        adventurers: Vec::new(),
    }
}

const fn mode_for(area: Rect) -> GuildRoomMode {
    if area.width >= 120 {
        GuildRoomMode::WholeRoom
    } else if area.width >= 80 {
        GuildRoomMode::CroppedRoom
    } else {
        GuildRoomMode::LandmarkCamera
    }
}

fn campaign_areas(area: Rect, count: usize) -> Vec<Rect> {
    let constraints = vec![Constraint::Fill(1); count];
    Layout::horizontal(constraints)
        .split(area)
        .iter()
        .copied()
        .collect::<Vec<_>>()
}

fn campaign_label(campaign: &Campaign) -> String {
    let authored = campaign.label.trim();
    if is_meaningful(authored) {
        return authored.to_owned();
    }

    campaign
        .cwd
        .file_name()
        .map(|name| name.to_string_lossy())
        .map(|name| name.trim().to_owned())
        .filter(|name| is_meaningful(name))
        .unwrap_or_else(|| campaign.workspace_id.as_str().to_owned())
}

fn is_meaningful(label: &str) -> bool {
    !label.is_empty() && label != "~"
}

fn campaign_seal(workspace_id: &WorkspaceId) -> u64 {
    let mut hasher = blake3::Hasher::new();
    hasher.update(b"questmancer-campaign-seal");
    hasher.update(&[0]);
    hasher.update(workspace_id.as_str().as_bytes());
    let digest = hasher.finalize();
    let bytes = digest.as_bytes();
    u64::from_le_bytes([
        bytes[0], bytes[1], bytes[2], bytes[3], bytes[4], bytes[5], bytes[6], bytes[7],
    ])
}
