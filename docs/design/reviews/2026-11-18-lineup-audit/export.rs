//! Export production assets and overlays from fixed Storybook fixtures.
use questmancer::{
    app::{Model, View},
    domain::AdventurerClass,
    interaction::reduce_action,
    portrait::{librarian_asset, portrait_asset},
    scene::{
        assets::{
            adventurer::{
                adventurer_animation_frame, adventurer_portrait_frame, adventurer_roster_frame,
            },
            librarian,
        },
        fit::{CellPixels, FitScene},
        pixel::Rgb,
        presentation::ScenePresentation,
        sprite::SpriteFrame,
        stage::ScenePose,
    },
    storybook::{
        density::{PartySize, fixture},
        fixtures::StoryContext,
    },
    ui::{input::Action, scene_overlays::render_scene_overlays},
};
use ratatui::{Terminal, backend::TestBackend, buffer::Buffer, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

fn rgb(pixel: Rgb) -> [u8; 3] {
    [pixel.r, pixel.g, pixel.b]
}

fn sprite(frame: &SpriteFrame) -> Value {
    json!({"size": [frame.size().width, frame.size().height], "pixels": frame.pixels().iter().map(|p| p.map(rgb)).collect::<Vec<_>>()})
}

fn overlay(model: &Model, width: u16, height: u16) -> Buffer {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    let presentation = ScenePresentation::from_model(model);
    terminal
        .draw(|frame| {
            render_scene_overlays(frame, model, &presentation, None);
        })
        .unwrap();
    terminal.backend().buffer().clone()
}

fn colour(colour: Color) -> Option<[u8; 3]> {
    match colour {
        Color::Rgb(r, g, b) => Some([r, g, b]),
        _ => None,
    }
}

fn picker_preview(model: &Model, class: AdventurerClass) -> Value {
    let buffer = overlay(model, 100, 28);
    let text: String = buffer.content.iter().map(|cell| cell.symbol()).collect();
    assert!(text.contains(&format!("Preview · {}", class.label())));
    let mut halves = buffer.content.iter().filter(|cell| cell.symbol() == "▀");
    let mut pixels = Vec::with_capacity(24 * 32);
    for _ in 0..16 {
        let row = halves.by_ref().take(24).collect::<Vec<_>>();
        assert_eq!(row.len(), 24);
        pixels.extend(row.iter().map(|cell| colour(cell.fg).unwrap()));
        pixels.extend(row.iter().map(|cell| colour(cell.bg).unwrap()));
    }
    assert!(halves.next().is_none());
    json!({"size": [24, 32], "pixels": pixels})
}

fn room(view: View) -> Value {
    let model = fixture(StoryContext::fixed(), PartySize::AllClasses, view);
    let fit = FitScene::project(
        &model,
        Rect::new(0, 0, 104, 30),
        CellPixels::new(8, 16).unwrap(),
    )
    .unwrap();
    assert_eq!(fit.frame.actors.len(), AdventurerClass::ALL.len());
    assert!(
        fit.frame
            .actors
            .iter()
            .all(|actor| (actor.bounds.width, actor.bounds.height) == (8, 12))
    );
    json!({"size": [160, 90], "pixels": fit.rgb.pixels().iter().copied().map(rgb).collect::<Vec<_>>()})
}

fn inbox(width: u16, height: u16) -> Value {
    let mut model = Model::new(View::Guild);
    let _ = reduce_action(&mut model, Action::OpenQuestBoard);
    let _ = reduce_action(&mut model, Action::OpenInbox);
    let buffer = overlay(&model, width, height);
    assert_eq!(
        buffer
            .content
            .iter()
            .filter(|cell| cell.symbol() == "▀")
            .count(),
        if width >= 74 { 50 } else { 0 }
    );
    json!({"size":[width,height],"cells":buffer.content.iter().map(|cell|json!([cell.symbol(),colour(cell.fg),colour(cell.bg)])).collect::<Vec<_>>()})
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let output = env::args().nth(1).expect("temporary review directory");
    let output = Path::new(&output);
    let mut model = fixture(StoryContext::fixed(), PartySize::AllClasses, View::Guild);
    let mut entries = Vec::new();
    for (index, &class) in AdventurerClass::ALL.iter().enumerate() {
        let agent = model
            .domain()
            .agents
            .values()
            .find(|agent| agent.persona.class == class)
            .unwrap();
        let persona = agent.persona.clone();
        let key = agent.key.clone();
        model.select_agent(&key);
        model.open_character_picker();
        let native = format!("{index}.png");
        fs::write(
            output.join(&native),
            portrait_asset(class).expect("selectable class has a native card"),
        )?;
        entries.push(json!({
            "label": class.label(), "native": native,
            "world": sprite(&adventurer_animation_frame(&persona, ScenePose::Resting, 0)),
            "roster": sprite(&adventurer_roster_frame(&persona)),
            "fallback": sprite(&adventurer_portrait_frame(&persona).expect("card fallback")),
            "picker": picker_preview(&model, class),
        }));
        model.dismiss_modal();
    }
    fs::write(output.join("librarian.png"), librarian_asset())?;
    entries.push(
        json!({"label": "Librarian", "native": "librarian.png", "world": sprite(librarian::world()),
        "fallback": sprite(librarian::ledger_portrait())}),
    );
    let evidence = json!({"lineup":entries,"hall":room(View::Guild),"delve":room(View::Delve),"inbox":[inbox(84,20),inbox(52,16)]});
    fs::write(output.join("evidence.json"), serde_json::to_vec(&evidence)?)?;
    Ok(())
}
