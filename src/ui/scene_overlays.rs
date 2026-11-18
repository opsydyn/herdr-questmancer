use ratatui::{
    Frame,
    layout::{Alignment, Rect},
    style::Color,
    text::{Line, Span, Text},
    widgets::{Block, Borders, Clear, Paragraph, Wrap},
};
use ratatui_image::Image;

use crate::{
    app::{CharacterSet, ColorMode, CounselPhase, Modal, Model, Notice},
    domain::{AdventurerClass, Presence},
    ledger,
    portrait::PortraitGallery,
    scene::{
        SceneFrame,
        assets::{adventurer::adventurer_portrait_frame, keepsakes, librarian, roster},
        heraldry::{CampaignCrest, CrestField},
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
        .flat_map(|region| {
            let is_roster =
                region.bounds.width == roster::WIDTH && region.bounds.height == roster::HEIGHT;
            let marker =
                is_roster.then(|| crate::scene::render::roster::state_marker_bounds(region.bounds));
            let mut body = region.bounds;
            if is_roster {
                body.height += 3; // Preserve the selection ring below the feet.
            }
            [Some(body), marker]
                .into_iter()
                .flatten()
                .filter_map(|bounds| actor_terminal_area(bounds, area))
        })
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
            let label = fit_identity_label(
                &agent.name,
                agent.presence,
                elapsed,
                *budget,
                model.preferences().character_set,
            );
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
            let foot_clearance =
                if region.bounds.width == roster::WIDTH && region.bounds.height == roster::HEIGHT {
                    4
                } else {
                    1
                };
            let below =
                (region.bounds.y + i32::from(region.bounds.height) + foot_clearance).div_euclid(2);
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
    let bottom = u16::try_from((bounds.y + i32::from(bounds.height) + 1).div_euclid(2)).ok()?;
    let height = bottom
        .saturating_sub(y)
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
    characters: CharacterSet,
) -> String {
    let badge = presence_badge(presence, characters);
    let separator = if characters == CharacterSet::Ascii {
        "|"
    } else {
        "·"
    };
    let suffix = elapsed.map_or_else(
        || format!(" {separator} {badge}"),
        |elapsed| format!(" {separator} {badge} {elapsed}"),
    );
    let suffix_length = suffix.chars().count();
    if name.chars().count() + suffix_length <= maximum {
        return format!("{name}{suffix}");
    }
    let name_budget = maximum.saturating_sub(suffix_length);
    if name_budget >= 4 {
        let kept = name.chars().take(name_budget - 1).collect::<String>();
        let ellipsis = if characters == CharacterSet::Ascii {
            '~'
        } else {
            '…'
        };
        return format!("{kept}{ellipsis}{suffix}");
    }
    let glyph = presence_glyph(presence, characters);
    let compact = elapsed.map_or_else(|| glyph.to_owned(), |elapsed| format!("{glyph} {elapsed}"));
    if compact.chars().count() <= maximum {
        compact
    } else {
        glyph.chars().take(maximum).collect()
    }
}

fn presence_badge(presence: Presence, characters: CharacterSet) -> &'static str {
    match presence {
        Presence::Working => "WORKING",
        Presence::Blocked => "! NEEDS COUNSEL",
        Presence::Done if characters == CharacterSet::Ascii => "+ COMPLETED",
        Presence::Done => "✓ COMPLETED",
        Presence::Idle => "RESTING",
        Presence::Exited if characters == CharacterSet::Ascii => "x DEPARTED",
        Presence::Exited => "× DEPARTED",
        Presence::Unknown => "? UNKNOWN",
    }
}

/// Single-character state marker for lanes too narrow to carry any name.
fn presence_glyph(presence: Presence, characters: CharacterSet) -> &'static str {
    match presence {
        Presence::Working if characters == CharacterSet::Ascii => ">",
        Presence::Working => "»",
        Presence::Blocked => "!",
        Presence::Done if characters == CharacterSet::Ascii => "+",
        Presence::Done => "✓",
        Presence::Idle => "z",
        Presence::Exited if characters == CharacterSet::Ascii => "x",
        Presence::Exited => "×",
        Presence::Unknown => "?",
    }
}

pub(crate) fn render_party_status(frame: &mut Frame<'_>, model: &Model) {
    let area = frame.area();
    if area.height < 2 || area.width < 20 {
        return;
    }
    let waiting = model
        .domain()
        .agents
        .values()
        .filter(|agent| {
            agent.attention.summons() == Some(crate::domain::GuildSummons::CounselRequested)
        })
        .count();
    let party = model
        .domain()
        .agents
        .values()
        .filter(|agent| agent.presence != Presence::Exited)
        .count();
    let wide = area.width >= 72;
    let counts = if wide {
        format!("{party} adventurers / {waiting} need counsel")
    } else if waiting > 0 && area.width >= 38 {
        format!("[!] {waiting} need counsel")
    } else if waiting > 0 {
        format!("[!] {waiting}")
    } else {
        String::new()
    };
    let characters = model.preferences().character_set;
    let identity = model.selected_agent().map_or_else(
        || "No adventurer selected".to_owned(),
        |agent| {
            let primary = match (agent.presence, wide) {
                (Presence::Exited, _) => "",
                (Presence::Blocked, true) => " [r] Counsel",
                (Presence::Blocked, false) => " [r]",
                (_, true) => " [Enter] Observe",
                (_, false) => " [Enter]",
            };
            let menu = if wide { " [?] Menu" } else { "" };
            let reserve =
                counts.len() + usize::from(!counts.is_empty()) * 3 + primary.len() + menu.len();
            let label = fit_identity_label(
                &agent.name,
                agent.presence,
                None,
                usize::from(area.width).saturating_sub(reserve),
                characters,
            );
            format!("{label}{primary}{menu}")
        },
    );
    let text = if counts.is_empty() {
        identity
    } else {
        format!("{identity} / {counts}")
    };
    let row = Rect::new(area.x, area.bottom() - 2, area.width, 1);
    frame.render_widget(Clear, row);
    frame.render_widget(Paragraph::new(text).style(PARCHMENT_BORDER), row);
}

pub fn render_scene_overlays(
    frame: &mut Frame<'_>,
    model: &Model,
    presentation: &ScenePresentation,
    portraits: Option<&PortraitGallery>,
) {
    match presentation.overlay {
        SceneOverlay::Counsel | SceneOverlay::Search | SceneOverlay::PartyName => {
            render_input_parchment(frame, model);
        }
        SceneOverlay::LibrarianLedger => render_librarian_ledger(frame, model, portraits),
        SceneOverlay::Scrying => render_scrying_parchment(frame, model),
        SceneOverlay::Chronicle => render_chronicle_parchment(frame, model),
        SceneOverlay::WarTable => crate::ui::war_table::render(frame, model, presentation),
        SceneOverlay::QuestBoard => crate::ui::work_board::render(frame, model),
        SceneOverlay::CharacterPicker => render_character_picker(frame, model),
        SceneOverlay::StatusExplanation => render_status_explanation(frame, model),
        SceneOverlay::None => {
            render_party_name(frame, model);
            render_standing_badge(frame, model);
            render_adventurer_card(frame, model, portraits);
            if model.command_ribbon_visible() {
                render_command_ribbon(frame, model);
            }
        }
    }
    // Last, so it wins the bottom row from the ribbon. Every other surface is
    // a place you went; this is the app answering for something you did.
    render_notice(frame, model);
}

fn render_party_name(frame: &mut Frame<'_>, model: &Model) {
    let Some(name) = model.durable_intent().party_name() else {
        return;
    };
    let area = frame.area();
    if area.width < 8 || area.height == 0 {
        return;
    }
    let badge = crate::rank::badge(model.experience(), model.preferences().character_set);
    let badge_width = u16::try_from(badge.chars().count())
        .unwrap_or(u16::MAX)
        .saturating_add(2);
    let name_area = Rect::new(
        area.x,
        area.y,
        area.width.saturating_sub(badge_width).max(1),
        1,
    );
    frame.render_widget(Paragraph::new(name).style(PARCHMENT_BORDER), name_area);
}

/// The one line that reports what just happened.
///
/// Questmancer set action feedback from thirty-two places and drew it from
/// none: `Model::status_message` had no caller in any renderer, so every
/// message — "Counsel issued.", the empty-draft refusal, every client failure
/// — was composed and discarded. Success and failure were equally silent, and
/// the tests passed because they asserted on the model rather than the frame.
fn render_notice(frame: &mut Frame<'_>, model: &Model) {
    let Some(message) = model.status_message() else {
        return;
    };
    let area = frame.area();
    if area.width < 8 || area.height == 0 {
        return;
    }
    let row = Rect::new(area.x, area.bottom().saturating_sub(1), area.width, 1);
    let width = usize::from(area.width);
    let seal = if matches!(model.notice(), Some(Notice::CounselIssued(_))) {
        match model.preferences().character_set {
            CharacterSet::Unicode => "◆ ",
            CharacterSet::Ascii => "(*) ",
        }
    } else {
        ""
    };
    let text_width = width.saturating_sub(seal.chars().count());
    let mut text: String = message.chars().take(text_width).collect();
    if text.chars().count() < text_width {
        // Painted over whatever held the row before, so a short notice cannot
        // leave the tail of a longer one behind it.
        text.push_str(&" ".repeat(text_width - text.chars().count()));
    }
    frame.render_widget(Clear, row);
    let line = Line::from(vec![
        Span::styled(seal, PARCHMENT.fg(Color::Red)),
        Span::raw(text),
    ]);
    frame.render_widget(Paragraph::new(line).style(PARCHMENT_BORDER), row);
}

/// The guild's standing, top right, always.
///
/// Permanent chrome over a scene that is meant to be the point, so it earns
/// one short line and no more. Returns the rows it occupied, because the
/// adventurer card lives in the same corner and has to start below it.
fn render_standing_badge(frame: &mut Frame<'_>, model: &Model) -> u16 {
    let area = frame.area();
    let badge = crate::rank::badge(model.experience(), model.preferences().character_set);
    let width = u16::try_from(badge.chars().count()).unwrap_or(u16::MAX);
    // A room too narrow for the badge keeps the room. The standing is a
    // flourish and loses every contest for space.
    if area.width < width + 2 || area.height < 4 {
        return 0;
    }
    let badge_area = Rect::new(area.right() - width - 1, area.y, width, 1);
    frame.render_widget(Clear, badge_area);
    frame.render_widget(Paragraph::new(badge).style(PARCHMENT_BORDER), badge_area);
    1
}

/// The shared presentation/input footprint of an actually visible profile.
pub fn adventurer_card_area(area: Rect, model: &Model) -> Option<Rect> {
    if model.modal() != &crate::app::Modal::None
        || !model.adventurer_card_visible()
        || model.selected_agent().is_none()
        || area.width < 60
        || area.height < 14
    {
        return None;
    }
    let detailed = area.width >= 96 && area.height >= 20;
    let width = area
        .width
        .saturating_sub(4)
        .min(if detailed { 78 } else { 48 });
    let height = (if detailed { 18 } else { 13 }).min(area.height.saturating_sub(2));
    Some(Rect::new(
        area.right() - width - 1,
        area.y + 2,
        width,
        height,
    ))
}

fn render_adventurer_card(
    frame: &mut Frame<'_>,
    model: &Model,
    portraits: Option<&PortraitGallery>,
) {
    let Some(card) = adventurer_card_area(frame.area(), model) else {
        return;
    };
    let Some(agent) = model.selected_agent() else {
        return;
    };
    let detailed = frame.area().width >= 96 && frame.area().height >= 20;
    let campaign = model
        .domain()
        .campaigns
        .get(&agent.workspace_id)
        .map_or(agent.workspace_id.as_str(), |campaign| {
            campaign.label.as_str()
        });
    let elapsed = format_elapsed(agent.presence_since.elapsed_until(model.now()));
    let status = presence_label(agent.presence);
    let role = agent.persona.role_label();
    let message = agent
        .custom_status
        .as_deref()
        .unwrap_or("No current field report.");
    let mut lines = vec![
        Line::from(agent.persona.name.clone()),
        Line::from(format!("{role} · {}", agent.persona.epithet.as_str())),
        campaign_crest_line(&agent.workspace_id, model),
        Line::from(format!("Agent: {}", agent.name)),
        Line::from(format!("Campaign: {campaign}")),
        Line::from(format!("{status} · {elapsed}")),
        Line::from(message.to_owned()),
    ];
    if detailed {
        lines.push(Line::from("[e] Character · [i] Why this status?"));
        render_portrait_card(
            frame,
            card,
            &agent.persona,
            Text::from(lines),
            model.preferences().color_mode,
            portraits,
        );
    } else {
        let keepsake = agent.persona.appearance.keepsake;
        lines.push(Line::from(format!(
            "Keepsake: {}",
            keepsakes::name(keepsake)
        )));
        lines.push(Line::from(keepsakes::description(keepsake)));
        lines.push(Line::from("Esc close · Enter observe · r counsel · o scry"));
        // Ten fixed rows fit even the minimum card. Truncate long live reports
        // horizontally so they cannot displace keepsake identity or controls.
        frame.render_widget(Clear, card);
        frame.render_widget(
            Paragraph::new(lines).style(PARCHMENT).block(
                Block::default()
                    .title(" ADVENTURER · [e] CHARACTER · [i] STATUS ")
                    .title_alignment(Alignment::Center)
                    .borders(Borders::ALL)
                    .border_style(PARCHMENT_BORDER),
            ),
            card,
        );
    }
}

fn campaign_crest_line(workspace: &crate::domain::WorkspaceId, model: &Model) -> Line<'static> {
    let crest = CampaignCrest::for_workspace(workspace);
    let rgb = crest.field.colour();
    let colour = match model.preferences().color_mode {
        ColorMode::Xterm256 => Color::Rgb(rgb.r, rgb.g, rgb.b),
        ColorMode::Ansi16 => match crest.field {
            CrestField::Azure => Color::Blue,
            CrestField::Moss => Color::Green,
            CrestField::Wine => Color::Magenta,
            CrestField::Umber => Color::Yellow,
        },
    };
    Line::from(vec![
        Span::raw("Crest: "),
        Span::styled(
            crest
                .charge
                .symbol(model.preferences().character_set == CharacterSet::Ascii),
            ratatui::style::Style::default().fg(colour),
        ),
        Span::raw(format!(" {} {}", crest.field.name(), crest.charge.name())),
    ])
}

fn render_portrait_card(
    frame: &mut Frame<'_>,
    area: Rect,
    persona: &crate::domain::AdventurerPersona,
    text: Text<'_>,
    colour_mode: ColorMode,
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
        render_sprite_portrait(frame, portrait_area, persona, colour_mode);
    }

    let text_area = Rect::new(area.x + 28, area.y + 1, area.width.saturating_sub(29), 10);
    // Keep each live fact on its own row: long labels must not displace status.
    frame.render_widget(Paragraph::new(text).style(PARCHMENT), text_area);
    let keepsake = persona.appearance.keepsake;
    let mut pixels = RgbBuffer::filled(8, 8, PARCHMENT_RGB);
    blit(
        &keepsakes::illustration(keepsake),
        PixelPoint::new(0, 0),
        &mut pixels,
    );
    flush_rgb(
        frame.buffer_mut(),
        Rect::new(text_area.x, area.y + 12, 8, 4),
        &pixels,
        PARCHMENT_RGB,
        colour_mode,
    );
    frame.render_widget(
        Paragraph::new(vec![
            Line::from(format!("Keepsake: {}", keepsakes::name(keepsake))),
            Line::from(keepsakes::description(keepsake)),
        ])
        .style(PARCHMENT)
        .wrap(Wrap { trim: false }),
        Rect::new(text_area.x + 10, area.y + 12, text_area.width - 10, 4),
    );
    frame.render_widget(
        Paragraph::new("Esc close · Enter observe · r counsel · o scry").style(PARCHMENT),
        Rect::new(text_area.x, area.y + 16, text_area.width, 1),
    );
}

fn render_sprite_portrait(
    frame: &mut Frame<'_>,
    area: Rect,
    persona: &crate::domain::AdventurerPersona,
    colour_mode: ColorMode,
) {
    const PAPER: Rgb = Rgb::new(230, 207, 154);
    let mut pixels = RgbBuffer::filled(24, 32, PAPER);
    if let Some(portrait) = adventurer_portrait_frame(persona) {
        blit(&portrait, PixelPoint::new(0, 0), &mut pixels);
    }
    flush_rgb(frame.buffer_mut(), area, &pixels, PAPER, colour_mode);
}

fn render_character_picker(frame: &mut Frame<'_>, model: &Model) {
    let Modal::CharacterPicker {
        adventurer,
        persona,
        choice,
        ..
    } = model.modal()
    else {
        return;
    };
    let Some(area) =
        centered(frame.area(), 70, 23).filter(|area| area.width >= 20 && area.height >= 8)
    else {
        frame.render_widget(Clear, frame.area());
        frame.render_widget(
            Paragraph::new("Widen pane\nEsc cancel").style(PARCHMENT),
            frame.area(),
        );
        return;
    };
    let Some(agent) = model.character_picker_target(adventurer, persona) else {
        render_parchment(
            frame,
            area,
            " CHARACTER ",
            Text::from("Adventurer unavailable.\nEsc close"),
        );
        return;
    };
    let classes = AdventurerClass::ALL;
    let automatic = agent.persona.automatic_class();
    let chosen = classes.get(*choice).copied().unwrap_or(automatic);
    let visual = area.width >= 57 && area.height >= 21;
    let width = if visual {
        area.width - 28
    } else {
        area.width - 2
    };
    let left = area.x + 1;
    let capacity = usize::from(area.height - 7);
    let start = choice
        .saturating_sub(capacity / 2)
        .min((classes.len() + 1).saturating_sub(capacity));
    let mut options = classes
        .iter()
        .map(|class| class.label())
        .chain(["Restore automatic"])
        .enumerate()
        .skip(start)
        .take(capacity)
        .map(|(index, label)| {
            let marker = if index == *choice { ">" } else { " " };
            Line::from(format!("{marker} {label}"))
        })
        .collect::<Vec<_>>();
    options.resize(capacity, Line::default());
    let apply_keys = if width < 23 {
        "Enter set Esc exit"
    } else {
        "Enter apply  Esc cancel"
    };
    let text = vec![
        Line::from(format!("{} · {}", agent.name, agent.persona.name)),
        Line::from(format!("Current · {}", agent.persona.class.label())),
        Line::from(format!("Preview · {}", chosen.label())),
    ]
    .into_iter()
    .chain(options)
    .chain([Line::from("j/k or arrows"), Line::from(apply_keys)])
    .collect::<Vec<_>>();
    frame.render_widget(Clear, area);
    frame.render_widget(
        Block::bordered()
            .title(" CHOOSE CHARACTER ")
            .title_alignment(Alignment::Center)
            .style(PARCHMENT)
            .border_style(PARCHMENT_BORDER),
        area,
    );
    frame.render_widget(
        Paragraph::new(text).style(PARCHMENT),
        Rect::new(left, area.y + 1, width, area.height - 2),
    );
    if visual {
        let mut preview = agent.persona.clone();
        preview.class = chosen;
        render_sprite_portrait(
            frame,
            Rect::new(area.right() - 26, area.y + 3, 24, 16),
            &preview,
            model.preferences().color_mode,
        );
    }
}

fn render_status_explanation(frame: &mut Frame<'_>, model: &Model) {
    let Modal::StatusExplanation(status) = model.modal() else {
        return;
    };
    let Some(area) = centered(frame.area(), 66, 11) else {
        return;
    };
    let mut lines = vec![Line::from("Esc close")];
    if let Some(agent) = model
        .selected_agent()
        .filter(|agent| status.request.matches(agent))
    {
        lines.push(Line::from(agent.name.as_str()));
        lines.push(Line::from(format!(
            "Live status · {}",
            presence_label(agent.presence)
        )));
        let explanation = match &status.outcome {
            None => "Asking Herdr…",
            Some(Err(error)) => error,
            Some(Ok(report)) if report.screen_detection_skipped => {
                "Herdr skipped screen detection for a reported state."
            }
            Some(Ok(report)) if report.matched_rule.is_some() => {
                "Herdr's screen check matched a rule."
            }
            Some(Ok(report)) if report.fallback_reason.is_some() => {
                "Herdr's screen check used a fallback."
            }
            Some(Ok(_)) => "Herdr supplied no detection reason.",
        };
        if let Some(Ok(report)) = &status.outcome {
            let estimate = Presence::from(report.state);
            if estimate != agent.presence {
                let label = if report.screen_detection_skipped {
                    "Herdr explanation"
                } else {
                    "Screen estimate"
                };
                lines.push(Line::from(format!(
                    "{label} · {}",
                    presence_label(estimate)
                )));
            }
        }
        lines.push(Line::from(explanation));
        let coverage = if agent.capture_identity.is_qualified() {
            "Chronicle: session identity available."
        } else {
            "Chronicle: observations withheld until session identity is unambiguous."
        };
        lines.push(Line::from(coverage));
    } else {
        lines.push(Line::from("Adventurer status changed. Reopen to refresh."));
    }
    render_parchment(frame, area, " WHY THIS STATUS? ", Text::from(lines));
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
        Modal::Counsel { draft, phase } => (
            " ISSUE COUNSEL ",
            draft.as_str(),
            match phase {
                CounselPhase::Drafting => "Enter send  Esc cancel",
                CounselPhase::Sending { .. } => "Sending counsel…",
                CounselPhase::TextFailed { .. } => "Enter retry  Esc cancel",
                CounselPhase::TextUncertain { .. } => "Enter observe  Esc abandon",
                CounselPhase::SubmissionFailed { .. } => "Enter retry submission  Esc close",
            },
        ),
        Modal::Search { query } => (
            " SEARCH THE GUILD ",
            query.as_str(),
            "Enter find  Esc cancel",
        ),
        Modal::PartyName { draft, .. } => (
            " NAME THE PARTY ",
            draft.as_str(),
            "Enter save  Esc cancel\nCtrl-U restore default",
        ),
        _ => return,
    };
    let Some(area) = centered(frame.area(), 64, 9) else {
        return;
    };
    // Without a caret the parchment reads as an empty panel rather than a
    // field waiting for you, and there is nothing to show a keystroke landed.
    let locked = matches!(
        model.modal(),
        Modal::Counsel {
            phase: CounselPhase::Sending { .. }
                | CounselPhase::TextUncertain { .. }
                | CounselPhase::SubmissionFailed { .. },
            ..
        }
    );
    let typed = if locked {
        input.to_owned()
    } else {
        format!("{input}_")
    };
    let mut lines = vec![Line::from(""), Line::from(typed), Line::from("")];
    if let Modal::Counsel {
        phase: CounselPhase::TextFailed { message },
        ..
    } = model.modal()
    {
        lines.push(Line::from(format!("Not issued: {message}")));
    }
    if let Modal::Counsel {
        phase: CounselPhase::SubmissionFailed { message, .. },
        ..
    } = model.modal()
    {
        lines.push(Line::from(format!("Written, not submitted: {message}")));
    }
    if let Modal::Counsel {
        phase: CounselPhase::TextUncertain { message, .. },
        ..
    } = model.modal()
    {
        lines.push(Line::from(format!("Delivery unconfirmed: {message}")));
    }
    lines.extend(keys.lines().map(Line::from));
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
    // The keyring's two columns need more width than a prose page; every
    // other page stays at its authored measure so paragraphs do not sprawl.
    let measure = if page_id == ledger::LedgerPageId::QuestmancersTools {
        112
    } else if wide {
        88
    } else {
        64
    };
    let width = available.width.saturating_sub(2).min(measure);
    // Sized to the page rather than to a fixed 20 rows. The generated keyring
    // is longer than any authored page, and a fixed height silently cut its
    // last two bindings and the footer off the bottom — the discoverability
    // page, hiding the least-known keys.
    let body = ledger_lines(page_id, wide, model.experience());
    let body_rows = u16::try_from(body.len()).unwrap_or(u16::MAX);
    // Title, blank, body, blank, footer, close, plus the frame's own padding.
    let needed = body_rows.saturating_add(9);
    let height = available
        .height
        .saturating_sub(2)
        .min(needed.max(if wide { 20 } else { 18 }));
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
        .chain(body.into_iter().map(Line::from).collect::<Vec<_>>())
        .chain(std::iter::once(Line::from("")))
        .chain(std::iter::once(Line::from(footer)))
        .chain(std::iter::once(Line::from("Esc/? close")))
        .collect::<Vec<_>>();

    // The keyring is a reference table, not prose. It takes the whole page:
    // the illustration costs it twenty-eight columns it needs for a second
    // column, and without one the list is taller than most terminals.
    if wide && page_id != ledger::LedgerPageId::QuestmancersTools {
        let portrait_area = Rect::new(area.x + 2, area.y + 2, 24, 16);
        render_librarian_illustration(
            frame,
            portrait_area,
            portraits,
            model.preferences().color_mode,
        );
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

/// The keyring pairs into two columns when the Ledger is wide enough, because
/// it has outgrown a single column. Every other page keeps its authored prose.
fn ledger_lines(page_id: ledger::LedgerPageId, wide: bool, experience: u64) -> Vec<String> {
    match page_id {
        ledger::LedgerPageId::QuestmancersTools if wide => super::keymap::paired_lines(),
        ledger::LedgerPageId::GuildStanding => ledger::standing_page_body(experience),
        other => ledger::page_body(other),
    }
}

fn render_librarian_illustration(
    frame: &mut Frame<'_>,
    area: Rect,
    portraits: Option<&PortraitGallery>,
    colour_mode: ColorMode,
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
        flush_rgb(
            frame.buffer_mut(),
            area,
            &pixels,
            PARCHMENT_RGB,
            colour_mode,
        );
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
    // The fetched output is longer than the parchment by design, so the
    // offset has to reach the widget or the scroll keys change a number
    // nothing draws.
    let offset = model.reading_scroll();
    // Borders, the blank line and the footer. Take only what fits, or the
    // footer telling you how to scroll is itself scrolled off the bottom.
    let capacity = usize::from(area.height.saturating_sub(4)).max(1);
    let visible = body
        .lines()
        .skip(usize::from(offset))
        .take(capacity)
        .map(|line| Line::from(line.to_owned()))
        .collect::<Vec<_>>();
    let scrollable = body.lines().count() > capacity;
    let footer = if scrollable {
        "Esc close · o refresh · j/k or wheel scroll"
    } else {
        "Esc close · o refresh"
    };
    render_parchment(
        frame,
        area,
        " SCRYING ",
        Text::from(
            visible
                .into_iter()
                .chain([Line::from(""), Line::from(footer)])
                .collect::<Vec<_>>(),
        ),
    );
}

/// The guild's own record of what happened.
///
/// Seven event types were recorded, persisted to `chronicle.jsonl` and
/// replayed on startup, and only one of them — returned spoils — reached a
/// human, as a count in a sidebar token. The other six were written and never
/// read by anything.
fn render_chronicle_parchment(frame: &mut Frame<'_>, model: &Model) {
    if let Some(lines) = model.chronicle_chapter_lines() {
        render_chronicle_chapter(frame, model, lines);
        return;
    }
    let available = frame.area();
    let width = available.width.saturating_sub(4).min(76);
    // Borders take two rows, the blank line and the footer two more. Ask for
    // only as many entries as fit, then shrink the parchment to what it holds
    // — a fixed-height box around three entries is mostly empty paper.
    let capacity = usize::from(available.height.saturating_sub(6)).clamp(1, 14);
    let offset = usize::from(model.reading_scroll());
    let all = model.chronicle_entries(usize::MAX);
    let scrollable = all.len() > capacity;
    let entries = all
        .into_iter()
        .skip(offset)
        .take(capacity)
        .collect::<Vec<_>>();
    let rows = u16::try_from(entries.len().max(1)).unwrap_or(1);
    let height = rows
        .saturating_add(4)
        .min(available.height.saturating_sub(2));
    let Some(area) = centered(available, width, height) else {
        return;
    };

    let (title, empty) = model.selected_agent().map_or(
        (
            " THE GUILD CHRONICLE ",
            "The Chronicle records nothing yet.",
        ),
        |_| {
            (
                " CHRONICLE OF THIS ADVENTURER ",
                "This adventurer has no Chronicle yet.",
            )
        },
    );

    let mut lines = Vec::new();
    if entries.is_empty() {
        lines.push(Line::from(empty));
    } else {
        for entry in entries {
            let ago = format_elapsed(entry.occurred_at.elapsed_until(model.now()));
            let sigil = entry.event().sigil();
            let summary = if entry.summary.is_empty() {
                entry.event().label().to_owned()
            } else {
                entry.summary.clone()
            };
            let time_label = if entry.observation().is_some() {
                format!("observed {ago} ago")
            } else {
                format!("{ago:>4} ago")
            };
            lines.push(Line::from(format!("{sigil} {time_label}  {summary}")));
        }
    }
    lines.push(Line::from(""));
    lines.push(Line::from(if scrollable {
        "Esc close · Tab guild chapter · j/k or wheel scroll"
    } else {
        "Esc close · Tab guild chapter"
    }));
    render_parchment(frame, area, title, Text::from(lines));
}

fn render_chronicle_chapter(frame: &mut Frame<'_>, model: &Model, lines: Vec<String>) {
    let available = frame.area();
    let capacity = usize::from(available.height.saturating_sub(6)).clamp(1, 14);
    let offset = usize::from(model.reading_scroll()).min(lines.len().saturating_sub(1));
    let visible = lines
        .into_iter()
        .skip(offset)
        .take(capacity)
        .map(Line::from)
        .collect::<Vec<_>>();
    let height = u16::try_from(visible.len()).unwrap_or(14).saturating_add(4);
    let Some(area) = centered(available, crate::chronicle_chapter::CHAPTER_COLUMNS, height) else {
        return;
    };
    let mut text = visible;
    text.push(Line::from(""));
    text.push(Line::from("Esc close · Tab records · j/k or wheel scroll"));
    frame.render_widget(Clear, area);
    frame.render_widget(
        Paragraph::new(text).style(PARCHMENT).block(
            Block::default()
                .title(" CHRONICLE / LAST HOUR ")
                .title_alignment(Alignment::Center)
                .borders(Borders::ALL)
                .border_style(PARCHMENT_BORDER),
        ),
        area,
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
    let character = model.selected_agent().map_or("", |_| "  [e] Character");
    // The urgency jump earns ribbon space only while somebody is actually
    // waiting, and it carries the count — so the ribbon answers "does anyone
    // need me?" before the key is ever pressed.
    let waiting = crate::work::queue(model).len();
    let urgent = if waiting == 0 {
        String::new()
    } else {
        format!("  [!] {waiting} waiting")
    };
    // `?` earns its place: it is the one binding that leads to all the others,
    // and the ribbon is most often read by somebody who knows none of them.
    let text = format!(
        "[1] Guild  [2] Delve  [?] Keys  [j/k] Select{character}  [Enter] Observe{counsel}{urgent}  [b] Board  [c] Chronicle  [/] Search"
    );
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
            fit_identity_label(
                "codex",
                Presence::Working,
                Some("7m"),
                36,
                CharacterSet::Unicode
            ),
            "codex · WORKING 7m"
        );
        assert_eq!(
            fit_identity_label("codex", Presence::Idle, None, 20, CharacterSet::Unicode),
            "codex · RESTING"
        );
    }

    #[test]
    fn truncation_shortens_the_name_and_never_the_state() {
        let label = fit_identity_label(
            "archive-mender-of-the-vaults",
            Presence::Working,
            None,
            20,
            CharacterSet::Unicode,
        );
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
            CharacterSet::Unicode,
        );
        assert!(label.ends_with(" · ! NEEDS COUNSEL 3m"), "{label}");
        assert!(label.chars().count() <= 36);
    }

    #[test]
    fn lanes_too_narrow_for_a_name_fall_back_to_the_state_glyph() {
        assert_eq!(
            fit_identity_label(
                "codex",
                Presence::Blocked,
                Some("3m"),
                8,
                CharacterSet::Unicode
            ),
            "! 3m"
        );
        assert_eq!(
            fit_identity_label("codex", Presence::Working, None, 6, CharacterSet::Unicode),
            "»"
        );
    }
}
