use std::collections::BTreeSet;

use ratatui::layout::{Constraint, Layout, Rect};

use crate::{
    app::{GuildFocus, Model},
    domain::{Agent, AgentKey, Campaign, GuildAttention, GuildSummons, Presence, WorkspaceId},
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
    pub illuminated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProjectedCampaignTable {
    pub workspace_id: WorkspaceId,
    pub label: String,
    pub seal: u64,
    pub area: Rect,
    pub selected: bool,
    pub illuminated: bool,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct GuildRoomProjection {
    pub mode: GuildRoomMode,
    pub focused: GuildFocus,
    pub breadcrumb: Option<String>,
    pub landmarks: Vec<ProjectedLandmark>,
    pub campaigns: Vec<ProjectedCampaignTable>,
    pub adventurers: Vec<AdventurerRepresentation>,
}

#[must_use]
pub fn project(model: &Model, area: Rect) -> GuildRoomProjection {
    let mode = mode_for(area);
    let focused = model.guild_focus();
    let illuminated_campaigns = model
        .domain()
        .agents
        .values()
        .filter(|agent| agent.focused)
        .map(|agent| &agent.workspace_id)
        .collect::<BTreeSet<_>>();
    let selected_workspace = model.selected_agent().map(|agent| &agent.workspace_id);
    let selected_index = model
        .domain()
        .campaigns
        .keys()
        .position(|workspace_id| Some(workspace_id) == selected_workspace)
        .unwrap_or(0);
    let geometry = geometry_for(
        mode,
        focused,
        area,
        model.domain().campaigns.len(),
        selected_index,
    );
    let landmarks = geometry
        .landmarks
        .into_iter()
        .map(|(landmark, area)| ProjectedLandmark {
            illuminated: landmark == GuildLandmark::Scrying && !illuminated_campaigns.is_empty(),
            landmark,
            area,
        })
        .collect();
    let campaigns = model
        .domain()
        .campaigns
        .values()
        .zip(geometry.campaigns)
        .map(|(campaign, campaign_area)| ProjectedCampaignTable {
            workspace_id: campaign.workspace_id.clone(),
            label: campaign_label(campaign),
            seal: campaign_seal(&campaign.workspace_id),
            area: campaign_area,
            selected: selected_workspace == Some(&campaign.workspace_id),
            illuminated: illuminated_campaigns.contains(&campaign.workspace_id),
        })
        .collect();
    let adventurers = model
        .domain()
        .agents
        .values()
        .filter_map(representation_for)
        .collect();

    GuildRoomProjection {
        mode,
        focused,
        breadcrumb: geometry.breadcrumb,
        landmarks,
        campaigns,
        adventurers,
    }
}

struct RoomGeometry {
    landmarks: Vec<(GuildLandmark, Rect)>,
    campaigns: Vec<Rect>,
    breadcrumb: Option<String>,
}

fn geometry_for(
    mode: GuildRoomMode,
    focus: GuildFocus,
    area: Rect,
    campaign_count: usize,
    selected_index: usize,
) -> RoomGeometry {
    match mode {
        GuildRoomMode::WholeRoom => whole_room_geometry(area, campaign_count),
        GuildRoomMode::CroppedRoom => cropped_room_geometry(area, campaign_count, selected_index),
        GuildRoomMode::LandmarkCamera => {
            landmark_camera_geometry(area, campaign_count, selected_index, focus)
        }
    }
}

fn whole_room_geometry(area: Rect, campaign_count: usize) -> RoomGeometry {
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
    RoomGeometry {
        landmarks: landmark_areas([
            (GuildLandmark::Door, door),
            (GuildLandmark::QuestWall, quest_wall),
            (GuildLandmark::CounselBell, counsel_bell),
            (GuildLandmark::Hearth, hearth),
            (GuildLandmark::Chronicle, chronicle),
            (GuildLandmark::Scrying, scrying),
            (GuildLandmark::Spoils, spoils),
        ]),
        campaigns: campaign_areas(campaign_room, campaign_count),
        breadcrumb: None,
    }
}

fn cropped_room_geometry(area: Rect, campaign_count: usize, selected_index: usize) -> RoomGeometry {
    let room = inset(area, 1);
    let [_, room] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(room);
    let [upper, middle, lower] = Layout::vertical([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 2),
        Constraint::Ratio(1, 4),
    ])
    .areas(room);
    let [door, quest_wall] =
        Layout::horizontal([Constraint::Ratio(1, 5), Constraint::Ratio(4, 5)]).areas(upper);
    let [campaign_zone, scrying] =
        Layout::horizontal([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)]).areas(middle);
    let [selected_table, markers] =
        Layout::vertical([Constraint::Ratio(3, 4), Constraint::Ratio(1, 4)]).areas(campaign_zone);
    let [hearth, counsel, chronicle, spoils] = Layout::horizontal([
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Ratio(1, 4),
        Constraint::Min(21),
    ])
    .areas(lower);
    RoomGeometry {
        landmarks: landmark_areas([
            (GuildLandmark::Door, door),
            (GuildLandmark::QuestWall, quest_wall),
            (GuildLandmark::CounselBell, counsel),
            (GuildLandmark::Hearth, hearth),
            (GuildLandmark::Chronicle, chronicle),
            (GuildLandmark::Scrying, scrying),
            (GuildLandmark::Spoils, spoils),
        ]),
        campaigns: selected_campaign_areas(selected_table, markers, campaign_count, selected_index),
        breadcrumb: None,
    }
}

fn landmark_camera_geometry(
    area: Rect,
    campaign_count: usize,
    selected_index: usize,
    focus: GuildFocus,
) -> RoomGeometry {
    let room = inset(area, 1);
    let [_, camera] = Layout::vertical([Constraint::Length(1), Constraint::Min(0)]).areas(room);
    let hidden = Rect::new(camera.x, camera.y, 0, 0);
    let focused_landmark = focus_landmark(focus);
    let landmarks = all_landmarks()
        .into_iter()
        .map(|landmark| {
            let landmark_area = if Some(&landmark) == focused_landmark.as_ref() {
                camera
            } else {
                hidden
            };
            (landmark, landmark_area)
        })
        .collect();
    let campaigns = (0..campaign_count)
        .map(|index| {
            if focus == GuildFocus::CampaignTables && index == selected_index {
                camera
            } else {
                hidden
            }
        })
        .collect();
    RoomGeometry {
        landmarks,
        campaigns,
        breadcrumb: Some(format!("GREAT ROOM / {}", focus_label(focus))),
    }
}

fn landmark_areas<const N: usize>(areas: [(GuildLandmark, Rect); N]) -> Vec<(GuildLandmark, Rect)> {
    areas.into_iter().collect()
}

fn all_landmarks() -> [GuildLandmark; 7] {
    [
        GuildLandmark::Door,
        GuildLandmark::QuestWall,
        GuildLandmark::CounselBell,
        GuildLandmark::Hearth,
        GuildLandmark::Chronicle,
        GuildLandmark::Scrying,
        GuildLandmark::Spoils,
    ]
}

fn focus_landmark(focus: GuildFocus) -> Option<GuildLandmark> {
    match focus {
        GuildFocus::QuestWall => Some(GuildLandmark::QuestWall),
        GuildFocus::CampaignTables => None,
        GuildFocus::CounselBell => Some(GuildLandmark::CounselBell),
        GuildFocus::Hearth => Some(GuildLandmark::Hearth),
        GuildFocus::Chronicle => Some(GuildLandmark::Chronicle),
        GuildFocus::Scrying => Some(GuildLandmark::Scrying),
        GuildFocus::Spoils => Some(GuildLandmark::Spoils),
        GuildFocus::Door => Some(GuildLandmark::Door),
    }
}

const fn focus_label(focus: GuildFocus) -> &'static str {
    match focus {
        GuildFocus::QuestWall => "QUEST WALL",
        GuildFocus::CampaignTables => "CAMPAIGN TABLES",
        GuildFocus::CounselBell => "COUNSEL BELL",
        GuildFocus::Hearth => "HEARTH",
        GuildFocus::Chronicle => "CHRONICLE",
        GuildFocus::Scrying => "SCRYING",
        GuildFocus::Spoils => "SPOILS",
        GuildFocus::Door => "DOOR",
    }
}

fn inset(area: Rect, margin: u16) -> Rect {
    Rect::new(
        area.x.saturating_add(margin).min(area.right()),
        area.y.saturating_add(margin).min(area.bottom()),
        area.width.saturating_sub(margin.saturating_mul(2)),
        area.height.saturating_sub(margin.saturating_mul(2)),
    )
}

fn selected_campaign_areas(
    selected_area: Rect,
    marker_area: Rect,
    count: usize,
    selected_index: usize,
) -> Vec<Rect> {
    if count == 0 {
        return Vec::new();
    }
    let marker_count = count.saturating_sub(1);
    let mut markers = campaign_areas(marker_area, marker_count).into_iter();
    (0..count)
        .map(|index| {
            if index == selected_index.min(count.saturating_sub(1)) {
                selected_area
            } else {
                markers.next().unwrap_or_default()
            }
        })
        .collect()
}

fn representation_for(agent: &Agent) -> Option<AdventurerRepresentation> {
    let unseen_completion = matches!(
        &agent.attention,
        GuildAttention::Unread {
            summons: GuildSummons::SpoilsReturned,
            ..
        }
    );
    match (agent.presence, unseen_completion) {
        (Presence::Exited, _) => None,
        (Presence::Blocked, _) => Some(AdventurerRepresentation::Projection {
            agent: agent.key.clone(),
            station: GuildLandmark::CounselBell,
        }),
        (Presence::Done, true) => Some(AdventurerRepresentation::Physical {
            agent: agent.key.clone(),
            station: GuildLandmark::Spoils,
        }),
        (Presence::Idle, _) => Some(AdventurerRepresentation::Physical {
            agent: agent.key.clone(),
            station: GuildLandmark::Hearth,
        }),
        (Presence::Working | Presence::Unknown, _) | (Presence::Done, false) => {
            Some(AdventurerRepresentation::Token {
                agent: agent.key.clone(),
                table: agent.workspace_id.clone(),
            })
        }
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
