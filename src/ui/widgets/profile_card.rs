use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    symbols::border,
    text::{Line, Text},
    widgets::{Block, BorderType, Borders, Paragraph},
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

use crate::{
    app::{CharacterSet, DisplayPreferences},
    domain::{Accessory, Agent, DeskProp},
    ui::{
        persona::compose_profile_for_palette,
        pixel::{ColorRole, Palette, pack},
        theatre::{TheatreFrame, TheatrePose},
    },
};

use super::presentation::present;

pub fn render_profile_card(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    preferences: &DisplayPreferences,
) {
    if area.is_empty() {
        return;
    }
    if area.width < 34 || area.height < 19 {
        render_compact(frame, area, agent, theatre, preferences.character_set);
        return;
    }

    let palette = Palette::from(preferences.color_mode);
    let border = Style::new().fg(if theatre.focused {
        palette.resolve(ColorRole::CrtGlow)
    } else {
        palette.resolve(ColorRole::CrtCase)
    });
    let title = match (preferences.character_set, theatre.focused) {
        (CharacterSet::Ascii, true) => " * AGENT PROFILE ",
        (CharacterSet::Ascii, false) => " AGENT PROFILE ",
        (CharacterSet::Unicode, true) => " * PROFILE ",
        (CharacterSet::Unicode, false) => " PROFILE ",
    };
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border)
        .style(Style::new().bg(palette.resolve(ColorRole::PanelBackground)));
    block = match preferences.character_set {
        CharacterSet::Unicode => block.border_type(BorderType::Rounded),
        CharacterSet::Ascii => block.border_set(ASCII_BORDER),
    };
    let inner = block.inner(area);
    frame.render_widget(block, area);
    if inner.is_empty() {
        return;
    }

    match preferences.character_set {
        CharacterSet::Unicode => {
            let canvas = compose_profile_for_palette(&agent.persona.appearance, palette);
            frame.render_widget(
                Paragraph::new(pack(&canvas, &palette, ColorRole::PanelBackground)),
                Rect::new(inner.x, inner.y, inner.width.min(16), inner.height.min(16)),
            );
        }
        CharacterSet::Ascii => frame.render_widget(
            Paragraph::new(Text::from(ascii_profile())),
            Rect::new(inner.x, inner.y, inner.width.min(16), inner.height.min(16)),
        ),
    }

    let details_x = inner
        .x
        .saturating_add(inner.width.min(16).saturating_add(1));
    let details_width = inner.width.saturating_sub(17);
    render_details(
        frame,
        Rect::new(details_x, inner.y, details_width, inner.height),
        agent,
        theatre,
        preferences.character_set,
        palette,
    );

    if inner.height > 16 {
        frame.render_widget(
            Paragraph::new(format!(
                "@{}",
                present(&agent.persona.handle, preferences.character_set)
            ))
            .style(Style::new().fg(palette.resolve(ColorRole::CrtGlow))),
            Rect::new(
                inner.x,
                inner.y.saturating_add(inner.height - 1),
                inner.width,
                1,
            ),
        );
    }
}

fn render_details(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    character_set: CharacterSet,
    palette: Palette,
) {
    if area.is_empty() {
        return;
    }
    let mut details = vec![
        Line::from(present(&agent.name, character_set).into_owned()),
        Line::from(format!(
            "Site: {}",
            present(agent.workspace_id.as_str(), character_set)
        )),
        Line::from(format!(
            "Pane: {}",
            present(agent.pane_id.as_str(), character_set)
        )),
        Line::from(format!("{} {}", state_marker(theatre.pose), theatre.label)),
    ];
    if theatre.focused {
        details.push(Line::from("(*) LIVE"));
    }
    details.push(Line::from(format!(
        "Accessory: {}",
        accessory_label(agent.persona.appearance.accessory)
    )));
    details.push(Line::from(format!(
        "Desk prop: {}",
        desk_prop_label(agent.persona.appearance.desk_prop)
    )));
    if let Some(status) = agent.custom_status.as_deref() {
        details.push(Line::from(format!(
            "Status: {}",
            present(status, character_set)
        )));
    }
    frame.render_widget(
        Paragraph::new(Text::from(details))
            .style(Style::new().fg(palette.resolve(ColorRole::Highlight))),
        area,
    );
}

fn render_compact(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    character_set: CharacterSet,
) {
    let live = if theatre.focused { " LIVE" } else { "" };
    let mut lines = vec![
        Line::from(present(&agent.name, character_set).into_owned()),
        Line::from(format!(
            "@{}",
            present(&agent.persona.handle, character_set)
        )),
        Line::from(format!(
            "{} {}{live}",
            state_marker(theatre.pose),
            theatre.label
        )),
        Line::from(format!(
            "Site: {}",
            present(agent.workspace_id.as_str(), character_set)
        )),
        Line::from(format!(
            "Pane: {}",
            present(agent.pane_id.as_str(), character_set)
        )),
    ];
    if let Some(status) = agent.custom_status.as_deref() {
        lines.push(Line::from(format!(
            "Status: {}",
            present(status, character_set)
        )));
    } else {
        lines.push(Line::from(format!(
            "Accessory: {}",
            accessory_label(agent.persona.appearance.accessory)
        )));
    }
    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn ascii_profile() -> Vec<Line<'static>> {
    [
        "  AGENT PROFILE ",
        "     .----.     ",
        "    / o  o \\    ",
        "    |  --  |    ",
        "     \\____/     ",
        "      /||\\      ",
        "     / || \\     ",
        "    |  ||  |    ",
        "    | [==] |    ",
        "       ||       ",
        "      /  \\      ",
        "      |  |      ",
        "      |  |      ",
        "      |  |      ",
        "     /_  _\\     ",
        "    FULL FIGURE ",
    ]
    .into_iter()
    .map(Line::from)
    .collect()
}

const fn state_marker(pose: TheatrePose) -> &'static str {
    match pose {
        TheatrePose::Working => "[>]",
        TheatrePose::Blocked => "[!]",
        TheatrePose::DoneUnseen | TheatrePose::DoneSeen => "[+]",
        TheatrePose::Idle => "[~]",
        TheatrePose::Exited => "[x]",
        TheatrePose::Unknown => "[?]",
    }
}

const fn accessory_label(accessory: Accessory) -> &'static str {
    match accessory {
        Accessory::Headphones => "Headphones",
        Accessory::Pager => "Pager",
        Accessory::Lanyard => "Lanyard",
        Accessory::Wristband => "Wristband",
        Accessory::Scarf => "Scarf",
        Accessory::Badge => "Badge",
        Accessory::PocketPen => "Pocket pen",
        Accessory::ShoulderBag => "Shoulder bag",
    }
}

const fn desk_prop_label(prop: DeskProp) -> &'static str {
    match prop {
        DeskProp::NoveltyMug => "Novelty mug",
        DeskProp::FloppyStack => "Floppy stack",
        DeskProp::DeskFan => "Desk fan",
        DeskProp::PizzaBox => "Pizza box",
        DeskProp::Joystick => "Joystick",
        DeskProp::Phone => "Phone",
        DeskProp::Manual => "Manual",
        DeskProp::TinyCactus => "Tiny cactus",
    }
}
