//! Review-only Librarian pixels and Hall placements, with no runtime effects.
use questmancer::{
    app::{ColorMode, Motion},
    domain::Presence,
    scene::{
        assets::{IndexedPaletteEntry, indexed_sprite, librarian},
        pixel::{PixelPoint, PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::{SpriteFrame, blit},
    },
    storybook::fixtures::{StoryContext, guild_world_fixture},
    ui::scene_adapter::flush_rgb,
};
use ratatui::{buffer::Buffer, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{collections::BTreeMap, env, fs, path::Path};

fn sprite(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,"pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}
fn half_blocks(rgb: &RgbBuffer) -> Value {
    let size = rgb.size();
    let area = Rect::new(0, 0, size.width, size.height.div_ceil(2));
    let mut cells = Buffer::empty(area);
    flush_rgb(&mut cells, area, rgb, Rgb::BLACK, ColorMode::Xterm256);
    let pixels = (0..size.height)
        .flat_map(|y| {
            let cells = &cells;
            (0..size.width).map(move |x| {
                let cell = cells.cell((x, y / 2)).expect("review cell bounds");
                let colour = if y % 2 == 0 { cell.fg } else { cell.bg };
                let Color::Rgb(r, g, b) = colour else {
                    panic!("truecolour adapter output");
                };
                [r, g, b]
            })
        })
        .collect::<Vec<_>>();
    assert!(
        pixels
            .iter()
            .zip(rgb.pixels())
            .all(|(a, b)| *a == [b.r, b.g, b.b]),
        "lossless RGB half-block export"
    );
    json!({"width":size.width,"height":size.height,"pixels":pixels})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("review directory");
    let directory = Path::new(&directory);
    let output = env::args().nth(2).expect("output file");
    let input: Value =
        serde_json::from_str(&fs::read_to_string(directory.join("candidates.json"))?)?;
    let palette = input["palette"]
        .as_object()
        .expect("palette")
        .iter()
        .map(|(key, value)| {
            let hex = value.as_str().expect("hex colour");
            IndexedPaletteEntry {
                key: key.chars().next().expect("palette key"),
                colour: Some(Rgb::new(
                    u8::from_str_radix(&hex[..2], 16).expect("red"),
                    u8::from_str_radix(&hex[2..4], 16).expect("green"),
                    u8::from_str_radix(&hex[4..6], 16).expect("blue"),
                )),
            }
        })
        .collect::<Vec<_>>();
    let mut frames = BTreeMap::new();
    frames.insert("current".to_owned(), librarian::world().clone());
    for option in input["options"].as_array().expect("options") {
        let rows = option["rows"]
            .as_array()
            .expect("rows")
            .iter()
            .map(|r| r.as_str().expect("row"))
            .collect::<Vec<_>>();
        let frame = indexed_sprite(&rows, &palette).expect("authored indexed pixels");
        assert_eq!(frame.size(), PixelSize::new(16, 24));
        assert_eq!(
            frame
                .pixels()
                .iter()
                .rposition(Option::is_some)
                .expect("nonempty")
                / 16,
            21
        );
        let feet = frame.pixels()[21 * 16..22 * 16]
            .iter()
            .enumerate()
            .filter_map(|(x, p)| p.is_some().then_some(x))
            .collect::<Vec<_>>();
        assert_eq!(feet, vec![3, 4, 5, 6, 8, 9, 10, 11]);
        frames.insert(option["id"].as_str().expect("ID").to_owned(), frame);
    }
    assert_ne!(frames["hearth"], frames["stacks"]);
    let mut model = guild_world_fixture(StoryContext::fixed());
    let keep = model
        .domain()
        .agents
        .keys()
        .take(2)
        .cloned()
        .collect::<Vec<_>>();
    model.domain_mut().agents.retain(|k, _| keep.contains(k));
    for a in model.domain_mut().agents.values_mut() {
        a.presence = Presence::Idle;
    }
    model.domain_mut().selected_agent = None;
    let mut prefs = *model.preferences();
    prefs.motion = Motion::None;
    model.set_preferences(prefs);
    let mut rooms = serde_json::Map::new();
    for (name, size) in [
        ("canonical", PixelSize::new(160, 90)),
        ("compact", PixelSize::new(64, 40)),
    ] {
        let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
        let presentation = ScenePresentation::from_model(&model);
        let rendered = render_scene_for_world(
            &SceneSnapshot::from_model(&model),
            &presentation,
            size,
            &mut rgb,
        );
        assert_eq!(
            rendered.interactables.len(),
            1,
            "complete Librarian hit region"
        );
        let bounds = rendered.interactables[0].bounds;
        assert_eq!((bounds.width, bounds.height), (16, 24));
        let mut options = serde_json::Map::new();
        for (id, frame) in &frames {
            let mut room = rgb.clone();
            blit(frame, PixelPoint::new(bounds.x, bounds.y), &mut room);
            options.insert(id.clone(), half_blocks(&room));
        }
        rooms.insert(name.into(),json!({"bounds":[bounds.x,bounds.y,bounds.width,bounds.height],"background":half_blocks(&rgb),"options":options}));
    }
    let out = json!({"frames":frames.iter().map(|(k,v)|(k.clone(),sprite(v))).collect::<BTreeMap<_,_>>(),"portrait":sprite(librarian::ledger_portrait()),"rooms":rooms});
    fs::write(output, serde_json::to_string(&out)?)?;
    println!(
        "Two Librarian studies decoded; 16x24, row-21 feet, Hall bounds and half-block RGB checks passed."
    );
    Ok(())
}
