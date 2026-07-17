use std::{borrow::Cow, collections::HashMap};

use ratatui::{
    Frame,
    buffer::CellWidth,
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
                LandmarkLayer, LandmarkTheme, content_visual_height, render_campaign_table,
                render_chronicle_lectern, render_chronicle_marginalia, render_counsel_bell,
                render_door, render_hearth, render_quest_wall, render_scrying_alcove,
                render_spoils_desk,
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
    area: Rect,
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
    let index = RenderIndex::new(projection);
    let measurements = RenderMeasurements::new(projection, &index, &copy);

    render_architecture(frame, area, model.preferences().character_set, palette);

    for path in &plan {
        if let GuildRoomRenderPath::Landmark(landmark) = path {
            render_landmark_path(
                frame,
                projection,
                &index,
                landmark,
                LandmarkLayer::Furniture,
                &copy,
                measurements,
                theme,
            );
        }
    }

    for path in &plan {
        if let GuildRoomRenderPath::CampaignTable(workspace_id) = path
            && let Some(campaign) = index.campaign(workspace_id)
        {
            render_campaign_table(frame, campaign, LandmarkLayer::Furniture, theme);
        }
    }

    render_representations(frame, model, &plan, &index, measurements, palette);

    for path in &plan {
        match path {
            GuildRoomRenderPath::Landmark(landmark) => render_landmark_path(
                frame,
                projection,
                &index,
                landmark,
                LandmarkLayer::Effects,
                &copy,
                measurements,
                theme,
            ),
            GuildRoomRenderPath::CampaignTable(workspace_id) => {
                if let Some(campaign) = index.campaign(workspace_id) {
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
                &index,
                landmark,
                LandmarkLayer::Labels,
                &copy,
                measurements,
                theme,
            ),
            GuildRoomRenderPath::CampaignTable(workspace_id) => {
                if let Some(campaign) = index.campaign(workspace_id) {
                    render_campaign_table(frame, campaign, LandmarkLayer::Labels, theme);
                }
            }
            GuildRoomRenderPath::Representation(_) => {}
        }
    }

    render_goblin_marginalia(frame, model, projection, &index, theme)
}

fn render_representations(
    frame: &mut Frame<'_>,
    model: &Model,
    plan: &[GuildRoomRenderPath],
    index: &RenderIndex<'_>,
    measurements: RenderMeasurements,
    palette: Palette,
) {
    let occupant_counts = plan
        .iter()
        .filter_map(|path| match path {
            GuildRoomRenderPath::Representation(representation) => {
                Some(OccupantOwner::for_representation(representation))
            }
            GuildRoomRenderPath::Landmark(_) | GuildRoomRenderPath::CampaignTable(_) => None,
        })
        .fold(
            HashMap::<OccupantOwner, usize>::new(),
            |mut counts, owner| {
                let count = counts.entry(owner).or_default();
                *count = count.saturating_add(1);
                counts
            },
        );
    let mut occupant_slots = HashMap::<OccupantOwner, usize>::new();
    for path in plan {
        if let GuildRoomRenderPath::Representation(representation) = path {
            let owner = OccupantOwner::for_representation(representation);
            let total = occupant_counts.get(&owner).copied().unwrap_or(1);
            let slot = occupant_slots.entry(owner).or_default();
            let rows = measurements.representation_rows(representation);
            render_representation(
                frame,
                model,
                index,
                representation,
                *slot,
                total,
                rows,
                palette,
            );
            *slot = slot.saturating_add(1);
        }
    }
}

struct RenderIndex<'a> {
    landmarks: HashMap<GuildLandmark, &'a ProjectedLandmark>,
    campaigns: HashMap<WorkspaceId, &'a ProjectedCampaignTable>,
}

impl<'a> RenderIndex<'a> {
    fn new(projection: &'a GuildRoomProjection) -> Self {
        Self {
            landmarks: projection
                .landmarks
                .iter()
                .map(|landmark| (landmark.landmark.clone(), landmark))
                .collect(),
            campaigns: projection
                .campaigns
                .iter()
                .map(|campaign| (campaign.workspace_id.clone(), campaign))
                .collect(),
        }
    }

    fn landmark(&self, identity: &GuildLandmark) -> Option<&'a ProjectedLandmark> {
        self.landmarks.get(identity).copied()
    }

    fn campaign(&self, workspace_id: &WorkspaceId) -> Option<&'a ProjectedCampaignTable> {
        self.campaigns.get(workspace_id).copied()
    }

    fn representation_area(&self, representation: &AdventurerRepresentation) -> Option<Rect> {
        match representation {
            AdventurerRepresentation::Physical { station, .. }
            | AdventurerRepresentation::Projection { station, .. } => {
                self.landmark(station).map(|landmark| landmark.area)
            }
            AdventurerRepresentation::Token { table, .. } => {
                self.campaign(table).map(|campaign| campaign.area)
            }
        }
    }
}

#[derive(Debug)]
struct GreatRoomCopy<'a> {
    door: Vec<Cow<'a, str>>,
    counsel: Vec<Cow<'a, str>>,
    hearth: Vec<Cow<'a, str>>,
    chronicle: Vec<Cow<'a, str>>,
    scrying: Vec<Cow<'a, str>>,
    spoils: Vec<Cow<'a, str>>,
}

impl<'a> GreatRoomCopy<'a> {
    fn for_model(model: &'a Model, projection: &GuildRoomProjection) -> Self {
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

#[derive(Clone, Copy, Debug)]
struct RenderMeasurements {
    spoils: SpoilsMeasurement,
}

#[derive(Clone, Copy, Debug, Default)]
struct RepresentationRows {
    top: u16,
    bottom: u16,
}

#[derive(Clone, Copy, Debug, Default)]
struct SpoilsMeasurement {
    copy_height: u16,
    furniture_height: u16,
    representation_bounds: RepresentationRows,
}

impl RenderMeasurements {
    fn new(
        projection: &GuildRoomProjection,
        index: &RenderIndex<'_>,
        copy: &GreatRoomCopy<'_>,
    ) -> Self {
        let spoils = index.landmark(&GuildLandmark::Spoils).map_or_else(
            SpoilsMeasurement::default,
            |landmark| {
                let has_returnee = projection.adventurers.iter().any(|representation| {
                    matches!(
                        representation,
                        AdventurerRepresentation::Physical {
                            station: GuildLandmark::Spoils,
                            ..
                        }
                    )
                });
                SpoilsMeasurement::new(
                    landmark.area,
                    has_returnee,
                    content_visual_height(landmark.area, 2, &copy.spoils),
                )
            },
        );
        Self { spoils }
    }

    fn representation_rows(self, representation: &AdventurerRepresentation) -> RepresentationRows {
        match representation {
            AdventurerRepresentation::Physical {
                station: GuildLandmark::Spoils,
                ..
            } => self.spoils.representation_bounds,
            AdventurerRepresentation::Projection {
                station: GuildLandmark::CounselBell,
                ..
            } => RepresentationRows { top: 2, bottom: 3 },
            AdventurerRepresentation::Physical { .. }
            | AdventurerRepresentation::Projection { .. }
            | AdventurerRepresentation::Token { .. } => RepresentationRows { top: 2, bottom: 2 },
        }
    }
}

impl SpoilsMeasurement {
    const HEADER_ROWS: u16 = 2;
    const FURNITURE_ROWS: u16 = 2;
    const RETURNED_IDENTITY_ROWS: u16 = 2;

    fn new(area: Rect, has_returnee: bool, content_height: u16) -> Self {
        let inner_height = representation_inner(area).height;
        let identity_rows = if has_returnee {
            Self::RETURNED_IDENTITY_ROWS
        } else {
            0
        };
        let content_without_furniture = inner_height
            .saturating_sub(Self::HEADER_ROWS)
            .saturating_sub(identity_rows);
        let content_with_furniture = content_without_furniture.saturating_sub(Self::FURNITURE_ROWS);
        let content_rows = content_height.min(content_without_furniture);
        let furniture_rows = if content_rows <= content_with_furniture {
            Self::FURNITURE_ROWS
        } else {
            0
        };

        Self {
            copy_height: content_rows,
            furniture_height: furniture_rows,
            representation_bounds: RepresentationRows {
                top: Self::HEADER_ROWS.saturating_add(content_rows),
                bottom: furniture_rows,
            },
        }
    }
}

fn render_landmark_path(
    frame: &mut Frame<'_>,
    projection: &GuildRoomProjection,
    index: &RenderIndex<'_>,
    identity: &GuildLandmark,
    layer: LandmarkLayer,
    copy: &GreatRoomCopy<'_>,
    measurements: RenderMeasurements,
    theme: LandmarkTheme,
) {
    let Some(landmark) = index.landmark(identity) else {
        return;
    };
    match identity {
        GuildLandmark::Door => render_door(frame, landmark, layer, &copy.door, theme),
        GuildLandmark::QuestWall => {
            render_quest_wall(frame, landmark, layer, &projection.campaigns, theme);
        }
        GuildLandmark::CampaignTable(workspace_id) => {
            if let Some(campaign) = index.campaign(workspace_id) {
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
            render_spoils_desk(
                frame,
                landmark,
                layer,
                &copy.spoils,
                measurements.spoils.copy_height,
                measurements.spoils.furniture_height,
                theme,
            );
        }
    }
}

fn render_architecture(
    frame: &mut Frame<'_>,
    area: Rect,
    character_set: CharacterSet,
    palette: Palette,
) {
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

#[derive(Clone, Debug, Eq, Hash, PartialEq)]
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
    index: &RenderIndex<'_>,
    representation: &AdventurerRepresentation,
    slot: usize,
    total: usize,
    rows: RepresentationRows,
    palette: Palette,
) {
    let Some(agent) = model
        .domain()
        .agents
        .get(representation_agent(representation))
    else {
        return;
    };
    let Some(area) = index.representation_area(representation) else {
        return;
    };
    let theatre = frame_for(agent, model.now(), model.preferences());
    match representation {
        AdventurerRepresentation::Token { .. } => {
            render_token(frame, area, agent, theatre, model, slot, total, palette);
        }
        AdventurerRepresentation::Physical { .. } | AdventurerRepresentation::Projection { .. } => {
            let projected = matches!(representation, AdventurerRepresentation::Projection { .. });
            if total == 1 {
                render_full_figure(frame, area, agent, theatre, model, rows, projected, palette);
            } else {
                render_dense_figure(
                    frame,
                    area,
                    agent,
                    theatre,
                    model,
                    slot,
                    total,
                    rows.top,
                    rows.bottom,
                    palette,
                );
            }
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
    total: usize,
    palette: Palette,
) {
    let inner = representation_inner(area);
    if inner.is_empty() {
        return;
    }
    let top_rows = if area.width <= 14 { 5 } else { 4 }.min(inner.height);
    let available_rows = inner.height.saturating_sub(top_rows).saturating_sub(2);
    if available_rows == 0 {
        return;
    }
    let max_columns = usize::from((inner.width / 6).max(1));
    let preferred_columns = usize::from((inner.width / 14).max(1)).min(total.max(1));
    let needed_columns = total.max(1).div_ceil(usize::from(available_rows));
    let columns = preferred_columns
        .max(needed_columns.min(max_columns))
        .min(max_columns)
        .min(total.max(1));
    let summary = OverflowSummary::for_owner(
        total,
        columns,
        available_rows,
        inner.width,
        "TOKENS",
        "TOKENS",
    );
    let visible_rows = summary.map_or(available_rows, |summary| summary.visible_rows);
    let capacity = columns.saturating_mul(usize::from(visible_rows));
    if let Some(summary) = summary
        && slot == capacity
    {
        render_overflow_summary(
            frame,
            inner,
            top_rows.saturating_add(visible_rows),
            summary,
            palette,
        );
        return;
    }
    if slot >= capacity {
        return;
    }
    let Ok(columns_u16) = u16::try_from(columns) else {
        return;
    };
    let column_width = inner.width / columns_u16.max(1);
    let row = slot / columns;
    let column = slot % columns;
    let Ok(row) = u16::try_from(row) else {
        return;
    };
    let Ok(column) = u16::try_from(column) else {
        return;
    };
    let y = inner.y.saturating_add(top_rows).saturating_add(row);
    if y >= inner.bottom().saturating_sub(2) {
        return;
    }
    let x = inner.x.saturating_add(column.saturating_mul(column_width));
    let marker = match model.preferences().character_set {
        CharacterSet::Unicode => "◆",
        CharacterSet::Ascii => "o",
    };
    let label = if total == 1 {
        format!(
            "{marker} {} [TOKEN / {}]",
            present(&agent.persona.name, model.preferences().character_set),
            status_label(theatre.pose, agent, model)
        )
    } else {
        format!(
            "{marker} {}",
            present(&agent.persona.name, model.preferences().character_set)
        )
    };
    frame.render_widget(
        Paragraph::new(label).style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        Rect::new(x, y, column_width, 1),
    );
}

fn render_full_figure(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    model: &Model,
    rows: RepresentationRows,
    projection: bool,
    palette: Palette,
) {
    if area.width < 4 || area.height < 4 {
        return;
    }
    let compact_horizontal = area.height < 12 && area.width >= 26;
    let inner = representation_inner(area);
    let column_width = inner.width;
    let x = inner.x;
    let y = inner.y.saturating_add(rows.top);
    if y >= inner.bottom() {
        return;
    }

    let label = present(&agent.persona.name, model.preferences().character_set);
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
    let label_area = label_area.intersection(inner);
    let status_area = status_area.intersection(inner);
    frame.render_widget(
        Paragraph::new(label.as_ref())
            .style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
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
        inner
            .bottom()
            .saturating_sub(rows.bottom)
            .saturating_sub(art_y),
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

#[allow(clippy::too_many_arguments)]
fn render_dense_figure(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    model: &Model,
    slot: usize,
    total: usize,
    top_rows: u16,
    bottom_rows: u16,
    palette: Palette,
) {
    let inner = representation_inner(area);
    let available_rows = inner
        .height
        .saturating_sub(top_rows)
        .saturating_sub(bottom_rows);
    if inner.width < 3 || available_rows == 0 {
        return;
    }
    let max_columns = usize::from((inner.width / 5).max(1));
    let preferred_columns = usize::from((inner.width / 18).max(1)).min(total.max(1));
    let needed_columns = total.max(1).div_ceil(usize::from(available_rows));
    let columns = preferred_columns
        .max(needed_columns.min(max_columns))
        .min(max_columns)
        .min(total.max(1));
    let summary = OverflowSummary::for_owner(
        total,
        columns,
        available_rows,
        inner.width,
        "ADVENTURERS",
        "AGENTS",
    );
    let visible_rows = summary.map_or(available_rows, |summary| summary.visible_rows);
    let capacity = columns.saturating_mul(usize::from(visible_rows));
    if let Some(summary) = summary
        && slot == capacity
    {
        render_overflow_summary(
            frame,
            inner,
            top_rows.saturating_add(visible_rows),
            summary,
            palette,
        );
        return;
    }
    if slot >= capacity {
        return;
    }
    let Ok(columns_u16) = u16::try_from(columns) else {
        return;
    };
    let column_width = inner.width / columns_u16.max(1);
    let row = slot / columns;
    let column = slot % columns;
    let (Ok(row), Ok(column)) = (u16::try_from(row), u16::try_from(column)) else {
        return;
    };
    let y = inner.y.saturating_add(top_rows).saturating_add(row);
    if y >= inner.bottom().saturating_sub(bottom_rows) {
        return;
    }
    let x = inner.x.saturating_add(column.saturating_mul(column_width));
    let art_width = column_width.min(2);
    match model.preferences().character_set {
        CharacterSet::Unicode => {
            let canvas = compose_chamber_adventurer_for_palette(&agent.persona, theatre, palette);
            frame.render_widget(
                Paragraph::new(pack(&canvas, &palette, ColorRole::DarkStone)),
                Rect::new(x, y, art_width, 1),
            );
        }
        CharacterSet::Ascii => frame.render_widget(
            Paragraph::new(ascii_figure(theatre.pose).remove(0)),
            Rect::new(x, y, art_width, 1),
        ),
    }
    let label_x = x.saturating_add(art_width);
    let label = present(&agent.persona.name, model.preferences().character_set);
    frame.render_widget(
        Paragraph::new(label.as_ref())
            .style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        Rect::new(label_x, y, column_width.saturating_sub(art_width), 1),
    );
}

#[derive(Clone, Copy, Debug)]
struct OverflowSummary {
    visible_rows: u16,
    count: usize,
    noun: &'static str,
    rows: u16,
}

impl OverflowSummary {
    fn for_owner(
        total: usize,
        columns: usize,
        available_rows: u16,
        width: u16,
        noun: &'static str,
        narrow_noun: &'static str,
    ) -> Option<Self> {
        let full_capacity = columns.saturating_mul(usize::from(available_rows));
        if total <= full_capacity {
            return None;
        }

        let one_line_visible_rows = available_rows.saturating_sub(1);
        let one_line_capacity = columns.saturating_mul(usize::from(one_line_visible_rows));
        let one_line_count = total.saturating_sub(one_line_capacity);
        if format!("+{one_line_count} {noun}").cell_width() <= width {
            return Some(Self {
                visible_rows: one_line_visible_rows,
                count: one_line_count,
                noun,
                rows: 1,
            });
        }

        if available_rows < 2 {
            let compact = format!("+{one_line_count} {narrow_noun}");
            debug_assert!(compact.cell_width() <= width);
            return Some(Self {
                visible_rows: one_line_visible_rows,
                count: one_line_count,
                noun: narrow_noun,
                rows: 1,
            });
        }

        let summary_rows = 2;
        let visible_rows = available_rows.saturating_sub(summary_rows);
        let capacity = columns.saturating_mul(usize::from(visible_rows));
        let count = total.saturating_sub(capacity);
        debug_assert!(format!("+{count}").cell_width() <= width);
        debug_assert!(narrow_noun.cell_width() <= width);
        Some(Self {
            visible_rows,
            count,
            noun: narrow_noun,
            rows: summary_rows,
        })
    }
}

fn render_overflow_summary(
    frame: &mut Frame<'_>,
    inner: Rect,
    row: u16,
    summary: OverflowSummary,
    palette: Palette,
) {
    let area = Rect::new(
        inner.x,
        inner.y.saturating_add(row),
        inner.width,
        summary.rows,
    );
    let text = if summary.rows == 1 {
        Text::from(format!("+{} {}", summary.count, summary.noun))
    } else {
        Text::from(vec![
            Line::from(format!("+{}", summary.count)),
            Line::from(summary.noun),
        ])
    };
    frame.render_widget(
        Paragraph::new(text).style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
        area,
    );
}

const fn representation_inner(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    )
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

fn door_lines(model: &Model) -> Vec<Cow<'_, str>> {
    let mut lines = match model.connection() {
        ConnectionState::Offline => vec![Cow::Borrowed("OFFLINE / door closed")],
        ConnectionState::Connecting => vec![Cow::Borrowed("CONNECTING / door opening")],
        ConnectionState::Connected => vec![Cow::Borrowed("CONNECTED / door open")],
        ConnectionState::Reconnecting { attempt } => vec![
            Cow::Borrowed("RECONNECTING / door barred"),
            Cow::Owned(format!("attempt {attempt}")),
        ],
        ConnectionState::Incompatible { expected, actual } => vec![
            Cow::Borrowed("INCOMPATIBLE / door sealed"),
            Cow::Owned(format!("protocol {actual}; expected {expected}")),
        ],
    };
    if !matches!(model.connection(), ConnectionState::Connected)
        && let Some(Notice::ConnectionDiagnostic(message)) = model.notice()
    {
        lines.push(Cow::Owned(format!(
            "Cause: {}",
            present(message, model.preferences().character_set)
        )));
    }
    lines
}

fn counsel_lines(projection: &GuildRoomProjection) -> Vec<Cow<'static, str>> {
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
        0 => vec![Cow::Borrowed("The bell is quiet.")],
        1 => vec![Cow::Borrowed("requests counsel")],
        count => vec![Cow::Owned(format!("{count} request counsel"))],
    }
}

fn hearth_lines<'a>(model: &'a Model, projection: &GuildRoomProjection) -> Vec<Cow<'a, str>> {
    if model.domain().agents.is_empty() {
        return vec![Cow::Borrowed(EMPTY_GUILD)];
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
        vec![Cow::Borrowed("Warm coals wait beside the communal rug.")]
    } else {
        Vec::new()
    }
}

fn chronicle_lines(model: &Model) -> Vec<Cow<'_, str>> {
    let lines = model
        .domain()
        .chronicle
        .entries()
        .iter()
        .rev()
        .take(5)
        .map(|entry| present(&entry.summary, model.preferences().character_set))
        .collect::<Vec<_>>();
    if lines.is_empty() {
        vec![Cow::Borrowed("No deeds recorded yet.")]
    } else {
        lines
    }
}

fn scrying_lines(model: &Model) -> Vec<Cow<'_, str>> {
    let Some(agent) = model.selected_agent() else {
        return vec![Cow::Borrowed(SCRYING_STILL)];
    };
    let mut lines = vec![Cow::Owned(format!(
        "SELECTED PANE: {}",
        present(&agent.name, model.preferences().character_set)
    ))];
    if model.managed_pane_id() == Some(&agent.pane_id) {
        lines.push(Cow::Borrowed(SCRYING_STILL));
        lines.push(Cow::Borrowed(
            "Cause: the Questmancer's own pane is never observed.",
        ));
        return lines;
    }
    match model
        .output_preview()
        .filter(|preview| preview.pane_id == agent.pane_id)
    {
        Some(preview) if preview.error.is_some() => {
            lines.push(Cow::Borrowed("The scrying pool has clouded."));
            lines.push(Cow::Owned(format!(
                "Cause: {}",
                present(
                    preview.error.as_deref().unwrap_or_default(),
                    model.preferences().character_set
                )
            )));
        }
        Some(preview) if preview.loading => {
            lines.push(Cow::Borrowed(SCRYING_STILL));
            lines.push(Cow::Borrowed("Tracing the selected adventurer..."));
        }
        Some(preview) => lines.extend(
            preview
                .text
                .lines()
                .map(|line| present(line, model.preferences().character_set)),
        ),
        None => {
            lines.push(Cow::Borrowed(SCRYING_STILL));
            lines.push(Cow::Borrowed("Select refresh to trace recent deeds."));
        }
    }
    lines
}

fn spoils_lines(model: &Model) -> Vec<Cow<'_, str>> {
    let mut lines = Vec::new();
    let actionable = model.selected_agent().is_some_and(|agent| {
        model.reviewr_available() && model.managed_pane_id() != Some(&agent.pane_id)
    });
    if actionable {
        lines.push(Cow::Borrowed("REVIEWR READY"));
        lines.push(Cow::Borrowed("[v] Inspect spoils"));
    }
    if let Some(Notice::IntegrationDiagnostic(message)) = model.notice() {
        lines.push(present(message, model.preferences().character_set));
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
    index: &RenderIndex<'_>,
    theme: LandmarkTheme,
) -> EffectCells {
    if !model.goblins().is_visible(model.now()) || projection.adventurers.is_empty() {
        return EffectCells::default();
    }
    let Some(chronicle) = index.landmark(&GuildLandmark::Chronicle) else {
        return EffectCells::default();
    };
    let before = frame.buffer_mut().clone();
    render_chronicle_marginalia(frame, chronicle, theme);
    EffectCells::changed_between(&before, frame.buffer_mut(), chronicle.area)
}
