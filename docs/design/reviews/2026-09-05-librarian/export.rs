//! Librarian review: production assets, room painters and Ratatui buffers.
use questmancer::{
    app::{ColorMode, Model, Motion},
    domain::Presence,
    scene::{
        assets::{IndexedPaletteEntry, indexed_sprite, librarian},
        pixel::{PixelPoint, PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::{SpriteFrame, blit},
    },
    storybook::fixtures::{StoryContext, guild_world_fixture, librarian_ledger_fixture},
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
        "actors":scene.actors.len(), "librarian":scene.interactables.iter().map(|r|json!([r.bounds.x,r.bounds.y,r.bounds.width,r.bounds.height])).collect::<Vec<_>>(),
    })
}

fn frame_json(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,"pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}
fn on_background(frame: &SpriteFrame, bg: Rgb, mode: ColorMode) -> Value {
    let mut pixels = RgbBuffer::filled(frame.size().width, frame.size().height, bg);
    blit(frame, PixelPoint::new(0, 0), &mut pixels);
    let size = frame.size();
    let area = Rect::new(0, 0, size.width, size.height.div_ceil(2));
    let mut converted = Buffer::empty(area);
    flush_rgb(&mut converted, area, &pixels, bg, mode);
    let pixels = (0..size.height)
        .flat_map(|y| {
            let cells = &converted;
            (0..size.width).map(move |x| {
                let c = cells.cell((x, y / 2)).unwrap();
                colour(if y % 2 == 0 { c.fg } else { c.bg })
            })
        })
        .collect::<Vec<_>>();
    json!({"width":size.width,"height":size.height,"pixels":pixels})
}
fn main() {
    let directory = env::args().nth(1).expect("review directory");
    let output = env::args().nth(2).expect("output JSON");
    let baseline: Value =
        serde_json::from_str(&fs::read_to_string(format!("{directory}/baseline.json")).unwrap())
            .unwrap();
    let palette = baseline["palette"]
        .as_object()
        .unwrap()
        .iter()
        .map(|(k, v)| IndexedPaletteEntry {
            key: k.chars().next().unwrap(),
            colour: Some(Rgb::new(
                v[0].as_u64().unwrap() as u8,
                v[1].as_u64().unwrap() as u8,
                v[2].as_u64().unwrap() as u8,
            )),
        })
        .collect::<Vec<_>>();
    let rows = baseline["world"]
        .as_array()
        .unwrap()
        .iter()
        .map(|r| r.as_str().unwrap())
        .collect::<Vec<_>>();
    let before = indexed_sprite(&rows, &palette).unwrap();
    let mut old_portrait = vec![None; 24 * 32];
    for y in 0..24 {
        for x in 0..16 {
            old_portrait[(y + 4) * 24 + x + 4] = before.pixels()[y * 16 + x];
        }
    }
    let old_portrait = SpriteFrame::from_pixels(24, 32, old_portrait);
    let mut data = serde_json::Map::new();
    data.insert("before-world".into(), frame_json(&before));
    data.insert("before-portrait".into(), frame_json(&old_portrait));
    data.insert("world".into(), frame_json(librarian::world()));
    data.insert("portrait".into(), frame_json(librarian::ledger_portrait()));
    for (label, bg) in [
        ("dark", Rgb::new(31, 33, 40)),
        ("warm", Rgb::new(74, 48, 35)),
        ("parchment", Rgb::new(230, 207, 154)),
    ] {
        for (name, frame) in [
            ("world", librarian::world()),
            ("portrait", librarian::ledger_portrait()),
        ] {
            for (mode, colour) in [("rgb", ColorMode::Xterm256), ("ansi", ColorMode::Ansi16)] {
                data.insert(
                    format!("{name}-{label}-{mode}"),
                    on_background(frame, bg, colour),
                );
            }
        }
    }
    let mut model = guild_world_fixture(StoryContext::fixed());
    let mut prefs = model.preferences().clone();
    prefs.motion = Motion::None;
    model.set_preferences(prefs);
    model.domain_mut().selected_agent = None;
    model
        .domain_mut()
        .agents
        .retain(|_, a| a.presence != Presence::Exited);
    for a in model.domain_mut().agents.values_mut() {
        a.presence = Presence::Idle;
    }
    data.insert(
        "hall".into(),
        render(&model, PixelSize::new(160, 90), ColorMode::Xterm256, false),
    );
    let keys = model
        .domain()
        .agents
        .keys()
        .take(2)
        .cloned()
        .collect::<Vec<_>>();
    model.domain_mut().agents.retain(|k, _| keys.contains(k));
    data.insert(
        "compact".into(),
        render(&model, PixelSize::new(64, 40), ColorMode::Xterm256, false),
    );
    let mut ledger = librarian_ledger_fixture(StoryContext::fixed());
    data.insert(
        "ledger".into(),
        render(&ledger, PixelSize::new(100, 56), ColorMode::Xterm256, true),
    );
    let mut prefs = ledger.preferences().clone();
    prefs.color_mode = ColorMode::Ansi16;
    ledger.set_preferences(prefs);
    data.insert(
        "ledger-ansi".into(),
        render(&ledger, PixelSize::new(100, 56), ColorMode::Ansi16, true),
    );
    fs::write(output, serde_json::to_string(&data).unwrap()).unwrap();
    println!("Librarian production masters, Hall compositions and Ledger buffers exported.");
}
