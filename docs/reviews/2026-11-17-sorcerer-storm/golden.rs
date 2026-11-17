use questmancer::{
    app::Motion,
    domain::{
        AccentTone, AdventurerPersona, AgentKey, PersonaKey, Presence, Timestamp, WorkspaceId,
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

fn mixed_snapshot() -> SceneSnapshot {
    SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![
            campaign("moss-vault", 0x47a1),
            campaign("rune-road", 0xb20f),
        ],
        agents: vec![
            agent("working", "moss-vault", Presence::Working, AccentTone::Cyan),
            agent(
                "blocked",
                "moss-vault",
                Presence::Blocked,
                AccentTone::Magenta,
            ),
            agent("done", "rune-road", Presence::Done, AccentTone::Amber),
            agent("idle", "rune-road", Presence::Idle, AccentTone::Lime),
            agent(
                "unknown",
                "moss-vault",
                Presence::Unknown,
                AccentTone::Violet,
            ),
            agent("exited", "rune-road", Presence::Exited, AccentTone::Red),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    }
}

fn main() {
    let mut rgb = RgbBuffer::filled(0, 0, Rgb::BLACK);
    let snapshot = mixed_snapshot();
    render_scene_for_story(
        &snapshot,
        Some(WorldScene::Delve),
        PixelSize::new(160, 90),
        &mut rgb,
    );
    fs::write(env::args().nth(1).expect("output"),serde_json::to_string(&serde_json::json!({"width":160,"height":90,"pixels":rgb.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>() })).expect("serialize fixture")).expect("write fixture");
}
