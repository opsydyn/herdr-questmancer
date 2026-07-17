use std::borrow::Cow;

use ratatui::{
    Frame,
    buffer::CellWidth,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::{Paragraph, Wrap},
};

use crate::{
    app::CharacterSet,
    ui::{
        guild_room_projection::{ProjectedCampaignTable, ProjectedLandmark},
        pixel::{ColorRole, Palette},
        widgets::presentation::present,
    },
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub(crate) enum LandmarkLayer {
    Furniture,
    Effects,
    Labels,
}

#[derive(Clone, Copy, Debug)]
pub(crate) struct LandmarkTheme {
    pub(crate) character_set: CharacterSet,
    pub(crate) palette: Palette,
}

pub(crate) fn render_door(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => {
            if landmark.area.height >= 10 {
                render_bottom_art(
                    frame,
                    landmark.area,
                    art(
                        theme.character_set,
                        &["╭──────╮", "│  ()  │", "╰──┬───╯"],
                        &["+------+", "|  ()  |", "+--+---+"],
                    ),
                    role_style(theme, ColorRole::Timber),
                );
            }
        }
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "GUILD DOOR", theme);
            render_content(frame, landmark.area, 1, lines, theme);
        }
    }
}

pub(crate) fn render_quest_wall(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    campaigns: &[ProjectedCampaignTable],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &["╞════ map rail ════╡", "  scrolls · wax seals  "],
                &["+==== map rail ====+", "  scrolls / wax seals  "],
            ),
            role_style(theme, ColorRole::Parchment),
        ),
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "QUEST WALL", theme);
            render_line(
                frame,
                landmark.area,
                1,
                "MAPS / COMMISSIONS",
                role_style(theme, ColorRole::Parchment),
            );
            let banners = if campaigns.is_empty() {
                vec![Cow::Borrowed("No campaign banners are hung.")]
            } else {
                campaigns
                    .iter()
                    .map(|campaign| {
                        Cow::Owned(format!(
                            "CAMPAIGN BANNER: {}  SEAL {:04X}",
                            present(&campaign.label, theme.character_set),
                            campaign.seal & 0xFFFF
                        ))
                    })
                    .collect()
            };
            render_content(frame, landmark.area, 2, &banners, theme);
        }
    }
}

pub(crate) fn render_counsel_bell(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &["      ╱╲", "     (  )", "──────╨──────"],
                &["      /\\", "     (  )", "------||------"],
            ),
            role_style(theme, ColorRole::Counsel),
        ),
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "COUNSEL BELL", theme);
            render_content(frame, landmark.area, 1, lines, theme);
        }
    }
}

pub(crate) fn render_hearth(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &[
                    "              ╭─ hearth ─╮    ( ) mugs      bedrolls ╱___╱",
                    "              │   (♨)    │    rugs · books · warm stone",
                ],
                &[
                    "              +-- hearth -+    ( ) mugs      bedrolls /___/",
                    "              |   (^^)    |    rugs / books / warm stone",
                ],
            ),
            role_style(theme, ColorRole::Hearth),
        ),
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "HEARTH", theme);
            render_line(
                frame,
                landmark.area,
                1,
                "MUGS / BEDROLLS",
                role_style(theme, ColorRole::Parchment),
            );
            render_content(frame, landmark.area, 2, lines, theme);
        }
    }
}

pub(crate) fn render_chronicle_lectern(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &["   ╱ open book ╲", "  ╱_____╲  candle"],
                &["   / open book /", "  /_____\\  candle"],
            ),
            role_style(theme, ColorRole::Parchment),
        ),
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "CHRONICLE LECTERN", theme);
            render_content(frame, landmark.area, 1, lines, theme);
        }
    }
}

pub(crate) fn render_scrying_alcove(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &["   ◇ scrying mirror ◇", "  🕯  books  pool  🕯"],
                &["   <> scry mirror <>", "  *  books  pool  *"],
            ),
            role_style(theme, ColorRole::RuneGlow),
        ),
        LandmarkLayer::Effects => {
            if landmark.illuminated {
                render_line(
                    frame,
                    landmark.area,
                    2,
                    "[LIT] LIVE MIRROR",
                    role_style(theme, ColorRole::Selection),
                );
            }
        }
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "SCRYING ALCOVE", theme);
            let caption = if landmark_inner(landmark.area).width < 24 {
                "MIRROR / CANDLES"
            } else {
                "MIRROR / CANDLES / BOOKS"
            };
            render_line(
                frame,
                landmark.area,
                1,
                caption,
                role_style(theme, ColorRole::Parchment),
            );
            render_content(frame, landmark.area, 3, lines, theme);
        }
    }
}

pub(crate) fn render_spoils_desk(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    layer: LandmarkLayer,
    lines: &[Cow<'_, str>],
    max_content_rows: u16,
    theme: LandmarkTheme,
) {
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            landmark.area,
            art(
                theme.character_set,
                &[
                    "              ┌ ledger ┐  ▣ lockbox  ( ) mug",
                    "              └────────┘  quiet oak desk",
                ],
                &[
                    "              + ledger +  # lockbox  ( ) mug",
                    "              +--------+  quiet oak desk",
                ],
            ),
            role_style(theme, ColorRole::Spoils),
        ),
        LandmarkLayer::Effects => {}
        LandmarkLayer::Labels => {
            render_heading(frame, landmark.area, "SPOILS DESK", theme);
            render_line(
                frame,
                landmark.area,
                1,
                "LEDGER / LOCKBOX / MUG",
                role_style(theme, ColorRole::Parchment),
            );
            render_measured_content(frame, landmark.area, 2, lines, max_content_rows, theme);
        }
    }
}

pub(crate) fn render_campaign_table(
    frame: &mut Frame<'_>,
    campaign: &ProjectedCampaignTable,
    layer: LandmarkLayer,
    theme: LandmarkTheme,
) {
    let compact_identity = campaign.area.width <= 14;
    match layer {
        LandmarkLayer::Furniture => render_bottom_art(
            frame,
            campaign.area,
            art(
                theme.character_set,
                &[
                    "  ╭──── expedition map ────╮",
                    "  ╰─ mugs · dice · notes ──╯",
                ],
                &[
                    "  +---- expedition map ----+",
                    "  +- mugs / dice / notes --+",
                ],
            ),
            role_style(theme, ColorRole::Timber),
        ),
        LandmarkLayer::Effects => {
            if campaign.selected {
                render_line(
                    frame,
                    campaign.area,
                    if compact_identity { 3 } else { 2 },
                    "[LIT] SELECTED LAMP",
                    role_style(theme, ColorRole::Selection),
                );
            }
            if campaign.illuminated {
                render_line(
                    frame,
                    campaign.area,
                    if compact_identity { 4 } else { 3 },
                    "[LIVE] FOCUSED EXPEDITION",
                    role_style(theme, ColorRole::RuneGlow),
                );
            }
        }
        LandmarkLayer::Labels => {
            if compact_identity {
                render_heading(frame, campaign.area, "TABLE", theme);
                render_line(
                    frame,
                    campaign.area,
                    1,
                    present(&campaign.label, theme.character_set).as_ref(),
                    role_style(theme, ColorRole::Parchment),
                );
            } else {
                render_heading(
                    frame,
                    campaign.area,
                    &format!(
                        "CAMPAIGN TABLE: {}",
                        present(&campaign.label, theme.character_set)
                    ),
                    theme,
                );
            }
            render_line(
                frame,
                campaign.area,
                if compact_identity { 2 } else { 1 },
                &format!("#{:04X}", campaign.seal & 0xFFFF),
                role_style(theme, ColorRole::Parchment),
            );
        }
    }
}

pub(crate) fn render_chronicle_marginalia(
    frame: &mut Frame<'_>,
    landmark: &ProjectedLandmark,
    theme: LandmarkTheme,
) {
    let width = landmark.area.width.saturating_sub(2);
    if width == 0 || landmark.area.height == 0 {
        return;
    }
    // The upper lintel is architecture, not content. Keeping the transient
    // marginalia here preserves every factual Chronicle row and its furniture.
    frame.render_widget(
        Paragraph::new("CREATURES DETECTED").style(role_style(theme, ColorRole::Goblin)),
        Rect::new(landmark.area.x.saturating_add(1), landmark.area.y, width, 1),
    );
}

fn render_heading(frame: &mut Frame<'_>, area: Rect, heading: &str, theme: LandmarkTheme) {
    render_line(
        frame,
        area,
        0,
        heading,
        role_style(theme, ColorRole::Parchment),
    );
}

fn render_content(
    frame: &mut Frame<'_>,
    area: Rect,
    row: u16,
    lines: &[Cow<'_, str>],
    theme: LandmarkTheme,
) {
    let inner = landmark_inner(area);
    if inner.is_empty() || row >= inner.height {
        return;
    }
    let content_area = Rect::new(
        inner.x,
        inner.y.saturating_add(row),
        inner.width,
        inner.height.saturating_sub(row),
    );
    frame.render_widget(
        Paragraph::new(Text::from(
            lines
                .iter()
                .map(|line| Line::from(line.as_ref()))
                .collect::<Vec<_>>(),
        ))
        .style(role_style(theme, ColorRole::Parchment))
        .wrap(Wrap { trim: false }),
        content_area,
    );
}

pub(crate) fn content_visual_height(area: Rect, row: u16, lines: &[Cow<'_, str>]) -> u16 {
    let inner = landmark_inner(area);
    if inner.is_empty() || row >= inner.height {
        return 0;
    }
    u16::try_from(wrapped_content_lines(lines, inner.width).len()).unwrap_or(u16::MAX)
}

fn render_measured_content(
    frame: &mut Frame<'_>,
    area: Rect,
    row: u16,
    lines: &[Cow<'_, str>],
    max_rows: u16,
    theme: LandmarkTheme,
) {
    let inner = landmark_inner(area);
    if inner.is_empty() || row >= inner.height {
        return;
    }
    let rows = wrapped_content_lines(lines, inner.width);
    frame.render_widget(
        Paragraph::new(Text::from(
            rows.iter()
                .take(usize::from(max_rows))
                .map(|line| Line::from(line.as_str()))
                .collect::<Vec<_>>(),
        ))
        .style(role_style(theme, ColorRole::Parchment)),
        Rect::new(
            inner.x,
            inner.y.saturating_add(row),
            inner.width,
            inner.height.saturating_sub(row),
        ),
    );
}

fn wrapped_content_lines(lines: &[Cow<'_, str>], width: u16) -> Vec<String> {
    if width == 0 {
        return Vec::new();
    }
    lines
        .iter()
        .flat_map(|line| wrap_content_line(line, width))
        .collect()
}

fn wrap_content_line(line: &str, width: u16) -> Vec<String> {
    if line.is_empty() {
        return vec![String::new()];
    }
    let mut rows = Vec::new();
    let mut row = String::new();
    let mut row_width = 0_u16;
    for word in line.split_whitespace() {
        let word_width = word.cell_width();
        let additional = word_width.saturating_add(u16::from(!row.is_empty()));
        if !row.is_empty() && row_width.saturating_add(additional) > width {
            rows.push(std::mem::take(&mut row));
            row_width = 0;
        }
        if word_width > width {
            for character in word.chars() {
                let mut encoded = [0; 4];
                let character_width = character.encode_utf8(&mut encoded).cell_width();
                if !row.is_empty() && row_width.saturating_add(character_width) > width {
                    rows.push(std::mem::take(&mut row));
                    row_width = 0;
                }
                row.push(character);
                row_width = row_width.saturating_add(character_width);
            }
        } else {
            if !row.is_empty() {
                row.push(' ');
                row_width = row_width.saturating_add(1);
            }
            row.push_str(word);
            row_width = row_width.saturating_add(word_width);
        }
    }
    if !row.is_empty() {
        rows.push(row);
    }
    rows
}

fn render_line(frame: &mut Frame<'_>, area: Rect, row: u16, text: &str, style: Style) {
    let inner = landmark_inner(area);
    if inner.is_empty() || row >= inner.height {
        return;
    }
    frame.render_widget(
        Paragraph::new(text).style(style),
        Rect::new(inner.x, inner.y.saturating_add(row), inner.width, 1),
    );
}

fn render_bottom_art(frame: &mut Frame<'_>, area: Rect, rows: &[&'static str], style: Style) {
    let inner = landmark_inner(area);
    if inner.is_empty() {
        return;
    }
    let height = u16::try_from(rows.len())
        .unwrap_or(u16::MAX)
        .min(inner.height);
    let start = inner.y.saturating_add(inner.height.saturating_sub(height));
    frame.render_widget(
        Paragraph::new(Text::from(
            rows.iter().copied().map(Line::from).collect::<Vec<_>>(),
        ))
        .style(style),
        Rect::new(inner.x, start, inner.width, height),
    );
}

const fn landmark_inner(area: Rect) -> Rect {
    Rect::new(
        area.x.saturating_add(1),
        area.y.saturating_add(1),
        area.width.saturating_sub(2),
        area.height.saturating_sub(2),
    )
}

const fn role_style(theme: LandmarkTheme, role: ColorRole) -> Style {
    Style::new().fg(theme.palette.resolve(role))
}

const fn art(
    character_set: CharacterSet,
    unicode: &'static [&'static str],
    ascii: &'static [&'static str],
) -> &'static [&'static str] {
    match character_set {
        CharacterSet::Unicode => unicode,
        CharacterSet::Ascii => ascii,
    }
}
