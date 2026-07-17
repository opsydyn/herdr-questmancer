use std::collections::BTreeMap;

use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::{
    app::{CharacterSet, ConnectionState, Model, Notice},
    domain::{Agent, AgentKey, Timestamp, WorkspaceId},
    ui::{
        EffectCells,
        copy::{EMPTY_GUILD, SCRYING_STILL},
        guild_room_projection::{
            AdventurerRepresentation, GuildLandmark, GuildRoomProjection, ProjectedCampaignTable,
            ProjectedLandmark,
        },
        persona::compose_chamber_adventurer_for_palette,
        pixel::{ColorRole, Palette, pack},
        theatre::{TheatreFrame, TheatrePose, frame_for},
        widgets::{
            guild_landmark::{
                LandmarkLayer, LandmarkTheme, render_campaign_table, render_chronicle_lectern,
                render_chronicle_marginalia, render_counsel_bell, render_door, render_hearth,
                render_quest_wall, render_scrying_alcove, render_spoils_desk,
            },
            presentation::present,
        },
    },
};

const ASCII_BORDER: border::Set<'static> = border::Set {
    top_left: "+",
    top_right: "+",
    bottom_left: "+",
    bottom_right: "+",
    vertical_left: "|",
    vertical_right: "|",
    horizontal_top: "-",
    horizontal_bottom: "-",
};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum GuildRoomRenderPath {
    Landmark(GuildLandmark),
    CampaignTable(WorkspaceId),
    Representation(AdventurerRepresentation),
}

#[must_use]
pub fn great_room_render_plan(projection: &GuildRoomProjection) -> Vec<GuildRoomRenderPath> {
    projection
        .landmarks
        .iter()
        .map(|landmark| GuildRoomRenderPath::Landmark(landmark.landmark.clone()))
        .chain(
            projection
                .campaigns
                .iter()
                .map(|campaign| GuildRoomRenderPath::CampaignTable(campaign.workspace_id.clone())),
        )
        .chain(
            projection
                .adventurers
                .iter()
                .cloned()
                .map(GuildRoomRenderPath::Representation),
        )
        .collect()
}

pub(super) fn render(
    frame: &mut Frame<'_>,
    model: &Model,
    projection: &GuildRoomProjection,
) -> EffectCells {
    let palette = Palette::from(model.preferences().color_mode);
    let theme = LandmarkTheme {
        character_set: model.preferences().character_set,
        palette,
    };
    let copy = GreatRoomCopy::for_model(model, projection);
    let plan = great_room_render_plan(projection);

    render_architecture(frame, model.preferences().character_set, palette);

    for path in &plan {
        if let GuildRoomRenderPath::Landmark(landmark) = path {
            render_landmark_path(
                frame,
                projection,
                landmark,
                LandmarkLayer::Furniture,
                &copy,
                theme,
            );
        }
    }

    for path in &plan {
        if let GuildRoomRenderPath::CampaignTable(workspace_id) = path
            && let Some(campaign) = campaign_for(projection, workspace_id)
        {
            render_campaign_table(frame, campaign, LandmarkLayer::Furniture, theme);
        }
    }

    let occupant_counts = plan
        .iter()
        .filter_map(|path| match path {
            GuildRoomRenderPath::Representation(representation) => {
                Some(OccupantOwner::for_representation(representation))
            }
            GuildRoomRenderPath::Landmark(_) | GuildRoomRenderPath::CampaignTable(_) => None,
        })
        .fold(
            BTreeMap::<OccupantOwner, usize>::new(),
            |mut counts, owner| {
                let count = counts.entry(owner).or_default();
                *count = count.saturating_add(1);
                counts
            },
        );
    let mut occupant_slots = BTreeMap::<OccupantOwner, usize>::new();
    for path in &plan {
        if let GuildRoomRenderPath::Representation(representation) = path {
            let owner = OccupantOwner::for_representation(representation);
            let total = occupant_counts.get(&owner).copied().unwrap_or(1);
            let slot = occupant_slots.entry(owner).or_default();
            render_representation(
                frame,
                model,
                projection,
                representation,
                *slot,
                total,
                palette,
            );
            *slot = slot.saturating_add(1);
        }
    }

    for path in &plan {
        match path {
            GuildRoomRenderPath::Landmark(landmark) => render_landmark_path(
                frame,
                projection,
                landmark,
                LandmarkLayer::Effects,
                &copy,
                theme,
            ),
            GuildRoomRenderPath::CampaignTable(workspace_id) => {
                if let Some(campaign) = campaign_for(projection, workspace_id) {
                    render_campaign_table(frame, campaign, LandmarkLayer::Effects, theme);
                }
            }
            GuildRoomRenderPath::Representation(_) => {}
        }
    }

    for path in &plan {
        match path {
            GuildRoomRenderPath::Landmark(landmark) => render_landmark_path(
                frame,
                projection,
                landmark,
                LandmarkLayer::Labels,
                &copy,
                theme,
            ),
            GuildRoomRenderPath::CampaignTable(workspace_id) => {
                if let Some(campaign) = campaign_for(projection, workspace_id) {
                    render_campaign_table(frame, campaign, LandmarkLayer::Labels, theme);
                }
            }
            GuildRoomRenderPath::Representation(_) => {}
        }
    }

    render_goblin_marginalia(frame, model, projection, theme)
}

#[derive(Debug)]
struct GreatRoomCopy {
    door: Vec<String>,
    counsel: Vec<String>,
    hearth: Vec<String>,
    chronicle: Vec<String>,
    scrying: Vec<String>,
    spoils: Vec<String>,
}

impl GreatRoomCopy {
    fn for_model(model: &Model, projection: &GuildRoomProjection) -> Self {
        Self {
            door: door_lines(model),
            counsel: counsel_lines(projection),
            hearth: hearth_lines(model, projection),
            chronicle: chronicle_lines(model),
            scrying: scrying_lines(model),
            spoils: spoils_lines(model),
        }
    }
}

fn render_landmark_path(
    frame: &mut Frame<'_>,
    projection: &GuildRoomProjection,
    identity: &GuildLandmark,
    layer: LandmarkLayer,
    copy: &GreatRoomCopy,
    theme: LandmarkTheme,
) {
    let Some(landmark) = landmark_for(projection, identity) else {
        return;
    };
    match identity {
        GuildLandmark::Door => render_door(frame, landmark, layer, &copy.door, theme),
        GuildLandmark::QuestWall => {
            render_quest_wall(frame, landmark, layer, &projection.campaigns, theme);
        }
        GuildLandmark::CampaignTable(workspace_id) => {
            if let Some(campaign) = campaign_for(projection, workspace_id) {
                render_campaign_table(frame, campaign, layer, theme);
            }
        }
        GuildLandmark::CounselBell => {
            render_counsel_bell(frame, landmark, layer, &copy.counsel, theme);
        }
        GuildLandmark::Hearth => render_hearth(frame, landmark, layer, &copy.hearth, theme),
        GuildLandmark::Chronicle => {
            render_chronicle_lectern(frame, landmark, layer, &copy.chronicle, theme);
        }
        GuildLandmark::Scrying => {
            render_scrying_alcove(frame, landmark, layer, &copy.scrying, theme);
        }
        GuildLandmark::Spoils => {
            render_spoils_desk(frame, landmark, layer, &copy.spoils, theme);
        }
    }
}

fn render_architecture(frame: &mut Frame<'_>, character_set: CharacterSet, palette: Palette) {
    let area = frame.area();
    if area.is_empty() {
        return;
    }
    let mut room = Block::default()
        .title(" QUESTMANCER'S GUILD HALL / THE GREAT ROOM ")
        .borders(Borders::ALL)
        .border_style(Style::new().fg(palette.resolve(ColorRole::Stone)))
        .style(
            Style::new()
                .fg(palette.resolve(ColorRole::Parchment))
                .bg(palette.resolve(ColorRole::DarkStone)),
        );
    if character_set == CharacterSet::Ascii {
        room = room.border_set(ASCII_BORDER);
    }
    frame.render_widget(room, area);
}

#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
enum OccupantOwner {
    Landmark(GuildLandmark),
    Campaign(WorkspaceId),
}

impl OccupantOwner {
    fn for_representation(representation: &AdventurerRepresentation) -> Self {
        match representation {
            AdventurerRepresentation::Physical { station, .. }
            | AdventurerRepresentation::Projection { station, .. } => {
                Self::Landmark(station.clone())
            }
            AdventurerRepresentation::Token { table, .. } => Self::Campaign(table.clone()),
        }
    }
}

fn render_representation(
    frame: &mut Frame<'_>,
    model: &Model,
    projection: &GuildRoomProjection,
    representation: &AdventurerRepresentation,
    slot: usize,
    total: usize,
    palette: Palette,
) {
    let Some(agent) = model
        .domain()
        .agents
        .get(representation_agent(representation))
    else {
        return;
    };
    let Some(area) = representation_area(projection, representation) else {
        return;
    };
    let theatre = frame_for(agent, model.now(), model.preferences());
    match representation {
        AdventurerRepresentation::Token { .. } => {
            render_token(frame, area, agent, theatre, model, slot, palette);
        }
        AdventurerRepresentation::Physical { .. } | AdventurerRepresentation::Projection { .. } => {
            render_full_figure(
                frame,
                area,
                agent,
                theatre,
                model,
                slot,
                total,
                matches!(representation, AdventurerRepresentation::Projection { .. }),
                palette,
            );
        }
    }
}

fn render_token(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    model: &Model,
    slot: usize,
    palette: Palette,
) {
    let Ok(slot) = u16::try_from(slot) else {
        return;
    };
    let y = area.y.saturating_add(4).saturating_add(slot);
    if y >= area.bottom() {
        return;
    }
    let marker = match model.preferences().character_set {
        CharacterSet::Unicode => "◆",
        CharacterSet::Ascii => "o",
    };
    let label = format!(
        "{marker} {} [TOKEN / {}]",
        present(&agent.persona.name, model.preferences().character_set),
        status_label(theatre.pose, agent, model)
    );
    frame.render_widget(
        Paragraph::new(label).style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        Rect::new(area.x.saturating_add(2), y, area.width.saturating_sub(4), 1),
    );
}

#[allow(clippy::too_many_arguments)]
fn render_full_figure(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    model: &Model,
    slot: usize,
    total: usize,
    projection: bool,
    palette: Palette,
) {
    if area.width < 4 || area.height < 4 {
        return;
    }
    let compact_horizontal = area.height < 12 && area.width >= 26;
    let capacity = if compact_horizontal {
        usize::from((area.width.saturating_sub(2) / 26).max(1))
    } else {
        usize::from((area.width.saturating_sub(2) / 18).max(1))
    };
    let columns = capacity.min(total.max(1));
    let row = slot / columns;
    let column = slot % columns;
    let column_width = area.width.saturating_sub(2) / u16::try_from(columns).unwrap_or(1);
    let x = area.x.saturating_add(1).saturating_add(
        u16::try_from(column)
            .unwrap_or(u16::MAX)
            .saturating_mul(column_width),
    );
    let row_height = if compact_horizontal { 6 } else { 9 };
    let y = area.y.saturating_add(3).saturating_add(
        u16::try_from(row)
            .unwrap_or(u16::MAX)
            .saturating_mul(row_height),
    );
    if y >= area.bottom() {
        return;
    }

    let name = present(&agent.persona.name, model.preferences().character_set).into_owned();
    let label = name;
    let status = status_label(theatre.pose, agent, model);
    let status = if projection {
        format!("PROJECTED / {status}")
    } else {
        status
    };
    let (art_y, label_area, status_area) = if compact_horizontal {
        (
            y,
            Rect::new(x.saturating_add(11), y, column_width.saturating_sub(11), 1),
            Rect::new(
                x.saturating_add(11),
                y.saturating_add(1),
                column_width.saturating_sub(11),
                1,
            ),
        )
    } else {
        (
            y.saturating_add(1),
            Rect::new(x, y, column_width, 1),
            Rect::new(x, y.saturating_add(7), column_width, 1),
        )
    };
    frame.render_widget(
        Paragraph::new(label).style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        label_area,
    );
    frame.render_widget(
        Paragraph::new(status).style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        status_area,
    );

    let art_area = Rect::new(
        x,
        art_y,
        column_width.min(10),
        area.bottom().saturating_sub(art_y),
    );
    match model.preferences().character_set {
        CharacterSet::Unicode => {
            let canvas = compose_chamber_adventurer_for_palette(&agent.persona, theatre, palette);
            frame.render_widget(
                Paragraph::new(pack(&canvas, &palette, ColorRole::DarkStone)),
                art_area,
            );
        }
        CharacterSet::Ascii => frame.render_widget(
            Paragraph::new(Text::from(ascii_figure(theatre.pose))),
            art_area,
        ),
    }
}

fn ascii_figure(pose: TheatrePose) -> Vec<Line<'static>> {
    let rows = match pose {
        TheatrePose::SeekingCounsel => [
            "   \\o/   ",
            "    |    ",
            "   / \\   ",
            " LANTERN ",
            " counsel ",
            "         ",
        ],
        TheatrePose::SpoilsUnopened => [
            "   \\o/   ",
            "    |    ",
            "   / \\   ",
            "  CHEST  ",
            " spoils  ",
            "         ",
        ],
        TheatrePose::Resting => [
            "    o    ",
            "   /|\\   ",
            "   / \\   ",
            " CAMPFIRE",
            " resting ",
            "         ",
        ],
        TheatrePose::Delving => [
            "    o>   ",
            "   /|\\   ",
            "   / \\   ",
            "  TOKEN  ",
            " working ",
            "         ",
        ],
        TheatrePose::VictoryRecorded => [
            "    o    ",
            "   /|\\   ",
            "   / \\   ",
            " LEDGER  ",
            " victory ",
            "         ",
        ],
        TheatrePose::Departed => [
            "   [x]   ",
            "  EMPTY  ",
            "         ",
            "         ",
            "departed ",
            "         ",
        ],
        TheatrePose::Unknown => [
            "    ?    ",
            "   /|\\   ",
            "   / \\   ",
            " UNKNOWN ",
            "         ",
            "         ",
        ],
    };
    rows.into_iter().map(Line::from).collect()
}

fn representation_agent(representation: &AdventurerRepresentation) -> &AgentKey {
    match representation {
        AdventurerRepresentation::Physical { agent, .. }
        | AdventurerRepresentation::Token { agent, .. }
        | AdventurerRepresentation::Projection { agent, .. } => agent,
    }
}

fn representation_area(
    projection: &GuildRoomProjection,
    representation: &AdventurerRepresentation,
) -> Option<Rect> {
    match representation {
        AdventurerRepresentation::Physical { station, .. }
        | AdventurerRepresentation::Projection { station, .. } => {
            landmark_for(projection, station).map(|landmark| landmark.area)
        }
        AdventurerRepresentation::Token { table, .. } => {
            campaign_for(projection, table).map(|campaign| campaign.area)
        }
    }
}

fn landmark_for<'a>(
    projection: &'a GuildRoomProjection,
    identity: &GuildLandmark,
) -> Option<&'a ProjectedLandmark> {
    projection
        .landmarks
        .iter()
        .find(|landmark| &landmark.landmark == identity)
}

fn campaign_for<'a>(
    projection: &'a GuildRoomProjection,
    workspace_id: &WorkspaceId,
) -> Option<&'a ProjectedCampaignTable> {
    projection
        .campaigns
        .iter()
        .find(|campaign| &campaign.workspace_id == workspace_id)
}

fn door_lines(model: &Model) -> Vec<String> {
    let mut lines = match model.connection() {
        ConnectionState::Offline => vec!["OFFLINE / door closed".to_owned()],
        ConnectionState::Connecting => vec!["CONNECTING / door opening".to_owned()],
        ConnectionState::Connected => vec!["CONNECTED / door open".to_owned()],
        ConnectionState::Reconnecting { attempt } => vec![
            "RECONNECTING / door barred".to_owned(),
            format!("attempt {attempt}"),
        ],
        ConnectionState::Incompatible { expected, actual } => vec![
            "INCOMPATIBLE / door sealed".to_owned(),
            format!("protocol {actual}; expected {expected}"),
        ],
    };
    if !matches!(model.connection(), ConnectionState::Connected)
        && let Some(Notice::ConnectionDiagnostic(message)) = model.notice()
    {
        lines.push(format!(
            "Cause: {}",
            present(message, model.preferences().character_set)
        ));
    }
    lines
}

fn counsel_lines(projection: &GuildRoomProjection) -> Vec<String> {
    let count = projection
        .adventurers
        .iter()
        .filter(|representation| {
            matches!(
                representation,
                AdventurerRepresentation::Projection {
                    station: GuildLandmark::CounselBell,
                    ..
                }
            )
        })
        .count();
    match count {
        0 => vec!["The bell is quiet.".to_owned()],
        1 => vec!["requests counsel".to_owned()],
        count => vec![format!("{count} request counsel")],
    }
}

fn hearth_lines(model: &Model, projection: &GuildRoomProjection) -> Vec<String> {
    if model.domain().agents.is_empty() {
        return vec![EMPTY_GUILD.to_owned()];
    }
    let resting = projection
        .adventurers
        .iter()
        .filter(|representation| {
            matches!(
                representation,
                AdventurerRepresentation::Physical {
                    station: GuildLandmark::Hearth,
                    ..
                }
            )
        })
        .count();
    if resting == 0 {
        vec!["Warm coals wait beside the communal rug.".to_owned()]
    } else {
        Vec::new()
    }
}

fn chronicle_lines(model: &Model) -> Vec<String> {
    let lines = model
        .domain()
        .chronicle
        .entries()
        .iter()
        .rev()
        .take(5)
        .map(|entry| present(&entry.summary, model.preferences().character_set).into_owned())
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec!["No deeds recorded yet.".to_owned()]
    } else {
        lines
    }
}

fn scrying_lines(model: &Model) -> Vec<String> {
    let Some(agent) = model.selected_agent() else {
        return vec![SCRYING_STILL.to_owned()];
    };
    let mut lines = vec![format!(
        "SELECTED PANE: {}",
        present(&agent.name, model.preferences().character_set)
    )];
    if model.managed_pane_id() == Some(&agent.pane_id) {
        lines.push(SCRYING_STILL.to_owned());
        lines.push("Cause: the Questmancer's own pane is never observed.".to_owned());
        return lines;
    }
    match model
        .output_preview()
        .filter(|preview| preview.pane_id == agent.pane_id)
    {
        Some(preview) if preview.error.is_some() => {
            lines.push("The scrying pool has clouded.".to_owned());
            lines.push(format!(
                "Cause: {}",
                present(
                    preview.error.as_deref().unwrap_or_default(),
                    model.preferences().character_set
                )
            ));
        }
        Some(preview) if preview.loading => {
            lines.push(SCRYING_STILL.to_owned());
            lines.push("Tracing the selected adventurer...".to_owned());
        }
        Some(preview) => lines.extend(
            preview
                .text
                .lines()
                .map(|line| present(line, model.preferences().character_set).into_owned()),
        ),
        None => {
            lines.push(SCRYING_STILL.to_owned());
            lines.push("Select refresh to trace recent deeds.".to_owned());
        }
    }
    lines
}

fn spoils_lines(model: &Model) -> Vec<String> {
    let mut lines = Vec::new();
    if model.reviewr_available() {
        lines.push("Reviewr ready: [v] Inspect spoils".to_owned());
    }
    if let Some(Notice::IntegrationDiagnostic(message)) = model.notice() {
        lines.push(present(message, model.preferences().character_set).into_owned());
    }
    lines
}

fn status_label(pose: TheatrePose, agent: &Agent, model: &Model) -> String {
    let state = match pose {
        TheatrePose::Delving => "working",
        TheatrePose::SeekingCounsel => "blocked",
        TheatrePose::SpoilsUnopened | TheatrePose::VictoryRecorded => "completed",
        TheatrePose::Resting => "resting",
        TheatrePose::Departed => "departed",
        TheatrePose::Unknown => "unknown",
    };
    if !model.settings().show_elapsed_time {
        return state.to_owned();
    }
    format!(
        "{state} {}",
        elapsed_label(agent.presence_since, model.now())
    )
}

fn elapsed_label(since: Timestamp, now: Timestamp) -> String {
    let elapsed = since.elapsed_until(now).as_secs();
    if elapsed >= 60 {
        format!("{}m", elapsed / 60)
    } else {
        format!("{elapsed}s")
    }
}

fn render_goblin_marginalia(
    frame: &mut Frame<'_>,
    model: &Model,
    projection: &GuildRoomProjection,
    theme: LandmarkTheme,
) -> EffectCells {
    if !model.goblins().is_visible(model.now()) || projection.adventurers.is_empty() {
        return EffectCells::default();
    }
    let Some(chronicle) = landmark_for(projection, &GuildLandmark::Chronicle) else {
        return EffectCells::default();
    };
    let before = frame.buffer_mut().clone();
    render_chronicle_marginalia(frame, chronicle, theme);
    EffectCells::changed_between(&before, frame.buffer_mut(), chronicle.area)
}
