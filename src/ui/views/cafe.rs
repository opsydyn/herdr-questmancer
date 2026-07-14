use ratatui::{
    Frame,
    layout::{Alignment, Constraint, Layout},
    text::{Line, Text},
    widgets::{Block, Borders, Paragraph},
};

use crate::ui::theme::{ACCENT, INK, MUTED};

pub(crate) fn render(frame: &mut Frame<'_>) {
    let area = frame.area();
    if area.width < 4 || area.height < 3 {
        frame.render_widget(Paragraph::new("C").style(ACCENT), area);
        return;
    }

    let [body, footer] = Layout::vertical([Constraint::Min(1), Constraint::Length(1)]).areas(area);
    let block = Block::default()
        .title(" THE HERDR CYBERCAFE ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .border_style(ACCENT)
        .style(INK);
    let inner = block.inner(body);
    frame.render_widget(block, body);

    let message = Text::from(vec![
        Line::from(""),
        Line::styled("All workstations are free", ACCENT),
        Line::from(""),
        Line::styled("connected at 56,600 bps  |  sites online: 0", MUTED),
    ]);
    frame.render_widget(
        Paragraph::new(message)
            .alignment(Alignment::Center)
            .style(INK),
        inner,
    );
    frame.render_widget(
        Paragraph::new("[1] desk  [2] cafe  [?] help  [q] quit")
            .alignment(Alignment::Center)
            .style(MUTED),
        footer,
    );
}
