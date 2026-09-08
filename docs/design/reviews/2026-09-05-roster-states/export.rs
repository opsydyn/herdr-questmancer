//! Terminal-free evidence from the production RGB painters and Ratatui overlays.
use questmancer::{
    app::{CharacterSet, ColorMode, ConnectionState, DisplayPreferences, Model, Motion, View},
    domain::{
        AdventurerClass, AdventurerPersona, Agent, AgentKey, DomainState, GuildAttention,
        GuildSummons, PaneId, PersonaKey, Presence, TabId, Timestamp, WorkspaceId,
    },
    scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
    ui::{
        scene_adapter::flush_rgb,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{env, fs};

fn colour(c: Color) -> [u8; 3] {
    match c {
        Color::Rgb(r, g, b) => [r, g, b],
        Color::Black => [0, 0, 0],
        Color::White => [255, 255, 255],
        Color::Gray => [170, 170, 170],
        Color::DarkGray => [85, 85, 85],
        Color::Yellow => [170, 85, 0],
        Color::LightYellow => [255, 255, 85],
        Color::Red => [170, 0, 0],
        Color::LightRed => [255, 85, 85],
        Color::Green => [0, 170, 0],
        Color::LightGreen => [85, 255, 85],
        Color::Blue => [0, 0, 170],
        Color::LightBlue => [85, 85, 255],
        Color::Magenta => [170, 0, 170],
        Color::LightMagenta => [255, 85, 255],
        Color::Cyan => [0, 170, 170],
        Color::LightCyan => [85, 255, 255],
        _ => [25, 25, 30],
    }
}
fn model(states: &[Presence], view: View, motion: Motion, now: i64) -> Model {
    let mut domain = DomainState::default();
    for (index, presence) in states.iter().copied().enumerate() {
        let key = AgentKey::new(format!("review-{index:02}"));
        let mut persona =
            AdventurerPersona::for_key(PersonaKey::new(format!("roster-review-{index}")));
        persona.class = [
            AdventurerClass::Wizard,
            AdventurerClass::Ranger,
            AdventurerClass::Barbarian,
        ][index % 3];
        domain.agents.insert(
            key.clone(),
            Agent {
                key: key.clone(),
                pane_id: PaneId::new(format!("review-pane-{index}")),
                workspace_id: WorkspaceId::new("review-campaign"),
                tab_id: TabId::new("review-tab"),
                name: ["Willow", "Bram", "Orin"][index % 3].to_owned(),
                custom_status: None,
                presence,
                presence_since: Timestamp::from_millis(1_000),
                attention: match presence {
                    Presence::Done => GuildAttention::unread(
                        GuildSummons::SpoilsReturned,
                        Timestamp::from_millis(1_000),
                    ),
                    Presence::Blocked => GuildAttention::unread(
                        GuildSummons::CounselRequested,
                        Timestamp::from_millis(1_000),
                    ),
                    _ => GuildAttention::Clear,
                },
                focused: false,
                pane_revision: 1,
                persona,
            },
        );
        if index == 0 {
            domain.selected_agent = Some(key);
        }
    }
    let mut model = Model::new(view);
    model.set_connection(ConnectionState::Connected);
    model.replace_domain(domain);
    model.set_preferences(DisplayPreferences {
        motion,
        ..DisplayPreferences::default()
    });
    model.set_now(Timestamp::from_millis(now));
    model
}
fn render(model: &Model, size: PixelSize, mode: ColorMode, overlays: bool) -> Value {
    let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let presentation = ScenePresentation::from_model(model);
    let scene = render_scene_for_world(
        &SceneSnapshot::from_model(model),
        &presentation,
        size,
        &mut pixels,
    );
    let area = Rect::new(0, 0, size.width, size.height.div_ceil(2));
    let mut converted = Buffer::empty(area);
    flush_rgb(&mut converted, area, &pixels, Rgb::BLACK, mode);
    let converted_pixels = (0..size.height)
        .flat_map(|y| {
            let cells = &converted;
            (0..size.width).map(move |x| {
                let c = cells.cell((x, y / 2)).unwrap();
                colour(if y % 2 == 0 { c.fg } else { c.bg })
            })
        })
        .collect::<Vec<_>>();
    let mut terminal = Terminal::new(TestBackend::new(area.width, area.height)).unwrap();
    terminal
        .draw(|frame| {
            flush_rgb(frame.buffer_mut(), area, &pixels, Rgb::BLACK, mode);
            if overlays {
                render_scene_identity_labels(frame, model, &scene);
                render_scene_overlays(frame, model, &presentation, None);
            }
        })
        .unwrap();
    json!({
        "width":size.width,"height":size.height,"pixels":converted_pixels,
        "columns":area.width,"rows":area.height,
        "cells":terminal.backend().buffer().content.iter().map(|c|json!({"text":c.symbol(),"fg":colour(c.fg),"bg":colour(c.bg)})).collect::<Vec<_>>(),
        "next_ms":scene.next_frame_in.map(|d|d.as_millis()),
        "actors":scene.actors.len(),
    })
}
fn main() {
    let output = env::args().nth(2).expect("output JSON path");
    let mut data = serde_json::Map::new();
    for (world, view) in [("guild", View::Guild), ("delve", View::Delve)] {
        for (name, presence) in [
            ("working", Presence::Working),
            ("counsel", Presence::Blocked),
            ("resting", Presence::Idle),
            ("completed", Presence::Done),
            ("unknown", Presence::Unknown),
        ] {
            for (mode, colour) in [("rgb", ColorMode::Xterm256), ("ansi", ColorMode::Ansi16)] {
                data.insert(
                    format!("{world}-{name}-{mode}"),
                    render(
                        &model(&[presence], view, Motion::None, 4_000),
                        PixelSize::new(30, 30),
                        colour,
                        false,
                    ),
                );
            }
        }
        for elapsed in [0, 125, 250, 375, 500, 625, 750, 875, 2999, 3000, 6000] {
            data.insert(
                format!("{world}-return-{elapsed}"),
                render(
                    &model(&[Presence::Done], view, Motion::Full, 1_000 + elapsed),
                    PixelSize::new(30, 30),
                    ColorMode::Xterm256,
                    false,
                ),
            );
        }
        for (name, motion) in [("reduced", Motion::Reduced), ("still", Motion::None)] {
            data.insert(
                format!("{world}-{name}"),
                render(
                    &model(&[Presence::Done], view, motion, 1_125),
                    PixelSize::new(30, 30),
                    ColorMode::Xterm256,
                    false,
                ),
            );
        }
        let party = [
            Presence::Working,
            Presence::Blocked,
            Presence::Done,
            Presence::Idle,
            Presence::Unknown,
            Presence::Working,
        ];
        data.insert(
            format!("{world}-crowded"),
            render(
                &model(&party, view, Motion::None, 4_000),
                PixelSize::new(30, 48),
                ColorMode::Xterm256,
                true,
            ),
        );
        let mut offline = model(&party, view, Motion::Full, 4_000);
        offline.set_connection(ConnectionState::Reconnecting { attempt: 3 });
        data.insert(
            format!("{world}-reconnect"),
            render(&offline, PixelSize::new(30, 48), ColorMode::Xterm256, true),
        );
        for (name, colour, characters) in [
            ("unicode", ColorMode::Xterm256, CharacterSet::Unicode),
            ("ascii", ColorMode::Ansi16, CharacterSet::Ascii),
        ] {
            let mut labels = model(&party, view, Motion::None, 4_000);
            labels.set_preferences(DisplayPreferences {
                motion: Motion::None,
                character_set: characters,
                color_mode: colour,
            });
            data.insert(
                format!("{world}-labels-{name}"),
                render(&labels, PixelSize::new(60, 30), colour, true),
            );
        }
    }
    fs::write(output, serde_json::to_vec(&data).unwrap()).unwrap();
}
