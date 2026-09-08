//! Static review export. It uses Questmancer's indexed_sprite, scene painters,
//! and half-block adapter; no production asset routes are replaced.
use questmancer::{
    app::{ColorMode, Motion, View},
    domain::{
        AdventurerClass, AdventurerPersona, Garb, GuildAttention, HairTone, PersonaKey, Presence,
        SkinTone,
    },
    scene::{
        assets::{
            IndexedPaletteEntry,
            adventurer::{
                adventurer_animation_frame, adventurer_portrait_frame, adventurer_roster_frame,
            },
            indexed_sprite,
        },
        pixel::{PixelPoint, PixelRect, PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render, render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::{SpriteFrame, blit},
        stage::{ScenePlan, ScenePose, WorldScene},
    },
    storybook::fixtures::{StoryContext, delve_world_fixture, guild_world_fixture},
    ui::{
        scene_adapter::flush_rgb,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs};

fn rgb(hex: &str) -> Rgb {
    Rgb::new(
        u8::from_str_radix(&hex[0..2], 16).unwrap(),
        u8::from_str_radix(&hex[2..4], 16).unwrap(),
        u8::from_str_radix(&hex[4..6], 16).unwrap(),
    )
}
fn frame(rows: &Value, palette: &[IndexedPaletteEntry]) -> SpriteFrame {
    let rows = rows
        .as_array()
        .unwrap()
        .iter()
        .map(|v| v.as_str().unwrap())
        .collect::<Vec<_>>();
    indexed_sprite(&rows, palette).expect("review frame must use valid authored pixel data")
}
fn frame_json(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,"pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}
fn buffer_json(buffer: &RgbBuffer) -> Value {
    json!({"width":buffer.size().width,"height":buffer.size().height,"pixels":buffer.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>()})
}
fn persona(class: AdventurerClass) -> AdventurerPersona {
    let mut p = AdventurerPersona::for_key(PersonaKey::new("storyboard-proportion-review"));
    p.class = class;
    p.appearance.skin_tone = SkinTone::Rose;
    p.appearance.garb = Garb::Vestments;
    p.appearance.hair_tone = match class {
        AdventurerClass::Sorcerer => HairTone::Gold,
        _ => HairTone::Chestnut,
    };
    p
}
fn foot(frame: &SpriteFrame) -> (usize, usize) {
    let w = usize::from(frame.size().width);
    let y = frame.pixels().iter().rposition(Option::is_some).unwrap() / w;
    let xs = frame.pixels()[y * w..(y + 1) * w]
        .iter()
        .enumerate()
        .filter_map(|(x, p)| p.is_some().then_some(x))
        .collect::<Vec<_>>();
    (y, xs[0] + xs[xs.len() - 1])
}
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
fn cells(buffer: &Buffer) -> Value {
    json!({"columns":buffer.area.width,"rows":buffer.area.height,"cells":buffer.content.iter().map(|c|json!({"text":c.symbol(),"fg":colour(c.fg),"bg":colour(c.bg)})).collect::<Vec<_>>()})
}
fn main() {
    let directory = env::args().nth(1).expect("review directory");
    let input: Value =
        serde_json::from_str(&fs::read_to_string(format!("{directory}/candidates.json")).unwrap())
            .unwrap();
    let mut outputs = serde_json::Map::new();
    let mut masters = BTreeMap::new();
    for c in input["classes"].as_array().unwrap() {
        let id = c["id"].as_str().unwrap();
        let class = match id {
            "mage" => AdventurerClass::Mage,
            "sorcerer" => AdventurerClass::Sorcerer,
            _ => unreachable!(),
        };
        let palette = c["palette"]
            .as_object()
            .unwrap()
            .iter()
            .map(|(k, v)| IndexedPaletteEntry {
                key: k.chars().next().unwrap(),
                colour: Some(rgb(v.as_str().unwrap())),
            })
            .collect::<Vec<_>>();
        let p = persona(class);
        let before = adventurer_animation_frame(&p, ScenePose::Working, 0);
        let mut frames = serde_json::Map::new();
        let mut foot_anchor = None;
        for (name, rows) in c["frames"].as_object().unwrap() {
            let f = frame(rows, &palette);
            assert_eq!(f.size(), PixelSize::new(16, 24));
            let anchor = foot(&f);
            assert_eq!(anchor.0, 21, "{id} {name}: foot line drifted");
            if let Some(expected) = foot_anchor {
                assert_eq!(
                    anchor, expected,
                    "{id} {name}: horizontal foot anchor drifted"
                );
            } else {
                foot_anchor = Some(anchor);
            }
            assert!(
                f.pixels().iter().filter(|p| p.is_some()).count() > 100,
                "empty-looking frame"
            );
            frames.insert(name.clone(), frame_json(&f));
            masters.insert((id.to_owned(), name.clone()), f);
        }
        assert_eq!(frames.len(), 8, "each class has eight authored moments");
        assert_ne!(
            frames["working_a"], frames["working_b"],
            "work must change pixels"
        );
        assert_ne!(
            frames["counsel_raise"], frames["counsel_wait"],
            "gesture must settle"
        );
        let work_a = &masters[&(id.to_owned(), "working_a".to_owned())];
        let work_b = &masters[&(id.to_owned(), "working_b".to_owned())];
        assert_eq!(
            &work_a.pixels()[..12 * 16],
            &work_b.pixels()[..12 * 16],
            "working head must stay fixed"
        );
        assert_eq!(
            &work_a.pixels()[19 * 16..],
            &work_b.pixels()[19 * 16..],
            "working feet must stay fixed"
        );
        let roster = adventurer_roster_frame(&p);
        let mut b = RgbBuffer::filled(16, 24, Rgb::new(31, 33, 40));
        blit(
            &masters[&(id.to_owned(), "working_a".to_owned())],
            PixelPoint::new(0, 0),
            &mut b,
        );
        let mut ansi = Buffer::empty(Rect::new(0, 0, 16, 12));
        flush_rgb(
            &mut ansi,
            Rect::new(0, 0, 16, 12),
            &b,
            Rgb::BLACK,
            ColorMode::Ansi16,
        );
        outputs.insert(id.to_owned(),json!({"class":c["label"],"before":frame_json(&before),"portrait":frame_json(&adventurer_portrait_frame(&p).unwrap()),"roster_before":frame_json(&adventurer_roster_frame(&p)),"roster":frame_json(&roster),"frames":frames,"zones":c["zones"],"ansi":cells(&ansi),"palette":c["palette"]}));
        println!(
            "{id}: world frames 16x24; foot line 21 and horizontal anchor stable; current roster retained"
        );
    }
    let marker_palette = [
        ('o', "e6cf9a"),
        ('l', "ffd76b"),
        ('P', "ffecb7"),
        ('M', "dae2d9"),
        ('m', "708486"),
        ('d', "a96f37"),
        ('z', "b4b7b8"),
    ]
    .into_iter()
    .map(|(key, hex)| IndexedPaletteEntry {
        key,
        colour: Some(rgb(hex)),
    })
    .collect::<Vec<_>>();
    let mut markers = serde_json::Map::new();
    for (name, rows) in input["proposed_state_markers"].as_object().unwrap() {
        let f = frame(rows, &marker_palette);
        assert_eq!(f.size(), PixelSize::new(5, 5));
        markers.insert(name.clone(), frame_json(&f));
    }
    let mut rooms = serde_json::Map::new();
    for (name, world, view) in [
        ("guild", WorldScene::GuildHall, View::Guild),
        ("delve", WorldScene::Delve, View::Delve),
    ] {
        let mut model = match view {
            View::Guild => guild_world_fixture(StoryContext::fixed()),
            View::Delve => delve_world_fixture(StoryContext::fixed()),
        };
        let keys = model.domain().agents.keys().cloned().collect::<Vec<_>>();
        // Two fixed study personas and real state labels, in the existing fixture topology.
        for (i, key) in keys.iter().enumerate() {
            let a = model.domain_mut().agents.get_mut(key).unwrap();
            if i < 2 {
                let class = [AdventurerClass::Mage, AdventurerClass::Sorcerer][i];
                a.persona = persona(class);
                a.name = format!("{:?}", class);
                a.presence = [Presence::Working, Presence::Blocked][i];
                a.attention = GuildAttention::Clear;
            } else {
                a.presence = Presence::Exited;
            }
        }
        model.domain_mut().selected_agent = Some(keys[1].clone());
        let mut snapshot = SceneSnapshot::from_model(&model);
        snapshot.motion = Motion::None;
        let presentation = ScenePresentation::from_model(&model);
        let mut before = RgbBuffer::filled(160, 90, Rgb::BLACK);
        let current = render_scene_for_world(
            &snapshot,
            &presentation,
            PixelSize::new(160, 90),
            &mut before,
        );
        let mut plan = ScenePlan::project(&snapshot, PixelSize::new(160, 90));
        plan.world = world;
        plan.actors.clear();
        plan.effects.clear();
        let mut proposed = RgbBuffer::filled(160, 90, Rgb::BLACK);
        render::paint(&snapshot, &plan, PixelSize::new(160, 90), &mut proposed);
        for r in &current.actors {
            let a = model.domain().agents.get(&r.agent).unwrap();
            let id = match a.persona.class {
                AdventurerClass::Mage => "mage",
                AdventurerClass::Sorcerer => "sorcerer",
                _ => unreachable!(),
            };
            let moment = match a.presence {
                Presence::Working => "working_a",
                Presence::Blocked => "counsel_wait",
                _ => "resting",
            };
            let f = &masters[&(id.to_owned(), moment.to_owned())];
            // Placement mock-up: actual current actor bounds, authored candidate pixels.
            proposed.fill_rect(
                PixelRect::new(r.bounds.x + 3, r.bounds.y + 21, 10, 2),
                Rgb::new(24, 22, 30),
            );
            blit(f, PixelPoint::new(r.bounds.x, r.bounds.y), &mut proposed);
            if a.presence == Presence::Blocked {
                let marker = questmancer::scene::assets::guild_hall::frame(
                    questmancer::scene::assets::guild_hall::GuildHallAsset::CounselMarker,
                );
                blit(
                    marker,
                    PixelPoint::new(r.bounds.x + 5, r.bounds.y - 4),
                    &mut proposed,
                );
            }
        }
        // Preserve the current renderer's reserved selection cue in this static study.
        for (source, destination) in before.pixels().iter().zip(proposed.pixels_mut()) {
            if *source == questmancer::scene::assets::palette::SELECTION_RUNE {
                *destination = *source;
            }
        }
        let mut terminal = Terminal::new(TestBackend::new(160, 45)).unwrap();
        terminal
            .draw(|f| {
                flush_rgb(
                    f.buffer_mut(),
                    Rect::new(0, 0, 160, 45),
                    &proposed,
                    Rgb::BLACK,
                    ColorMode::Xterm256,
                );
                render_scene_identity_labels(f, &model, &current);
                render_scene_overlays(f, &model, &presentation, None);
            })
            .unwrap();
        let full = cells(terminal.backend().buffer());
        model.show_adventurer_card();
        terminal
            .draw(|f| {
                flush_rgb(
                    f.buffer_mut(),
                    Rect::new(0, 0, 160, 45),
                    &proposed,
                    Rgb::BLACK,
                    ColorMode::Xterm256,
                );
                render_scene_identity_labels(f, &model, &current);
                render_scene_overlays(f, &model, &presentation, None);
            })
            .unwrap();
        rooms.insert(name.into(),json!({"before":buffer_json(&before),"proposed":buffer_json(&proposed),"labels":full,"card":cells(terminal.backend().buffer())}));
    }
    let output = env::args()
        .nth(2)
        .unwrap_or_else(|| format!("{directory}/rendered.json"));
    fs::write(
        output,
        serde_json::to_string(&json!({"classes":outputs,"rooms":rooms,"markers":markers})).unwrap(),
    )
    .unwrap();
}
