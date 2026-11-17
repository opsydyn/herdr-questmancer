//! Production Cleric review fixtures; no Herdr connection or durable state.
use questmancer::{
    app::{ColorMode, Model, Motion, View},
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, GuildAttention, GuildSummons, HairTone,
        PersonaKey, Presence, SkinTone, Timestamp,
    },
    scene::{
        assets::adventurer::{
            adventurer_animation_frame, adventurer_portrait_frame, adventurer_roster_frame,
        },
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::SpriteFrame,
        stage::ScenePose,
    },
    storybook::fixtures::{StoryContext, delve_world_fixture, guild_world_fixture},
    ui::{
        scene_adapter::flush_rgb,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use ratatui::{Terminal, backend::TestBackend, layout::Rect, style::Color};
use serde_json::{Value, json};
use std::{env, fs, path::Path};

fn colour(c: Color) -> [u8; 3] {
    match c {
        Color::Rgb(r, g, b) => [r, g, b],
        _ => panic!("truecolour fixture must contain RGB colours"),
    }
}
fn sprite(frame: &SpriteFrame) -> Value {
    json!({"width":frame.size().width,"height":frame.size().height,"pixels":frame.pixels().iter().map(|p|p.map(|p|[p.r,p.g,p.b])).collect::<Vec<_>>()})
}
fn persona() -> AdventurerPersona {
    let mut p = AdventurerPersona::for_key(PersonaKey::new("cleric-review-2026-11-17"));
    p.class = AdventurerClass::Cleric;
    p.appearance.skin_tone = SkinTone::Porcelain;
    p.appearance.hair_tone = HairTone::Chestnut;
    p.appearance.accent = AccentTone::Cyan;
    p
}
fn model(view: View, presence: Presence, age: i64) -> Model {
    let mut m = match view {
        View::Guild => guild_world_fixture(StoryContext::fixed()),
        View::Delve => delve_world_fixture(StoryContext::fixed()),
    };
    let first = m
        .domain()
        .agents
        .keys()
        .next()
        .expect("fixture actor")
        .clone();
    m.domain_mut().agents.retain(|k, _| *k == first);
    let a = m
        .domain_mut()
        .agents
        .get_mut(&first)
        .expect("retained actor");
    a.persona = persona();
    a.name = "Sunward Chaplain".into();
    a.presence = presence;
    a.presence_since = Timestamp::from_millis(200_000);
    a.attention = match presence {
        Presence::Blocked => {
            GuildAttention::unread(GuildSummons::CounselRequested, a.presence_since)
        }
        Presence::Done => GuildAttention::unread(GuildSummons::SpoilsReturned, a.presence_since),
        _ => GuildAttention::Clear,
    };
    let workspace = a.workspace_id.clone();
    m.domain_mut().campaigns.retain(|k, _| *k == workspace);
    m.domain_mut()
        .campaigns
        .get_mut(&workspace)
        .expect("fixture campaign")
        .party = vec![first];
    m.domain_mut().selected_agent = None;
    let mut prefs = *m.preferences();
    prefs.motion = Motion::Full;
    m.set_preferences(prefs);
    m.set_now(Timestamp::from_millis(200_000 + age));
    m
}
fn render(m: &Model, card: bool) -> Value {
    let mut m = m.clone();
    if card {
        let key = m.domain().agents.keys().next().expect("card actor").clone();
        m.select_agent(&key);
        m.show_adventurer_card();
    }
    let mut rgb = RgbBuffer::filled(160, 90, Rgb::BLACK);
    let presentation = ScenePresentation::from_model(&m);
    let frame = render_scene_for_world(
        &SceneSnapshot::from_model(&m),
        &presentation,
        PixelSize::new(160, 90),
        &mut rgb,
    );
    assert_eq!(
        frame.actors.len(),
        1,
        "Cleric must reach production room pixels"
    );
    let area = Rect::new(0, 0, 160, 45);
    let mut terminal = Terminal::new(TestBackend::new(160, 45)).expect("test terminal");
    terminal
        .draw(|f| {
            flush_rgb(f.buffer_mut(), area, &rgb, Rgb::BLACK, ColorMode::Xterm256);
            render_scene_identity_labels(f, &m, &frame);
            if card {
                render_scene_overlays(f, &m, &presentation, None);
            }
        })
        .expect("draw production review");
    json!({"width":160,"height":90,"pixels":rgb.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>(),"bounds":[frame.actors[0].bounds.x,frame.actors[0].bounds.y,16,24],"columns":160,"rows":45,"cells":terminal.backend().buffer().content.iter().map(|c|json!({"text":c.symbol(),"fg":colour(c.fg),"bg":colour(c.bg)})).collect::<Vec<_>>(),"next_ms":frame.next_frame_in.map(|d|d.as_millis())})
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("review directory");
    let directory = Path::new(&directory);
    let p = persona();
    let mut frames = serde_json::Map::new();
    for (name, pose, index) in [
        ("work-a", ScenePose::Working, 0),
        ("work-b", ScenePose::Working, 1),
        ("signal", ScenePose::SeekingCounsel, 0),
        ("wait", ScenePose::SeekingCounsel, 1),
        ("spoils", ScenePose::ReturningWithSpoils, 0),
        ("settled", ScenePose::Settled, 0),
        ("rest", ScenePose::Resting, 0),
        ("unknown", ScenePose::Unknown, 0),
    ] {
        frames.insert(
            name.into(),
            sprite(&adventurer_animation_frame(&p, pose, index)),
        );
    }
    let variants=SkinTone::ALL.iter().map(|skin| {
        let mut p=p.clone();p.appearance.skin_tone=*skin;
        json!({"skin":format!("{skin:?}"),"frame":sprite(&adventurer_animation_frame(&p,ScenePose::Working,0))})
    }).collect::<Vec<_>>();
    let mut worlds = serde_json::Map::new();
    for (world, view) in [("hall", View::Guild), ("delve", View::Delve)] {
        for (name, presence, age) in [
            ("work-a", Presence::Working, 0),
            ("work-b", Presence::Working, 500),
            ("signal", Presence::Blocked, 0),
            ("wait", Presence::Blocked, 600),
            ("spoils", Presence::Done, 0),
            ("settled", Presence::Done, 3000),
            ("rest", Presence::Idle, 0),
            ("unknown", Presence::Unknown, 0),
        ] {
            worlds.insert(
                format!("{world}-{name}"),
                render(&model(view, presence, age), false),
            );
        }
    }
    let output = json!({"frames":frames,"variants":variants,"worlds":worlds,"card":render(&model(View::Guild,Presence::Working,0),true),"fallback":sprite(&adventurer_portrait_frame(&p).expect("Cleric fallback")),"roster":sprite(&adventurer_roster_frame(&p))});
    fs::write(
        directory.join("rendered.json"),
        serde_json::to_string(&output)?,
    )?;
    println!(
        "Exported actual production Cleric poses, two lit rooms, card fallback and six skin variants."
    );
    Ok(())
}
