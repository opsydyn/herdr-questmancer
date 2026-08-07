//! Temporary art-direction preview. Deleted before commit.
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

fn agent(key: &str, ws: &str, presence: Presence, accent: AccentTone) -> SceneAgent {
    let mut persona = AdventurerPersona::for_key(PersonaKey::new(format!("delve-{key}")));
    persona.appearance.accent = accent;
    SceneAgent {
        key: AgentKey::new(key),
        workspace_id: WorkspaceId::new(ws),
        name: key.replace('-', " "),
        custom_status: None,
        presence,
        presence_since: Timestamp::from_millis(1_000),
        transition: None,
        focused: false,
        persona,
    }
}

fn save(buffer: &RgbBuffer, path: &str, scale: u32) {
    let (w, h) = (
        u32::from(buffer.size().width),
        u32::from(buffer.size().height),
    );
    let mut img = image::RgbImage::new(w * scale, h * scale);
    for y in 0..h * scale {
        for x in 0..w * scale {
            let px = buffer
                .get(
                    i32::try_from(x / scale).unwrap(),
                    i32::try_from(y / scale).unwrap(),
                )
                .unwrap_or(Rgb::BLACK);
            img.put_pixel(x, y, image::Rgb([px.r, px.g, px.b]));
        }
    }
    img.save(path).unwrap();
    println!("png: {path}");
}

fn main() {
    let snapshot = SceneSnapshot {
        connection: SceneConnection::Connected,
        campaigns: vec![
            SceneCampaign {
                workspace_id: WorkspaceId::new("amber-library"),
                label: "amber library".into(),
                variant_seed: 7,
            },
            SceneCampaign {
                workspace_id: WorkspaceId::new("moss-vault"),
                label: "moss vault".into(),
                variant_seed: 29,
            },
        ],
        agents: vec![
            agent(
                "working-one",
                "amber-library",
                Presence::Working,
                AccentTone::Cyan,
            ),
            agent(
                "working-two",
                "amber-library",
                Presence::Working,
                AccentTone::Lime,
            ),
            agent(
                "blocked-one",
                "moss-vault",
                Presence::Blocked,
                AccentTone::Magenta,
            ),
            agent("done-one", "moss-vault", Presence::Done, AccentTone::Blue),
            agent(
                "idle-one",
                "amber-library",
                Presence::Idle,
                AccentTone::Teal,
            ),
            agent(
                "unknown-one",
                "amber-library",
                Presence::Unknown,
                AccentTone::Amber,
            ),
            agent(
                "unknown-two",
                "moss-vault",
                Presence::Unknown,
                AccentTone::Violet,
            ),
        ],
        motion: Motion::None,
        now: Timestamp::from_millis(10_000),
    };
    let mut target = RgbBuffer::filled(0, 0, Rgb::BLACK);
    render_scene_for_story(
        &snapshot,
        Some(WorldScene::Delve),
        PixelSize::new(160, 90),
        &mut target,
    );
    save(
        &target,
        "/private/tmp/claude-501/-Users-alancurrie-Projects-herdr-web-master/5e962b57-1cb9-4ad2-bfd7-ea9b2d024024/scratchpad/delve-before.png",
        6,
    );
}
