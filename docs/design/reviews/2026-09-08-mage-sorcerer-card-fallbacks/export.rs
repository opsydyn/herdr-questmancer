//! Card fallback review through current production sprites and Ratatui overlays.
use questmancer::{
    app::{CharacterSet, ColorMode, Model, Motion, View},
    domain::{
        AdventurerClass, AdventurerPersona, AgentKey, GuildAttention, GuildSummons, PersonaKey,
        Presence, Timestamp,
    },
    scene::{
        assets::{
            adventurer::{adventurer_animation_frame, adventurer_portrait_frame},
            archetypes,
        },
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::SpriteFrame,
        stage::ScenePose,
    },
    storybook::fixtures::{StoryContext, guild_world_fixture},
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
        "actors":scene.actors.len(), "actor_bounds":scene.actors.iter().map(|a|json!([a.bounds.x,a.bounds.y,a.bounds.width,a.bounds.height])).collect::<Vec<_>>(), "librarian":scene.interactables.iter().map(|r|json!([r.bounds.x,r.bounds.y,r.bounds.width,r.bounds.height])).collect::<Vec<_>>(),
    })
}

fn frame_json(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,"pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}
const CLASSES: [AdventurerClass; 2] = [AdventurerClass::Mage, AdventurerClass::Sorcerer];
fn persona(index: usize) -> AdventurerPersona {
    let mut p = AdventurerPersona::for_key(PersonaKey::new(format!("pilot-{index}")));
    p.class = CLASSES[index % CLASSES.len()];
    p
}
fn party(view: View, presence: Presence, age: i64, motion: Motion, count: usize) -> Model {
    let mut model = guild_world_fixture(StoryContext::fixed());
    model.switch_to(view);
    let original = model.domain().agents.values().next().unwrap().clone();
    model.domain_mut().agents.clear();
    model.domain_mut().selected_agent = None;
    model
        .domain_mut()
        .campaigns
        .retain(|k, _| *k == original.workspace_id);
    for campaign in model.domain_mut().campaigns.values_mut() {
        campaign.party.clear();
    }
    for index in 0..count {
        let mut agent = original.clone();
        agent.key = AgentKey::new(format!("pilot-{index}"));
        agent.persona = persona(index);
        agent.name = ["Aria", "Tobin"][index % 2].to_owned();
        agent.presence = presence;
        agent.presence_since = Timestamp::from_millis(200_000);
        agent.attention = match presence {
            Presence::Blocked => {
                GuildAttention::unread(GuildSummons::CounselRequested, agent.presence_since)
            }
            Presence::Done => {
                GuildAttention::unread(GuildSummons::SpoilsReturned, agent.presence_since)
            }
            _ => GuildAttention::Clear,
        };
        agent.custom_status = None;
        model
            .domain_mut()
            .campaigns
            .values_mut()
            .next()
            .unwrap()
            .party
            .push(agent.key.clone());
        model.domain_mut().agents.insert(agent.key.clone(), agent);
    }
    let mut prefs = *model.preferences();
    prefs.motion = motion;
    model.set_preferences(prefs);
    model.set_now(Timestamp::from_millis(200_000 + age));
    model
}
fn main() {
    let output = env::args().nth(2).expect("output JSON");
    let mut data = serde_json::Map::new();
    for (index, class) in CLASSES.into_iter().enumerate() {
        let p = persona(index);
        data.insert(
            format!("before-{index}"),
            frame_json(&archetypes::portrait_frame(class).unwrap()),
        );
        data.insert(
            format!("after-{index}"),
            frame_json(&adventurer_portrait_frame(&p).unwrap()),
        );
        data.insert(
            format!("world-{index}"),
            frame_json(&adventurer_animation_frame(&p, ScenePose::Working, 0)),
        );
        let mut m = party(View::Guild, Presence::Blocked, 1_000, Motion::None, 1);
        let a = m.domain_mut().agents.values_mut().next().unwrap();
        a.persona = p;
        a.name = ["Aria", "Tobin"][index].to_owned();
        m.select_agent(&AgentKey::new("pilot-0"));
        m.show_adventurer_card();
        data.insert(
            format!("card-{index}"),
            render(&m, PixelSize::new(100, 60), ColorMode::Xterm256, true),
        );
        let mut prefs = *m.preferences();
        prefs.color_mode = ColorMode::Ansi16;
        prefs.character_set = CharacterSet::Ascii;
        m.set_preferences(prefs);
        data.insert(
            format!("card-ansi-{index}"),
            render(&m, PixelSize::new(100, 60), ColorMode::Ansi16, true),
        );
    }
    fs::write(output, serde_json::to_string(&data).unwrap()).unwrap();
    println!("Exported current personalised card fallbacks, prior masters and real card overlays.");
}
