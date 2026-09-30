use crate::{
    app::Model,
    ui::theme::{PARCHMENT, PARCHMENT_BORDER},
    work::view,
};
use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::Text,
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
fn area(width: u16, height: u16) -> Rect {
    let w = width.saturating_sub(4).min(110);
    let h = height.saturating_sub(2).min(36);
    Rect::new((width - w) / 2, (height - h) / 2, w, h)
}
fn detail_width(width: u16) -> u16 {
    let w = area(width, 40).width.saturating_sub(2);
    if w >= 72 { w.saturating_sub(32) } else { w }
}
pub fn wrapped_details(model: &Model, width: u16) -> Vec<String> {
    wrap_text(&view::details(model).join("\n"), detail_width(width))
}
pub(crate) fn wrap_text(text: &str, width: u16) -> Vec<String> {
    let width = usize::from(width.max(1));
    let mut result = Vec::new();
    for line in text.lines() {
        let mut row = String::new();
        let mut cells = 0;
        for word in line.split_inclusive(' ') {
            let length = ratatui::text::Line::raw(word).width();
            if cells > 0 && cells + length > width {
                result.push(row.trim_end().to_owned());
                row.clear();
                cells = 0;
            }
            for c in word.chars() {
                let length = ratatui::text::Span::raw(c.to_string()).width();
                if cells > 0 && cells + length > width {
                    result.push(std::mem::take(&mut row));
                    cells = 0;
                }
                row.push(c);
                cells += length;
            }
        }
        result.push(row.trim_end().to_owned());
    }
    result
}
pub fn detail_line_count(model: &Model, width: u16) -> usize {
    wrapped_details(model, width).len().max(1)
}

fn inventory_rows(model: &Model, height: u16) -> String {
    let mut rows = Vec::new();
    let mut selected = 0..0;
    for item in view::items(model) {
        let is_selected = model.work_selection() == Some(&item);
        let start = rows.len();
        for (i, line) in wrap_text(&view::label(model, &item), 28)
            .into_iter()
            .enumerate()
        {
            let marker = if is_selected && i == 0 { ">" } else { " " };
            rows.push(format!("{marker} {line}"));
        }
        if is_selected {
            selected = start..rows.len();
        }
    }
    let height = usize::from(height);
    let mut start = selected.start.saturating_sub(height / 2);
    if selected.end > start + height {
        start = selected.end.saturating_sub(height);
    }
    if selected.len() > height {
        start = selected.start;
    }
    rows.into_iter()
        .skip(start)
        .take(height)
        .collect::<Vec<_>>()
        .join("\n")
}

pub fn render(frame: &mut Frame<'_>, model: &Model) {
    let viewport = frame.area();
    if viewport.width < 8 || viewport.height < 6 {
        frame.render_widget(Paragraph::new("Quest Board").style(PARCHMENT), viewport);
        return;
    }
    let mut area = area(viewport.width, viewport.height);
    area.x += viewport.x;
    area.y += viewport.y;
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(" QUEST BOARD ")
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(PARCHMENT)
        .border_style(PARCHMENT_BORDER);
    let inner = block.inner(area);
    frame.render_widget(block, area);
    let items = view::items(model);
    let index = model
        .work_selection()
        .and_then(|s| items.iter().position(|i| i == s));
    let heading = format!(
        "{} need you | Work {}/{}",
        crate::work::queue(model).len(),
        index.map_or(0, |i| i + 1),
        items.len()
    );
    frame.render_widget(
        Paragraph::new(heading).style(PARCHMENT_BORDER),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );
    let footer_height = if inner.width < 72 { 4 } else { 2 }.min(inner.height.saturating_sub(1));
    let body = Rect::new(
        inner.x,
        inner.y + 1,
        inner.width,
        inner.height.saturating_sub(footer_height + 1),
    );
    let details = if inner.width >= 72 {
        let list = Rect::new(body.x, body.y, 30, body.height);
        let rows = inventory_rows(model, body.height);
        frame.render_widget(Paragraph::new(rows).style(PARCHMENT_BORDER), list);
        Rect::new(
            body.x + 32,
            body.y,
            body.width.saturating_sub(32),
            body.height,
        )
    } else {
        body
    };
    let rows = wrapped_details(model, viewport.width);
    let scroll = usize::from(model.reading_scroll()).min(rows.len().saturating_sub(1));
    frame.render_widget(
        Paragraph::new(Text::from(
            rows.into_iter().skip(scroll).collect::<Vec<_>>().join("\n"),
        ))
        .style(PARCHMENT),
        details,
    );
    let footer = if inner.width < 72 {
        "j/k: work  Up/Down: read\n!: next summons\nSpace: seen  s: defer 15m\nEnter: observe  Esc: close"
    } else {
        "j/k or Tab: work   Up/Down: read   !: next summons\nSpace: seen   s: defer 15m   Enter: observe verified pane   Esc: close"
    };
    frame.render_widget(
        Paragraph::new(footer)
            .style(PARCHMENT_BORDER)
            .wrap(Wrap { trim: false }),
        Rect::new(
            inner.x,
            inner.bottom().saturating_sub(footer_height),
            inner.width,
            footer_height,
        ),
    );
}
