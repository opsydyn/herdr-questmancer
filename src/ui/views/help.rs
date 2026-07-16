use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    app::{CharacterSet, Model},
    ui::theme::{ACCENT, INK, MUTED},
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

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let frame_area = frame.area();
    if frame_area.is_empty() {
        return;
    }
    if frame_area.width < 8 || frame_area.height < 5 {
        frame.render_widget(Clear, frame_area);
        frame.render_widget(Paragraph::new("?").style(ACCENT), frame_area);
        return;
    }

    let width = frame_area.width.saturating_sub(4).min(72);
    let height = frame_area.height.saturating_sub(2).min(18);
    let area = Rect::new(
        frame_area.x + (frame_area.width - width) / 2,
        frame_area.y + (frame_area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);

    let lines = vec![
        Line::styled(
            "Your agents have entered the dungeon. You are the Questmancer.",
            ACCENT,
        ),
        Line::from(""),
        Line::from("[1/F1] Guild Hall    [2/F2] Delves"),
        Line::from("[j/k] Choose adventurer    [g/G] First / last"),
        Line::from("[enter] Observe selected adventurer"),
        Line::from("[r] Issue Counsel"),
        Line::from("[space] Acknowledge Summons"),
        Line::from("[o] Scry Again"),
        Line::from("[/] Search adventurers and campaigns"),
        Line::from("[v] Inspect Spoils"),
        Line::from("[tab] Cycle Guild Hall region"),
        Line::from(""),
        Line::styled("[esc/?] Close guide", MUTED),
    ];
    let mut block = Block::default()
        .title(" QUESTMANCER'S FIELD GUIDE ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(ACCENT);
    if model.preferences().character_set == CharacterSet::Ascii {
        block = block.border_set(ASCII_BORDER);
    }
    frame.render_widget(
        Paragraph::new(Text::from(lines))
            .block(block)
            .style(INK)
            .wrap(Wrap { trim: false }),
        area,
    );
}
