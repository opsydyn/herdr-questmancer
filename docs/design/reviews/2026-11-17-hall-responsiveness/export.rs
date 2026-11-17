use questmancer::{
    app::Motion,
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, AgentKey, PersonaKey, Presence, Timestamp,
        WorkspaceId,
    },
    scene::{
        pixel::{PixelSize, Rgb, RgbBuffer},
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection, SceneSnapshot},
        stage::WorldScene,
    },
};
use std::{env, fs};
fn campaign(id: &str, seed: u64) -> SceneCampaign {
    SceneCampaign {
        workspace_id: WorkspaceId::new(id),
        label: id.replace('-', " "),
        variant_seed: seed,
    }
}

fn agent(key: &str, workspace: &str, presence: Presence, accent: AccentTone) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("delve-{key}")));
    persona.appearance.accent = accent;
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: WorkspaceId::new(workspace),
        name: key.replace('-', " "),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(1_000),
        transition: None,
        focused: false,
        persona,
    }
}

fn snapshot(count: usize) -> SceneSnapshot {
    let identities = [
        ("forge", AdventurerClass::Druid, Presence::Working),
        ("review", AdventurerClass::Bard, Presence::Blocked),
        ("cleric", AdventurerClass::Cleric, Presence::Working),
        ("rest", AdventurerClass::Sorcerer, Presence::Idle),
        ("mist", AdventurerClass::Sorcerer, Presence::Unknown),
    ];
    let agents = (0..count)
        .map(|i| {
            let (label, class, presence) = identities[i % identities.len()];
            let mut a = agent(
                &format!("{label}-{i}"),
                "guild-review",
                presence,
                AccentTone::Cyan,
            );
            a.persona.class = class;
            a
        })
        .collect();
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![campaign("guild-review", 7)],
        agents,
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    }
}
fn main() -> Result<(), Box<dyn std::error::Error>> {
    let path = env::args().nth(1).expect("output path");
    let mut renders = Vec::new();
    for count in [5, 11] {
        for (cols, rows) in [
            (180, 50),
            (160, 45),
            (160, 44),
            (120, 34),
            (96, 28),
            (80, 24),
            (60, 20),
            (40, 18),
            (20, 14),
        ] {
            let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
            let frame = render_scene_for_story(
                &snapshot(count),
                Some(WorldScene::GuildHall),
                PixelSize::new(cols, rows * 2),
                &mut rgb,
            );
            renders.push(serde_json::json!({"party":count,"columns":cols,"rows":rows,
                "width":rgb.size().width,"height":rgb.size().height,
                "pixels":rgb.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>(),
                "actors":frame.actors.iter().map(|a|serde_json::json!({"key":a.agent.as_str(),"x":a.bounds.x,"y":a.bounds.y,"width":a.bounds.width,"height":a.bounds.height})).collect::<Vec<_>>(),
                "interactables":frame.interactables.len(),"next_frame":frame.next_frame_in.map(|d|d.as_millis())}));
        }
    }
    fs::write(path, serde_json::to_string(&renders)?)?;
    println!(
        "Exported 18 current production Hall compositions at fixed sizes; no native transport or live operations."
    );
    Ok(())
}
