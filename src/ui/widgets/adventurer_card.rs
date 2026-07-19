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
    domain::{AdventurerClass, AdventuringGear, Agent, Ancestry, Keepsake},
    ui::{
        persona::compose_profile_adventurer_for_palette,
        pixel::{ColorRole, Palette, pack},
        theatre::{TheatreFrame, TheatrePose},
    },
};

use super::presentation::present;

#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub enum AdventurerCardPresentation {
    Hidden,
    Compact,
    Full,
}

#[must_use]
pub const fn adventurer_card_presentation(area: Rect) -> AdventurerCardPresentation {
    if area.width == 0 || area.height == 0 {
        AdventurerCardPresentation::Hidden
    } else if area.width >= 34 && area.height >= 19 {
        AdventurerCardPresentation::Full
    } else {
        AdventurerCardPresentation::Compact
    }
}

pub fn render_adventurer_card(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    preferences: &DisplayPreferences,
) {
    match adventurer_card_presentation(area) {
        AdventurerCardPresentation::Hidden => return,
        AdventurerCardPresentation::Compact => {
            render_compact(frame, area, agent, theatre, preferences.character_set);
            return;
        }
        AdventurerCardPresentation::Full => {}
    }

    let palette = Palette::from(preferences.color_mode);
    let border = Style::new().fg(if theatre.focused {
        palette.resolve(ColorRole::Selection)
    } else {
        palette.resolve(ColorRole::Stone)
    });
    let title = match (preferences.character_set, theatre.focused) {
        (CharacterSet::Ascii, true) => " * ADVENTURER PROFILE ",
        (CharacterSet::Ascii, false) => " ADVENTURER PROFILE ",
        (CharacterSet::Unicode, true) => " * PROFILE ",
        (CharacterSet::Unicode, false) => " PROFILE ",
    };
    let mut block = Block::default()
        .borders(Borders::ALL)
        .title(title)
        .border_style(border)
        .style(Style::new().bg(palette.resolve(ColorRole::DarkStone)));
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
            let canvas = compose_profile_adventurer_for_palette(&agent.persona, palette);
            frame.render_widget(
                Paragraph::new(pack(&canvas, &palette, ColorRole::DarkStone)),
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
                "{}",
                present(agent.persona.epithet.as_str(), preferences.character_set)
            ))
            .style(Style::new().fg(palette.resolve(ColorRole::RuneGlow))),
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
        Line::from(present(&agent.persona.name, character_set).into_owned()),
        Line::from(format!(
            "{} {}",
            ancestry_label(agent.persona.ancestry),
            class_label(agent.persona.class)
        )),
        Line::from(format!("Gear: {}", gear_label(agent.persona.class.gear()))),
        Line::from(format!("Agent: {}", present(&agent.name, character_set))),
        Line::from(format!(
            "Campaign: {}",
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
        "Keepsake: {}",
        keepsake_label(agent.persona.appearance.keepsake)
    )));
    if let Some(status) = agent.custom_status.as_deref() {
        details.push(Line::from(format!(
            "Status: {}",
            present(status, character_set)
        )));
    }
    frame.render_widget(
        Paragraph::new(Text::from(details))
            .style(Style::new().fg(palette.resolve(ColorRole::Parchment))),
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
        Line::from(present(&agent.persona.name, character_set).into_owned()),
        Line::from(format!(
            "{} {}{live}",
            state_marker(theatre.pose),
            theatre.label
        )),
        Line::from(format!(
            "Campaign: {}",
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
            "Keepsake: {}",
            keepsake_label(agent.persona.appearance.keepsake)
        )));
    }
    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn ascii_profile() -> Vec<Line<'static>> {
    [
        " ADVENTURER     ",
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
        TheatrePose::Delving => "[>]",
        TheatrePose::SeekingCounsel => "[!]",
        TheatrePose::SpoilsUnopened | TheatrePose::VictoryRecorded => "[+]",
        TheatrePose::Resting => "[~]",
        TheatrePose::Departed => "[x]",
        TheatrePose::Unknown => "[?]",
    }
}

const fn ancestry_label(ancestry: Ancestry) -> &'static str {
    match ancestry {
        Ancestry::Human => "Human",
        Ancestry::Dwarf => "Dwarf",
        Ancestry::Elf => "Elf",
        Ancestry::Halfling => "Halfling",
        Ancestry::Orc => "Orc",
        Ancestry::Gnome => "Gnome",
        Ancestry::Goblin => "Goblin",
    }
}

const fn class_label(class: AdventurerClass) -> &'static str {
    match class {
        AdventurerClass::Barbarian => "Barbarian",
        AdventurerClass::Bard => "Bard",
        AdventurerClass::Cleric => "Cleric",
        AdventurerClass::Druid => "Druid",
        AdventurerClass::Paladin => "Paladin",
        AdventurerClass::Ranger => "Ranger",
        AdventurerClass::Rogue => "Rogue",
        AdventurerClass::Wizard => "Wizard",
        AdventurerClass::Artificer => "Artificer",
        AdventurerClass::Runewright => "Runewright",
        AdventurerClass::Testmender => "Testmender",
        AdventurerClass::Pathseeker => "Pathseeker",
    }
}

const fn gear_label(gear: AdventuringGear) -> &'static str {
    match gear {
        AdventuringGear::Axe => "Axe",
        AdventuringGear::BowAndQuiver => "Bow and quiver",
        AdventuringGear::HolySymbol => "Holy symbol",
        AdventuringGear::LivingStaff => "Living staff",
        AdventuringGear::Lute => "Lute",
        AdventuringGear::MapAndCompass => "Map and compass",
        AdventuringGear::RuneChisel => "Rune chisel",
        AdventuringGear::Shield => "Shield",
        AdventuringGear::SpellbookAndStaff => "Spellbook and staff",
        AdventuringGear::TestKit => "Test kit",
        AdventuringGear::ThievesTools => "Thieves' tools",
        AdventuringGear::Toolkit => "Toolkit",
    }
}

const fn keepsake_label(keepsake: Keepsake) -> &'static str {
    match keepsake {
        Keepsake::Feather => "Feather",
        Keepsake::LuckyCoin => "Lucky coin",
        Keepsake::Mug => "Mug",
        Keepsake::PressedLeaf => "Pressed leaf",
        Keepsake::Ribbon => "Ribbon",
        Keepsake::TinyFamiliar => "Familiar",
    }
}
