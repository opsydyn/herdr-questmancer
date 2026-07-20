use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    text::{Line, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use ratatui_image::Image;

use crate::{
    app::{Modal, Model},
    domain::Presence,
    portrait::PortraitGallery,
    scene::{
        SceneFrame,
        assets::adventurer::adventurer_portrait_frame,
        pixel::{PixelPoint, Rgb, RgbBuffer},
        presentation::{SceneOverlay, ScenePresentation},
        sprite::blit,
    },
    ui::{
        scene_adapter::flush_rgb,
        theme::{PARCHMENT, PARCHMENT_BORDER},
    },
};

pub fn render_scene_identity_labels(frame: &mut Frame<'_>, model: &Model, scene: &SceneFrame) {
    let area = frame.area();
    if area.width < 40 || area.height < 8 {
        return;
    }
    let mut occupied = Vec::new();
    for region in &scene.actors {
        let Some(agent) = model.domain().agents.get(&region.agent) else {
            continue;
        };
        let elapsed = format_elapsed(agent.presence_since.elapsed_until(model.now()));
        let label = compact_label(
            &format!(
                "{} · {} {elapsed}",
                agent.name,
                presence_badge(agent.presence)
            ),
            36,
        );
        let width = u16::try_from(label.chars().count())
            .unwrap_or(u16::MAX)
            .min(area.width);
        if width == 0 {
            continue;
        }
        let actor_centre = region.bounds.x + i32::from(region.bounds.width) / 2;
        let preferred_x = actor_centre - i32::from(width) / 2;
        let maximum_x = i32::from(area.right().saturating_sub(width));
        let x = preferred_x.clamp(i32::from(area.x), maximum_x.max(i32::from(area.x)));
        let actor_row = region.bounds.y.div_euclid(2);
        let below = (region.bounds.y + i32::from(region.bounds.height) + 1).div_euclid(2);
        let candidates = [actor_row - 1, below];
        let Some(label_area) = candidates.into_iter().find_map(|y| {
            let y = u16::try_from(y).ok()?;
            if y < area.y || y >= area.bottom().saturating_sub(1) {
                return None;
            }
            let candidate = Rect::new(u16::try_from(x).ok()?, y, width, 1);
            (!occupied
                .iter()
                .any(|other| rects_intersect(candidate, *other)))
            .then_some(candidate)
        }) else {
            continue;
        };
        occupied.push(label_area);
        let selected = model.selected_agent_key() == Some(&region.agent);
        let style = if selected {
            PARCHMENT
        } else {
            PARCHMENT_BORDER
        };
        frame.render_widget(Paragraph::new(label).style(style), label_area);
    }
}

fn rects_intersect(left: Rect, right: Rect) -> bool {
    left.x < right.right()
        && left.right() > right.x
        && left.y < right.bottom()
        && left.bottom() > right.y
}

fn compact_label(label: &str, maximum: usize) -> String {
    if label.chars().count() <= maximum {
        return label.to_owned();
    }
    let mut compact = label
        .chars()
        .take(maximum.saturating_sub(1))
        .collect::<String>();
    compact.push('…');
    compact
}

fn presence_badge(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "WORKING",
        Presence::Blocked => "! NEEDS COUNSEL",
        Presence::Done => "✓ COMPLETED",
        Presence::Idle => "RESTING",
        Presence::Exited => "× DEPARTED",
        Presence::Unknown => "? UNKNOWN",
    }
}

pub fn render_scene_overlays(
    frame: &mut Frame<'_>,
    model: &Model,
    presentation: &ScenePresentation,
    portraits: Option<&PortraitGallery>,
) {
    match presentation.overlay {
        SceneOverlay::Counsel | SceneOverlay::Search => render_input_parchment(frame, model),
        SceneOverlay::Help => render_help_parchment(frame),
        SceneOverlay::Scrying => render_scrying_parchment(frame, model),
        SceneOverlay::None => {
            render_adventurer_card(frame, model, portraits);
            if model.command_ribbon_visible() {
                render_command_ribbon(frame, model);
            }
        }
    }
}

fn render_adventurer_card(
    frame: &mut Frame<'_>,
    model: &Model,
    portraits: Option<&PortraitGallery>,
) {
    if !model.adventurer_card_visible() {
        return;
    }
    let Some(agent) = model.selected_agent() else {
        return;
    };
    let area = frame.area();
    if area.width < 60 || area.height < 14 {
        return;
    }
    let detailed = area.width >= 96 && area.height >= 20;
    let width = area
        .width
        .saturating_sub(4)
        .min(if detailed { 78 } else { 48 });
    let height = (if detailed { 18 } else { 13 }).min(area.height.saturating_sub(2));
    let card = Rect::new(area.right() - width - 1, area.y + 1, width, height);
    let campaign = model
        .domain()
        .campaigns
        .get(&agent.workspace_id)
        .map_or(agent.workspace_id.as_str(), |campaign| {
            campaign.label.as_str()
        });
    let elapsed = format_elapsed(agent.presence_since.elapsed_until(model.now()));
    let status = presence_label(agent.presence);
    let role = format!("{:?} {:?}", agent.persona.ancestry, agent.persona.class);
    let message = agent
        .custom_status
        .as_deref()
        .unwrap_or("No current field report.");
    let lines = vec![
        Line::from(agent.persona.name.clone()),
        Line::from(format!("{role} · {}", agent.persona.epithet.as_str())),
        Line::from(""),
        Line::from(format!("Agent: {}", agent.name)),
        Line::from(format!("Campaign: {campaign}")),
        Line::from(format!("{status} · {elapsed}")),
        Line::from(message.to_owned()),
        Line::from(""),
        Line::from("Esc close · Enter observe · r counsel · o scry"),
    ];
    if detailed {
        render_portrait_card(frame, card, &agent.persona, Text::from(lines), portraits);
    } else {
        render_parchment(frame, card, " ADVENTURER ", Text::from(lines));
    }
}

fn render_portrait_card(
    frame: &mut Frame<'_>,
    area: Rect,
    persona: &crate::domain::AdventurerPersona,
    text: Text<'_>,
    portraits: Option<&PortraitGallery>,
) {
    const PARCHMENT_RGB: Rgb = Rgb::new(230, 207, 154);
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::default()
            .title(" ADVENTURER ")
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .style(PARCHMENT)
            .border_style(PARCHMENT_BORDER),
        area,
    );

    let portrait_area = Rect::new(area.x + 2, area.y + 1, 24, 16);
    frame.render_widget(Block::default().style(PARCHMENT), portrait_area);
    if let Some(portrait) = portraits.and_then(|gallery| gallery.portrait_for(persona)) {
        frame.render_widget(Image::new(portrait), portrait_area);
    } else {
        let mut pixels = RgbBuffer::filled(24, 32, PARCHMENT_RGB);
        if let Some(portrait) = adventurer_portrait_frame(persona) {
            blit(&portrait, PixelPoint::new(0, 0), &mut pixels);
        }
        flush_rgb(frame.buffer_mut(), portrait_area, &pixels, PARCHMENT_RGB);
    }

    let text_area = Rect::new(
        area.x + 28,
        area.y + 1,
        area.width.saturating_sub(29),
        area.height.saturating_sub(2),
    );
    frame.render_widget(
        Paragraph::new(text)
            .style(PARCHMENT)
            .wrap(Wrap { trim: false }),
        text_area,
    );
}

fn presence_label(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "Working",
        Presence::Blocked => "Needs counsel",
        Presence::Done => "Completed",
        Presence::Idle => "Resting",
        Presence::Exited => "Departed",
        Presence::Unknown => "Unknown",
    }
}

fn format_elapsed(elapsed: std::time::Duration) -> String {
    let seconds = elapsed.as_secs();
    if seconds < 60 {
        format!("{seconds}s")
    } else if seconds < 3_600 {
        format!("{}m", seconds / 60)
    } else {
        format!("{}h", seconds / 3_600)
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
