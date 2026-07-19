use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};

use crate::{
    app::{Modal, Model},
    scene::presentation::{SceneOverlay, ScenePresentation},
    ui::theme::{PARCHMENT, PARCHMENT_BORDER},
};

pub fn render_scene_overlays(
    frame: &mut Frame<'_>,
    model: &Model,
    presentation: &ScenePresentation,
) {
    match presentation.overlay {
        SceneOverlay::Counsel | SceneOverlay::Search => render_input_parchment(frame, model),
        SceneOverlay::Help => render_help_parchment(frame),
        SceneOverlay::Scrying => render_scrying_parchment(frame, model),
        SceneOverlay::None if model.command_ribbon_visible() => render_command_ribbon(frame, model),
        SceneOverlay::None => {}
    }
}

fn render_input_parchment(frame: &mut Frame<'_>, model: &Model) {
    let (title, input, keys) = match model.modal() {
        Modal::Counsel { draft } => (" ISSUE COUNSEL ", draft.as_str(), "Enter send  Esc cancel"),
        Modal::Search { query } => (
            " SEARCH THE GUILD ",
            query.as_str(),
            "Enter find  Esc cancel",
        ),
        _ => return,
    };
    let Some(area) = centered(frame.area(), 64, 9) else {
        return;
    };
    let lines = vec![
        Line::from(""),
        Line::from(input.to_owned()),
        Line::from(""),
        Line::from(keys),
    ];
    render_parchment(frame, area, title, Text::from(lines));
}

fn render_help_parchment(frame: &mut Frame<'_>) {
    let Some(area) = centered(frame.area(), 72, 15) else {
        return;
    };
    let lines = vec![
        Line::from("[1/2] Guild Hall / Delve"),
        Line::from("[j/k or arrows] Select adventurer"),
        Line::from("[Enter] Observe selected adventurer"),
        Line::from("[r] Issue counsel"),
        Line::from("[/] Search the guild"),
        Line::from("[o] Open the scrying parchment"),
        Line::from("[v] Inspect spoils"),
        Line::from("[Esc/?] Close guide"),
    ];
    render_parchment(
        frame,
        area,
        " QUESTMANCER'S FIELD GUIDE ",
        Text::from(lines),
    );
}

fn render_scrying_parchment(frame: &mut Frame<'_>, model: &Model) {
    let Some(area) = centered(frame.area(), 80, 18) else {
        return;
    };
    let body = model.output_preview().map_or_else(
        || "The scrying pool is still.".to_owned(),
        |preview| {
            preview.error.clone().unwrap_or_else(|| {
                if preview.loading {
                    "The scrying pool is clouding...".to_owned()
                } else {
                    preview.text.clone()
                }
            })
        },
    );
    render_parchment(
        frame,
        area,
        " SCRYING ",
        Text::from(vec![
            Line::from(body),
            Line::from(""),
            Line::from("Esc close  o refresh"),
        ]),
    );
}

fn render_command_ribbon(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();
    if area.width < 20 || area.height == 0 {
        return;
    }
    let y = area.bottom().saturating_sub(1);
    let ribbon = Rect::new(area.x, y, area.width, 1);
    let counsel = model.selected_agent().map_or("", |_| "  [r] Counsel");
    let text = format!("[1] Guild  [2] Delve  [j/k] Select  [Enter] Observe{counsel}  [/] Search");
    frame.render_widget(Clear, ribbon);
    frame.render_widget(Paragraph::new(text).style(PARCHMENT_BORDER), ribbon);
}

fn render_parchment(frame: &mut Frame<'_>, area: Rect, title: &str, text: Text<'_>) {
    frame.render_widget(Clear, area);
    let block = Block::default()
        .title(title)
        .title_alignment(Alignment::Center)
        .borders(Borders::ALL)
        .style(PARCHMENT)
        .border_style(PARCHMENT_BORDER);
    frame.render_widget(
        Paragraph::new(text)
            .block(block)
            .style(PARCHMENT)
            .wrap(Wrap { trim: false }),
        area,
    );
}

fn centered(area: Rect, maximum_width: u16, maximum_height: u16) -> Option<Rect> {
    if area.width < 8 || area.height < 5 {
        return None;
    }
    let width = area.width.saturating_sub(4).min(maximum_width);
    let height = area.height.saturating_sub(2).min(maximum_height);
    Some(Rect::new(
        area.x + (area.width - width) / 2,
        area.y + (area.height - height) / 2,
        width,
        height,
    ))
}
