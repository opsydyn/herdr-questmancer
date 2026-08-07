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
    ledger,
    portrait::PortraitGallery,
    scene::{
        SceneFrame,
        assets::{adventurer::adventurer_portrait_frame, librarian},
        pixel::{PixelPoint, PixelRect, Rgb, RgbBuffer},
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
    let actor_areas = scene
        .actors
        .iter()
        .filter_map(|region| actor_terminal_area(region.bounds, area))
        .collect::<Vec<_>>();
    for region in &scene.actors {
        let Some(agent) = model.domain().agents.get(&region.agent) else {
            continue;
        };
        let elapsed = format_elapsed(agent.presence_since.elapsed_until(model.now()));
        let selected = model.selected_agent_key() == Some(&region.agent);
        let urgent = agent.presence == Presence::Blocked;
        // Budgets from the roomy form down to the bare state glyph. The first
        // that finds a free lane wins, so a crowded party degrades to short
        // nameplates instead of silently losing them: a compact Hall used to
        // show six adventurers and two labels.
        let budgets: &[usize] = if selected || urgent {
            &[36, 20, 12, 8, 4]
        } else {
            &[20, 12, 8, 4]
        };
        let elapsed = (selected || urgent).then_some(elapsed.as_str());

        let placed = budgets.iter().find_map(|budget| {
            let label = fit_identity_label(&agent.name, agent.presence, elapsed, *budget);
            let width = u16::try_from(label.chars().count())
                .unwrap_or(u16::MAX)
                .min(area.width);
            if width == 0 {
                return None;
            }
            let actor_centre = region.bounds.x + i32::from(region.bounds.width) / 2;
            let preferred_x = actor_centre - i32::from(width) / 2;
            let maximum_x = i32::from(area.right().saturating_sub(width));
            let x = preferred_x.clamp(i32::from(area.x), maximum_x.max(i32::from(area.x)));
            let actor_row = region.bounds.y.div_euclid(2);
            let below = (region.bounds.y + i32::from(region.bounds.height) + 1).div_euclid(2);
            [actor_row - 1, below].into_iter().find_map(|y| {
                let y = u16::try_from(y).ok()?;
                if y < area.y || y >= area.bottom().saturating_sub(1) {
                    return None;
                }
                let candidate = Rect::new(u16::try_from(x).ok()?, y, width, 1);
                (!occupied
                    .iter()
                    .any(|other| rects_intersect(with_gutter(candidate, area), *other))
                    && !actor_areas
                        .iter()
                        .any(|actor| rects_intersect(candidate, *actor)))
                .then_some((candidate, label.clone()))
            })
        });
        let Some((label_area, label)) = placed else {
            continue;
        };
        occupied.push(label_area);
        let style = if selected {
            PARCHMENT
        } else {
            PARCHMENT_BORDER
        };
        frame.render_widget(Paragraph::new(label).style(style), label_area);
    }
}

fn actor_terminal_area(bounds: PixelRect, frame_area: Rect) -> Option<Rect> {
    let x = u16::try_from(bounds.x).ok()?;
    let y = u16::try_from(bounds.y.div_euclid(2)).ok()?;
    let width = bounds.width.min(frame_area.width.saturating_sub(x));
    let height = bounds
        .height
        .div_ceil(2)
        .min(frame_area.height.saturating_sub(y));
    (width > 0 && height > 0 && x < frame_area.right() && y < frame_area.bottom())
        .then_some(Rect::new(x, y, width, height))
}

/// Widens a label by one column on each side for collision testing only.
///
/// `rects_intersect` is exclusive, so two nameplates whose edges merely meet
/// are not considered to collide. They were placed flush and read as a single
/// run of text — `codex · WORKING 2m` and `ember-car… · WORKING` became
/// `codex · WORKING 2member-car… · WORKING`.
fn with_gutter(label: Rect, area: Rect) -> Rect {
    let left = label.x.saturating_sub(1).max(area.x);
    let right = label.right().saturating_add(1).min(area.right());
    Rect::new(left, label.y, right.saturating_sub(left), label.height)
}

fn rects_intersect(left: Rect, right: Rect) -> bool {
    left.x < right.right()
        && left.right() > right.x
        && left.y < right.bottom()
        && left.bottom() > right.y
}

/// Fits an identity label into `maximum` columns, degrading name-first so the
/// presence state is never the part that gets truncated: full label, then a
/// shortened name with the whole badge, then the state glyph and age alone.
fn fit_identity_label(
    name: &str,
    presence: Presence,
    elapsed: Option<&str>,
    maximum: usize,
) -> String {
    let badge = presence_badge(presence);
    let suffix = elapsed.map_or_else(
        || format!(" · {badge}"),
        |elapsed| format!(" · {badge} {elapsed}"),
    );
    let suffix_length = suffix.chars().count();
    if name.chars().count() + suffix_length <= maximum {
        return format!("{name}{suffix}");
    }
    let name_budget = maximum.saturating_sub(suffix_length);
    if name_budget >= 4 {
        let kept = name.chars().take(name_budget - 1).collect::<String>();
        return format!("{kept}…{suffix}");
    }
    let glyph = presence_glyph(presence);
    let compact = elapsed.map_or_else(|| glyph.to_owned(), |elapsed| format!("{glyph} {elapsed}"));
    if compact.chars().count() <= maximum {
        compact
    } else {
        glyph.chars().take(maximum).collect()
    }
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

/// Single-character state marker for lanes too narrow to carry any name.
fn presence_glyph(presence: Presence) -> &'static str {
    match presence {
        Presence::Working => "»",
        Presence::Blocked => "!",
        Presence::Done => "✓",
        Presence::Idle => "z",
        Presence::Exited => "×",
        Presence::Unknown => "?",
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
        SceneOverlay::LibrarianLedger => render_librarian_ledger(frame, model, portraits),
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

fn render_librarian_ledger(
    frame: &mut Frame<'_>,
    model: &Model,
    portraits: Option<&PortraitGallery>,
) {
    let Some(page_id) = model.ledger_page() else {
        return;
    };
    let page = ledger::page(page_id);
    let available = frame.area();
    if available.width == 0 || available.height == 0 {
        return;
    }
    let wide = available.width >= 96 && available.height >= 22;
    let width = available
        .width
        .saturating_sub(2)
        .min(if wide { 88 } else { 64 });
    let height = available
        .height
        .saturating_sub(2)
        .min(if wide { 20 } else { 18 });
    let Some(area) = centered(available, width, height) else {
        return;
    };
    render_parchment(frame, area, " LIBRARIAN'S LEDGER ", Text::default());

    let page_number = page_id.index() + 1;
    let footer = format!(
        "Page {page_number} / {} · j/k or arrows turn · g/G ends",
        ledger::LedgerPageId::ALL.len()
    );
    let lines = std::iter::once(Line::from(page.title))
        .chain(std::iter::once(Line::from("")))
        .chain(page.body.iter().map(|line| Line::from(*line)))
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(footer)))
        .chain(std::iter::once(Line::from("Esc/? close")))
        .collect::<Vec<_>>();

    if wide {
        let portrait_area = Rect::new(area.x + 2, area.y + 2, 24, 16);
        render_librarian_illustration(frame, portrait_area, portraits);
        let text_area = Rect::new(
            area.x + 28,
            area.y + 2,
            area.width.saturating_sub(30),
            area.height.saturating_sub(4),
        );
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .style(PARCHMENT)
                .wrap(Wrap { trim: false }),
            text_area,
        );
    } else {
        let text_area = Rect::new(
            area.x + 2,
            area.y + 2,
            area.width.saturating_sub(4),
            area.height.saturating_sub(4),
        );
        frame.render_widget(
            Paragraph::new(Text::from(lines))
                .style(PARCHMENT)
                .wrap(Wrap { trim: false }),
            text_area,
        );
    }
}

fn render_librarian_illustration(
    frame: &mut Frame<'_>,
    area: Rect,
    portraits: Option<&PortraitGallery>,
) {
    const PARCHMENT_RGB: Rgb = Rgb::new(230, 207, 154);
    frame.render_widget(Block::default().style(PARCHMENT), area);
    if let Some(image) = portraits.and_then(PortraitGallery::librarian) {
        frame.render_widget(Image::new(image), area);
    } else {
        let mut pixels = RgbBuffer::filled(24, 32, PARCHMENT_RGB);
        blit(
            librarian::ledger_portrait(),
            PixelPoint::new(0, 0),
            &mut pixels,
        );
        flush_rgb(frame.buffer_mut(), area, &pixels, PARCHMENT_RGB);
    }
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
    // The urgency jump earns ribbon space only while somebody is actually
    // waiting, and it carries the count — so the ribbon answers "does anyone
    // need me?" before the key is ever pressed.
    let waiting = model.adventurers_awaiting_a_human().len();
    let urgent = if waiting == 0 {
        String::new()
    } else {
        format!("  [!] {waiting} waiting")
    };
    let text =
        format!("[1] Guild  [2] Delve  [j/k] Select  [Enter] Observe{counsel}{urgent}  [/] Search");
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn full_labels_pass_through_untouched() {
        assert_eq!(
            fit_identity_label("codex", Presence::Working, Some("7m"), 36),
            "codex · WORKING 7m"
        );
        assert_eq!(
            fit_identity_label("codex", Presence::Idle, None, 20),
            "codex · RESTING"
        );
    }

    #[test]
    fn truncation_shortens_the_name_and_never_the_state() {
        let label = fit_identity_label("archive-mender-of-the-vaults", Presence::Working, None, 20);
        assert_eq!(label, "archive-m… · WORKING");
        assert_eq!(label.chars().count(), 20);
    }

    #[test]
    fn urgent_labels_keep_badge_and_elapsed_over_the_name() {
        let label = fit_identity_label(
            "archive-mender-of-the-vaults",
            Presence::Blocked,
            Some("3m"),
            36,
        );
        assert!(label.ends_with(" · ! NEEDS COUNSEL 3m"), "{label}");
        assert!(label.chars().count() <= 36);
    }

    #[test]
    fn lanes_too_narrow_for_a_name_fall_back_to_the_state_glyph() {
        assert_eq!(
            fit_identity_label("codex", Presence::Blocked, Some("3m"), 8),
            "! 3m"
        );
        assert_eq!(fit_identity_label("codex", Presence::Working, None, 6), "»");
    }
}
