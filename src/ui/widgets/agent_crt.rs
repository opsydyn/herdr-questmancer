use ratatui::{
    Frame,
    layout::Rect,
    style::Style,
    text::{Line, Text},
    widgets::Paragraph,
};

use crate::{
    app::{CharacterSet, ColorMode, DisplayPreferences},
    domain::{Agent, PersonaAppearance},
    ui::{
        cafe_scene::SeatAnchor,
        persona::compose_seated_with_gear_for_palette,
        pixel::{Canvas, ColorRole, Palette, pack},
        theatre::{TheatreFrame, TheatrePose},
    },
};

use super::presentation::present;

const MIN_FULL_WIDTH: u16 = 28;
const MIN_FULL_HEIGHT: u16 = 10;
pub trait WorkstationAnchor {
    fn rect(self) -> Rect;
}

impl WorkstationAnchor for Rect {
    fn rect(self) -> Rect {
        self
    }
}

impl WorkstationAnchor for SeatAnchor {
    fn rect(self) -> Rect {
        Rect::new(self.x, self.y, self.width, self.height)
    }
}

pub fn render_workstation<A: WorkstationAnchor>(
    frame: &mut Frame<'_>,
    anchor: A,
    agent: &Agent,
    theatre: TheatreFrame,
    selected: bool,
    preferences: &DisplayPreferences,
) {
    let area = anchor.rect();
    if area.is_empty() {
        return;
    }
    if area.width < MIN_FULL_WIDTH || area.height < MIN_FULL_HEIGHT {
        render_compact(
            frame,
            area,
            agent,
            theatre,
            selected,
            preferences.character_set,
            preferences.color_mode,
        );
        return;
    }

    let palette = Palette::from(preferences.color_mode);
    let panel = Style::new()
        .fg(palette.resolve(ColorRole::Highlight))
        .bg(palette.resolve(ColorRole::PanelBackground));
    let inner = area;
    frame.render_widget(Paragraph::new("").style(panel), area);
    if inner.is_empty() {
        return;
    }

    let selection = if selected { ">" } else { " " };
    let name = format!(
        "{selection} {}",
        present(&agent.name, preferences.character_set)
    );
    frame.render_widget(
        Paragraph::new(name).style(Style::new().fg(palette.resolve(ColorRole::Highlight))),
        Rect::new(inner.x, inner.y, inner.width, 1),
    );

    let scene = Rect::new(
        inner.x,
        inner.y.saturating_add(1),
        inner.width,
        inner.height.saturating_sub(2).min(6),
    );
    render_scene(
        frame,
        scene,
        agent,
        theatre,
        selected,
        preferences.character_set,
        palette,
    );

    let state_y = inner.y.saturating_add(inner.height.saturating_sub(1));
    let state = state_line(agent, theatre, inner.width, preferences.character_set);
    frame.render_widget(
        Paragraph::new(state).style(Style::new().fg(palette.resolve(ColorRole::CrtGlow))),
        Rect::new(inner.x, state_y, inner.width, 1),
    );
}

fn render_compact(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    selected: bool,
    character_set: CharacterSet,
    color_mode: ColorMode,
) {
    if area.width >= 14 && area.height >= 6 {
        render_compact_scene(
            frame,
            area,
            agent,
            theatre,
            selected,
            character_set,
            color_mode,
        );
        return;
    }

    let selection = if selected { ">" } else { " " };
    let live = if theatre.focused { " LIVE" } else { "" };
    let mut lines = vec![
        Line::from(format!(
            "{selection} {}",
            present(&agent.name, character_set)
        )),
        Line::from(format!(
            "{} {}{live}",
            state_marker(theatre.pose),
            theatre.label
        )),
    ];
    if let Some(status) = agent.custom_status.as_deref() {
        lines.push(Line::from(format!(
            "Status: {}",
            present(status, character_set)
        )));
    }
    frame.render_widget(Paragraph::new(Text::from(lines)), area);
}

fn render_compact_scene(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    selected: bool,
    character_set: CharacterSet,
    color_mode: ColorMode,
) {
    let palette = Palette::from(color_mode);
    frame.render_widget(
        Paragraph::new("").style(
            Style::new()
                .fg(palette.resolve(ColorRole::Highlight))
                .bg(palette.resolve(ColorRole::PanelBackground)),
        ),
        area,
    );

    let scene_height = area.height.saturating_sub(1);
    if scene_height > 0 {
        render_scene(
            frame,
            Rect::new(area.x, area.y, area.width, scene_height),
            agent,
            theatre,
            selected,
            character_set,
            palette,
        );
    }

    let selection = if selected { ">" } else { " " };
    frame.render_widget(
        Paragraph::new(format!(
            "{selection} {}",
            present(&agent.name, character_set)
        ))
        .style(Style::new().fg(palette.resolve(ColorRole::Highlight))),
        Rect::new(area.x, area.y, area.width, 1),
    );

    let state_y = area.y.saturating_add(area.height.saturating_sub(1));
    frame.render_widget(
        Paragraph::new(state_line(agent, theatre, area.width, character_set))
            .style(Style::new().fg(palette.resolve(ColorRole::CrtGlow))),
        Rect::new(area.x, state_y, area.width, 1),
    );
}

fn render_scene(
    frame: &mut Frame<'_>,
    area: Rect,
    agent: &Agent,
    theatre: TheatreFrame,
    selected: bool,
    character_set: CharacterSet,
    palette: Palette,
) {
    if area.is_empty() {
        return;
    }
    let background = Style::new()
        .fg(palette.resolve(ColorRole::Desk))
        .bg(palette.resolve(ColorRole::PanelBackground));
    frame.render_widget(
        Paragraph::new(Text::from(scene_lines(theatre, selected))).style(background),
        area,
    );

    let persona_area = Rect::new(
        area.x.saturating_add(area.width.saturating_sub(10)),
        area.y,
        area.width.min(10),
        area.height.min(6),
    );
    match character_set {
        CharacterSet::Unicode => {
            let canvas = compose_station_figure(
                &agent.persona.appearance,
                agent.persona.class.gear(),
                theatre,
                palette,
            );
            frame.render_widget(
                Paragraph::new(pack(&canvas, &palette, ColorRole::PanelBackground)),
                persona_area,
            );
        }
        CharacterSet::Ascii => frame.render_widget(
            Paragraph::new(Text::from(ascii_pose(theatre.pose))),
            persona_area,
        ),
    }
}

fn compose_station_figure(
    appearance: &PersonaAppearance,
    gear: crate::domain::AdventuringGear,
    theatre: TheatreFrame,
    palette: Palette,
) -> Canvas {
    let mut station = chair_for_pose(theatre.pose);
    let persona = compose_seated_with_gear_for_palette(appearance, gear, theatre, palette);
    overlay(&mut station, &persona);
    station
}

fn chair_for_pose(pose: TheatrePose) -> Canvas {
    let mut chair = Canvas::new(10, 12);
    match pose {
        TheatrePose::DoneUnseen | TheatrePose::DoneSeen | TheatrePose::Idle => {
            chair.fill_rect(1, 5, 2, 5, ColorRole::Chair);
            chair.fill_rect(2, 9, 7, 2, ColorRole::Chair);
            chair.set(1, 10, ColorRole::Chair);
            chair.set(8, 11, ColorRole::Chair);
        }
        TheatrePose::Working
        | TheatrePose::Blocked
        | TheatrePose::Exited
        | TheatrePose::Unknown => {
            chair.fill_rect(0, 4, 2, 6, ColorRole::Chair);
            chair.fill_rect(1, 8, 7, 2, ColorRole::Chair);
            chair.fill_rect(2, 10, 1, 2, ColorRole::Chair);
            chair.fill_rect(7, 10, 1, 2, ColorRole::Chair);
        }
    }
    chair
}

fn overlay(target: &mut Canvas, source: &Canvas) {
    let width = usize::from(source.width());
    for y in 0..source.height() {
        for x in 0..source.width() {
            let index = usize::from(y) * width + usize::from(x);
            if let Some(role) = source.pixels()[index] {
                target.set(x, y, role);
            }
        }
    }
}

fn scene_lines(theatre: TheatreFrame, selected: bool) -> Vec<Line<'static>> {
    let lamp = if theatre.focused || selected {
        "(*)"
    } else {
        "(.)"
    };
    let modem = if theatre.animation_frame.is_multiple_of(2) {
        "o*o"
    } else {
        "*o*"
    };
    let (screen, activity) = match theatre.pose {
        TheatrePose::Working => (
            if theatre.animation_frame.is_multiple_of(2) {
                "> BUILD_"
            } else {
                "> BUILD "
            },
            "CURSOR/HAND",
        ),
        TheatrePose::Blocked => ("! HELP !", "RAISED HAND"),
        TheatrePose::DoneUnseen => ("[+] UPD", "UPDATE READY"),
        TheatrePose::DoneSeen => ("[+] DONE", "COMPLETE"),
        TheatrePose::Idle => ("[~] IDLE", "SCREENSAVER"),
        TheatrePose::Exited => ("[x] /\\", "EMPTY CHAIR"),
        TheatrePose::Unknown => ("[?] ???", "UNKNOWN"),
    };

    let mut rows = vec![
        format!(" .--------. {lamp}"),
        format!(" |{screen:<8}|"),
        format!(" |{activity:<12}"),
        " '--------'".to_owned(),
        " ======DESK=====".to_owned(),
        format!(" {modem} MODEM"),
    ];
    if theatre.pose == TheatrePose::DoneUnseen && (1..=8).contains(&theatre.animation_frame) {
        const CONFETTI: [(usize, usize, char); 8] = [
            (0, 1, '^'),
            (0, 8, '^'),
            (1, 0, '^'),
            (2, 1, '^'),
            (3, 0, '^'),
            (4, 1, '^'),
            (4, 13, '^'),
            (5, 0, '^'),
        ];
        let (row, column, glyph) = CONFETTI[usize::from(theatre.animation_frame - 1)];
        replace_ascii_char(&mut rows[row], column, glyph);
    }
    rows.into_iter().map(Line::from).collect()
}

fn replace_ascii_char(text: &mut String, index: usize, replacement: char) {
    if index < text.len() {
        text.replace_range(index..=index, &replacement.to_string());
    }
}

fn ascii_pose(pose: TheatrePose) -> Vec<Line<'static>> {
    let rows: [&str; 6] = match pose {
        TheatrePose::Working => [
            "  AGENT   ",
            "   o>_    ",
            "  /|\\     ",
            "  / \\     ",
            "  CHAIR   ",
            "  typing  ",
        ],
        TheatrePose::Blocked => [
            " AGENT [!]",
            "   \\o/    ",
            "    |     ",
            "   / \\    ",
            "  CHAIR   ",
            " raised   ",
        ],
        TheatrePose::DoneUnseen => [
            " AGENT [+]",
            "   \\o/    ",
            "    |     ",
            "   / \\    ",
            "  CHAIR   ",
            " update   ",
        ],
        TheatrePose::DoneSeen => [
            " AGENT [+]",
            "    o     ",
            "   /|\\    ",
            "   / \\    ",
            "  CHAIR   ",
            "   done   ",
        ],
        TheatrePose::Idle => [
            " AGENT [~]",
            "    o     ",
            "   /|\\    ",
            "   / \\    ",
            "  CHAIR   ",
            "   idle   ",
        ],
        TheatrePose::Exited => [
            "   [x]    ",
            "  EMPTY   ",
            "  CHAIR   ",
            "   /_\\    ",
            "          ",
            "  exited  ",
        ],
        TheatrePose::Unknown => [
            " AGENT [?]",
            "    ?     ",
            "   /|\\    ",
            "   / \\    ",
            "  CHAIR   ",
            " unknown  ",
        ],
    };
    rows.into_iter().map(Line::from).collect()
}

fn state_line(
    agent: &Agent,
    theatre: TheatreFrame,
    width: u16,
    character_set: CharacterSet,
) -> String {
    let mut state = format!("{} {}", state_marker(theatre.pose), theatre.label);
    if theatre.focused {
        state.push_str(" | LIVE");
    }
    if let Some(status) = agent.custom_status.as_deref() {
        let suffix = format!(" | {}", present(status, character_set));
        if state.chars().count() + suffix.chars().count() <= usize::from(width) {
            state.push_str(&suffix);
        }
    }
    state
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
