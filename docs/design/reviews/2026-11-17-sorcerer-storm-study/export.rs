//! Review-only Sorcerer pixels and placement exports; production routes stay intact.
use questmancer::{
    app::{ColorMode, Motion, View},
    domain::{
        AdventurerClass, AdventurerPersona, GuildAttention, HairTone, PersonaKey, Presence,
        SkinTone,
    },
    scene::{
        assets::{IndexedPaletteEntry, adventurer::adventurer_animation_frame, indexed_sprite},
        pixel::{PixelPoint, PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render, render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::{SpriteFrame, blit},
        stage::{ScenePlan, ScenePose, WorldScene},
    },
    storybook::fixtures::{StoryContext, delve_world_fixture, guild_world_fixture},
    ui::scene_adapter::flush_rgb,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, path::Path};

fn sprite_json(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,
        "pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}

fn buffer_json(buffer: &RgbBuffer) -> Value {
    json!({"width":buffer.size().width,"height":buffer.size().height,
        "pixels":buffer.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>()})
}

fn half_blocks(buffer: &RgbBuffer) -> Value {
    let size = buffer.size();
    let area = Rect::new(0, 0, size.width, size.height.div_ceil(2));
    let mut cells = Buffer::empty(area);
    flush_rgb(&mut cells, area, buffer, Rgb::BLACK, ColorMode::Xterm256);
    let pixels = (0..size.height)
        .flat_map(|y| {
            let cells = &cells;
            (0..size.width).map(move |x| {
                let cell = cells
                    .cell((x, y / 2))
                    .expect("review cell is within bounds");
                let colour = if y % 2 == 0 { cell.fg } else { cell.bg };
                let Color::Rgb(r, g, b) = colour else {
                    panic!("truecolour half-block export must retain RGB");
                };
                [r, g, b]
            })
        })
        .collect::<Vec<_>>();
    assert_eq!(pixels.len(), buffer.pixels().len());
    for (actual, expected) in pixels.iter().zip(buffer.pixels()) {
        assert_eq!(*actual, [expected.r, expected.g, expected.b]);
    }
    json!({"width":size.width,"height":size.height,"pixels":pixels})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("supply the review directory");
    let directory = Path::new(&directory);
    let input: Value =
        serde_json::from_str(&fs::read_to_string(directory.join("candidates.json"))?)?;
    let palette = input["palette"]
        .as_object()
        .expect("palette object")
        .iter()
        .map(|(key, value)| {
            let hex = value.as_str().expect("hex palette colour");
            IndexedPaletteEntry {
                key: key.chars().next().expect("palette key"),
                colour: Some(Rgb::new(
                    u8::from_str_radix(&hex[..2], 16).expect("red hex"),
                    u8::from_str_radix(&hex[2..4], 16).expect("green hex"),
                    u8::from_str_radix(&hex[4..6], 16).expect("blue hex"),
                )),
            }
        })
        .collect::<Vec<_>>();
    let mut persona = AdventurerPersona::for_key(PersonaKey::new("sorcerer-review-2026-11-17"));
    persona.class = AdventurerClass::Sorcerer;
    persona.appearance.skin_tone = SkinTone::Porcelain;
    persona.appearance.hair_tone = HairTone::Espresso;
    let mut frames = BTreeMap::new();
    frames.insert(
        "current".to_owned(),
        adventurer_animation_frame(&persona, ScenePose::Working, 0),
    );
    frames.insert(
        "current_resting".to_owned(),
        adventurer_animation_frame(&persona, ScenePose::Resting, 0),
    );
    frames.insert(
        "current_unknown".to_owned(),
        adventurer_animation_frame(&persona, ScenePose::Unknown, 0),
    );
    for option in input["options"].as_array().expect("candidate array") {
        let id = option["id"].as_str().expect("candidate ID");
        let rows = option["rows"]
            .as_array()
            .expect("indexed rows")
            .iter()
            .map(|v| v.as_str().expect("pixel row"))
            .collect::<Vec<_>>();
        let frame = indexed_sprite(&rows, &palette).expect("authored candidate must decode");
        assert_eq!(frame.size(), PixelSize::new(16, 24));
        let last = frame
            .pixels()
            .iter()
            .rposition(Option::is_some)
            .expect("nonempty candidate");
        assert_eq!(last / 16, 21, "candidate feet must stay on row 21");
        let feet = frame.pixels()[21 * 16..22 * 16]
            .iter()
            .enumerate()
            .filter_map(|(x, p)| p.is_some().then_some(x))
            .collect::<Vec<_>>();
        assert_eq!(
            feet,
            vec![4, 5, 6, 9, 10, 11],
            "retain the production foot anchors"
        );
        assert!(frame.pixels().iter().filter(|p| p.is_some()).count() > 120);
        frames.insert(id.to_owned(), frame);
    }
    for id in ["storm_working", "storm_resting", "storm_unknown"] {
        assert!(
            frames[id]
                .pixels()
                .iter()
                .enumerate()
                .filter(|(i, p)| i % 16 >= 12 && (11..16).contains(&(i / 16)) && p.is_some())
                .count()
                >= 10,
            "Orb remains visible in {id}"
        );
    }
    let mut rooms = serde_json::Map::new();
    for (name, view) in [("hall", View::Guild), ("delve", View::Delve)] {
        let mut model = match view {
            View::Guild => guild_world_fixture(StoryContext::fixed()),
            View::Delve => delve_world_fixture(StoryContext::fixed()),
        };
        let first = model
            .domain()
            .agents
            .keys()
            .next()
            .expect("fixture actor")
            .clone();
        for (key, agent) in &mut model.domain_mut().agents {
            agent.presence = if *key == first {
                Presence::Working
            } else {
                Presence::Exited
            };
            agent.attention = GuildAttention::Clear;
            agent.persona = persona.clone();
        }
        model.domain_mut().selected_agent = None;
        let mut snapshot = SceneSnapshot::from_model(&model);
        snapshot.motion = Motion::None;
        let presentation = ScenePresentation::from_model(&model);
        let size = PixelSize::new(160, 90);
        let mut before = RgbBuffer::filled(160, 90, Rgb::BLACK);
        let rendered = render_scene_for_world(&snapshot, &presentation, size, &mut before);
        assert_eq!(rendered.actors.len(), 1, "one Sorcerer placement per room");
        let bounds = rendered.actors[0].bounds;
        let mut plan = ScenePlan::project(&snapshot, size);
        plan.world = match view {
            View::Guild => WorldScene::GuildHall,
            View::Delve => WorldScene::Delve,
        };
        plan.actors.clear();
        plan.effects.clear();
        let mut background = RgbBuffer::filled(160, 90, Rgb::BLACK);
        render::paint(&snapshot, &plan, size, &mut background);
        let mut options = serde_json::Map::new();
        for (id, frame) in &frames {
            let mut pixels = background.clone();
            blit(frame, PixelPoint::new(bounds.x, bounds.y), &mut pixels);
            options.insert(id.clone(), half_blocks(&pixels));
        }
        rooms.insert(name.into(),json!({"production":buffer_json(&before),"background":buffer_json(&background),"bounds":[bounds.x,bounds.y,bounds.width,bounds.height],"options":options}));
    }
    let output = json!({"frames":frames.iter().map(|(id,frame)|(id.clone(),sprite_json(frame))).collect::<BTreeMap<_,_>>(),"rooms":rooms});
    fs::write(
        directory.join("rendered.json"),
        serde_json::to_string(&output)?,
    )?;
    println!(
        "Storm Channeler study frames decoded; row-21 foot anchors retained; two room placements and lossless RGB half-block conversion checked."
    );
    Ok(())
}
