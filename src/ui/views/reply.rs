use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    app::{Modal, Model},
    ui::theme::{ACCENT, INK, MUTED},
};

pub(crate) fn render(frame: &mut Frame<'_>, model: &Model) {
    let (title, input, keys) = match model.modal() {
        Modal::Reply { draft } => (
            " SHOUT OVER ",
            draft.as_str(),
            "[enter] send   [esc] cancel   [ctrl-u] clear",
        ),
        Modal::Search { query } => (
            " SEARCH AGENTS ",
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
    let mut lines = vec![Line::from(""), Line::from(input.to_owned())];
    if let Some(status) = model.status_message() {
        lines.push(Line::styled(status.to_owned(), MUTED));
    }
    lines.extend([Line::from(""), Line::styled(keys, MUTED)]);
    let text = Text::from(lines);
    frame.render_widget(
        Paragraph::new(text)
            .block(
                Block::default()
                    .title(title)
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(ACCENT),
            )
            .style(INK)
            .wrap(Wrap { trim: false }),
        area,
    );
}
