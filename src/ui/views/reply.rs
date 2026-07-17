use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    symbols::border,
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    app::{CharacterSet, Modal, Model, Notice},
    ui::{
        theme::{ACCENT, INK, MUTED},
        widgets::presentation::present,
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

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let (title, input, keys) = match model.modal() {
        Modal::Counsel { draft } => (
            " ISSUE COUNSEL ",
            draft.as_str(),
            "[enter] send   [esc] cancel   [ctrl-u] clear",
        ),
        Modal::Search { query } => (
            " SEARCH ADVENTURERS ",
            query.as_str(),
            "[enter] find   [esc] cancel   [ctrl-u] clear",
        ),
        Modal::None | Modal::Help => return,
    };
    let frame_area = frame.area();
    if frame_area.width < 8 || frame_area.height < 5 {
        return;
    }
    let width = frame_area.width.saturating_sub(4).min(64);
    let height = frame_area.height.saturating_sub(2).min(9);
    let area = Rect::new(
        frame_area.x + (frame_area.width - width) / 2,
        frame_area.y + (frame_area.height - height) / 2,
        width,
        height,
    );
    frame.render_widget(Clear, area);
    let character_set = model.preferences().character_set;
    let mut lines = vec![
        Line::from(""),
        Line::from(present(input, character_set).into_owned()),
    ];
    if let Some(status) = modal_notice_message(model) {
        lines.push(Line::styled(
            present(status, character_set).into_owned(),
            MUTED,
        ));
    }
    lines.extend([Line::from(""), Line::styled(keys, MUTED)]);
    let text = Text::from(lines);
    let mut block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(ACCENT);
    if character_set == CharacterSet::Ascii {
        block = block.border_set(ASCII_BORDER);
    }
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .style(INK)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn modal_notice_message(model: &Model) -> Option<&str> {
    match model.notice() {
        Some(
            Notice::ActionFeedback(message)
            | Notice::PersistenceDiagnostic(message)
            | Notice::ReviewrAvailabilityDiagnostic(message)
            | Notice::IntegrationDiagnostic(message),
        ) => Some(message),
        Some(Notice::ConnectionDiagnostic(_)) | None => None,
    }
}
