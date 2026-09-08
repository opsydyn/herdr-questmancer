//! Review fixtures use production sprites, room rendering and Ratatui overlays.
use questmancer::{
    app::{CharacterSet, ColorMode, CounselPhase, Model, Motion, View},
    command::CommandResult,
    domain::{
        AccentTone, AdventurerClass, AdventurerPersona, AgentKey, GuildAttention, GuildSummons,
        PersonaKey, Presence, Timestamp,
    },
    interaction::reduce_action,
    runtime_loop::apply_command_result,
    scene::{
        assets::adventurer::{adventurer_animation_frame, adventurer_roster_frame},
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
        sprite::SpriteFrame,
        stage::ScenePose,
    },
    storybook::fixtures::{StoryContext, delve_world_fixture, guild_world_fixture},
    ui::{
        input::Action,
        scene_adapter::flush_rgb,
        scene_overlays::{render_scene_identity_labels, render_scene_overlays},
    },
};
use questmancer::{
    domain::WorkspaceId,
    scene::{
        render_scene_for_story,
        snapshot::{SceneAgent, SceneCampaign, SceneConnection},
        stage::WorldScene,
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
const CLASSES: [AdventurerClass; 3] = [
    AdventurerClass::Bard,
    AdventurerClass::Artificer,
    AdventurerClass::Testmender,
];
fn persona(index: usize) -> AdventurerPersona {
    let mut p = AdventurerPersona::for_key(PersonaKey::new(format!("pilot-{index}")));
    p.class = CLASSES[index % 3];
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
        agent.name = ["Aria", "Tobin", "Mira"][index % 3].to_owned();
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
fn counsel(outcome: &str, ascii: bool) -> Model {
    let mut m = party(View::Guild, Presence::Blocked, 1_000, Motion::None, 3);
    m.select_agent(&AgentKey::new("pilot-0"));
    if ascii {
        let mut p = *m.preferences();
        p.character_set = CharacterSet::Ascii;
        p.color_mode = ColorMode::Ansi16;
        m.set_preferences(p);
    }
    let _ = reduce_action(&mut m, Action::Counsel);
    for ch in "Please check the failing test before continuing.".chars() {
        m.push_counsel_character(ch);
    }
    if outcome == "draft" {
        return m;
    }
    let _ = reduce_action(&mut m, Action::Submit);
    let CounselPhase::Sending { request, pane_id } = m.counsel_phase().unwrap().clone() else {
        panic!("not sending");
    };
    if outcome == "sending" {
        return m;
    }
    if outcome == "late" {
        m.dismiss_modal();
    }
    let result = match outcome {
        "confirmed" | "late" => CommandResult::CounselSent { request, pane_id },
        "rejected" => CommandResult::CounselTextFailed {
            request,
            message: "Herdr rejected the text.".to_owned(),
        },
        "uncertain" => CommandResult::CounselTextUncertain {
            request,
            pane_id,
            message: "The acknowledgement did not arrive.".to_owned(),
        },
        _ => CommandResult::CounselSubmissionFailed {
            request,
            pane_id,
            message: "Text arrived; Enter was not confirmed.".to_owned(),
        },
    };
    let now = m.now();
    apply_command_result(&mut m, result, now);
    m
}
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
    let directory = env::args().nth(1).expect("review directory");
    let approved: Value = serde_json::from_str(
        &fs::read_to_string(
            std::path::Path::new(&directory)
                .join("../2026-09-08-tool-ritual-storyboard/candidates.json"),
        )
        .unwrap(),
    )
    .unwrap();
    let output = env::args().nth(2).expect("output JSON");
    let mut data = serde_json::Map::new();
    for index in 0..3 {
        let p = persona(index);
        for (label, pose, frame) in [
            ("work-a", ScenePose::Working, 0),
            ("work-b", ScenePose::Working, 1),
            ("counsel", ScenePose::SeekingCounsel, 0),
            ("waiting", ScenePose::SeekingCounsel, 1),
            ("spoils", ScenePose::ReturningWithSpoils, 0),
            ("settled", ScenePose::Settled, 0),
            ("rest", ScenePose::Resting, 0),
            ("unknown", ScenePose::Unknown, 0),
        ] {
            let actual = adventurer_animation_frame(&p, pose, frame);
            let source_name = match label {
                "work-a" => "working_a",
                "work-b" => "working_b",
                "counsel" => "counsel_raise",
                "waiting" => "counsel_wait",
                "rest" => "resting",
                other => other,
            };
            let source = &approved["classes"][index];
            let rows = source["frames"][source_name].as_array().unwrap();
            assert_eq!(actual.size(), PixelSize::new(16, 24));
            for (pixel, key) in actual
                .pixels()
                .iter()
                .zip(rows.iter().flat_map(|row| row.as_str().unwrap().chars()))
            {
                assert_eq!(
                    pixel.is_some(),
                    key != '.',
                    "approved silhouette {index} {label}"
                );
                if key != '.' && !"kKhrRla".contains(key) {
                    let hex = source["palette"][key.to_string()].as_str().unwrap();
                    let rgb = [0, 2, 4]
                        .map(|start| u8::from_str_radix(&hex[start..start + 2], 16).unwrap());
                    assert_eq!(
                        *pixel,
                        Some(Rgb::new(rgb[0], rgb[1], rgb[2])),
                        "approved material {index} {label} {key}"
                    );
                }
            }
            data.insert(format!("{index}-{label}"), frame_json(&actual));
        }
        data.insert(
            format!("{index}-roster"),
            frame_json(&adventurer_roster_frame(&p)),
        );
    }
    println!(
        "24 production poses match the approved silhouettes and class-owned material pixels; persona roles remain dynamic."
    );
    for index in 0..12 {
        data.insert(
            format!("variant-{index}"),
            frame_json(&adventurer_animation_frame(
                &persona(index),
                ScenePose::Working,
                0,
            )),
        );
    }
    for (world, view) in [("hall", View::Guild), ("delve", View::Delve)] {
        for (label, presence, age) in [
            ("work-a", Presence::Working, 0),
            ("work-b", Presence::Working, 500),
            ("counsel", Presence::Blocked, 0),
            ("waiting", Presence::Blocked, 600),
            ("spoils", Presence::Done, 0),
            ("placed", Presence::Done, 1_000),
            ("settled", Presence::Done, 3_000),
            ("rest", Presence::Idle, 0),
            ("unknown", Presence::Unknown, 0),
            ("exited", Presence::Exited, 0),
        ] {
            let m = party(view, presence, age, Motion::Full, 3);
            data.insert(
                format!("{world}-{label}"),
                render(&m, PixelSize::new(160, 90), ColorMode::Xterm256, false),
            );
        }
        for frame in 0..25 {
            let m = party(view, Presence::Done, frame * 125, Motion::Full, 3);
            data.insert(
                format!("{world}-return-{frame}"),
                render(&m, PixelSize::new(160, 90), ColorMode::Xterm256, false),
            );
        }
        for (label, width, height, count) in [
            ("canonical", 160, 90, 3),
            ("medium", 100, 60, 3),
            ("compact", 64, 40, 3),
            ("roster", 30, 30, 3),
            ("vignette", 24, 26, 3),
            ("status", 16, 18, 3),
            ("capacity", 160, 90, 11),
            ("overflow", 160, 90, 12),
        ] {
            let mut m = party(view, Presence::Working, 0, Motion::None, count);
            for (index, a) in m.domain_mut().agents.values_mut().enumerate() {
                a.presence = [
                    Presence::Working,
                    Presence::Blocked,
                    Presence::Done,
                    Presence::Idle,
                    Presence::Unknown,
                ][index % 5];
                a.attention = GuildAttention::Clear;
            }
            m.select_agent(&AgentKey::new("pilot-1"));
            data.insert(
                format!("{world}-{label}"),
                render(&m, PixelSize::new(width, height), ColorMode::Xterm256, true),
            );
            let mut p = *m.preferences();
            p.character_set = CharacterSet::Ascii;
            p.color_mode = ColorMode::Ansi16;
            m.set_preferences(p);
            data.insert(
                format!("{world}-{label}-ansi"),
                render(&m, PixelSize::new(width, height), ColorMode::Ansi16, true),
            );
        }
    }
    for label in [
        "draft",
        "sending",
        "confirmed",
        "rejected",
        "uncertain",
        "submit",
        "late",
    ] {
        for ascii in [false, true] {
            let m = counsel(label, ascii);
            let mode = m.preferences().color_mode;
            data.insert(
                format!("counsel-{label}{}", if ascii { "-ascii" } else { "" }),
                render(&m, PixelSize::new(100, 52), mode, true),
            );
        }
    }
    for (world, view) in [("hall", View::Guild), ("delve", View::Delve)] {
        let mut times = vec![
            ("work-a".to_owned(), Presence::Working, 0),
            ("work-b".to_owned(), Presence::Working, 500),
            ("counsel".to_owned(), Presence::Blocked, 0),
            ("waiting".to_owned(), Presence::Blocked, 600),
            ("spoils".to_owned(), Presence::Done, 0),
            ("settled".to_owned(), Presence::Done, 3000),
        ];
        times.extend((0..25).map(|n| (format!("return-{n}"), Presence::Done, n * 125)));
        for (label, presence, age) in times {
            for (index, class) in CLASSES.into_iter().enumerate() {
                let mut m = party(view, presence, age, Motion::Full, 1);
                m.domain_mut().agents.values_mut().next().unwrap().persona = persona(index);
                assert_eq!(
                    m.domain().agents.values().next().unwrap().persona.class,
                    class
                );
                data.insert(
                    format!("solo-{world}-{label}-{index}"),
                    render(&m, PixelSize::new(160, 90), ColorMode::Xterm256, false),
                );
            }
        }
    }
    for (index, class) in CLASSES.into_iter().enumerate() {
        let mut m = party(View::Guild, Presence::Blocked, 1_000, Motion::None, 1);
        m.domain_mut()
            .agents
            .values_mut()
            .next()
            .unwrap()
            .persona
            .class = class;
        m.select_agent(&AgentKey::new("pilot-0"));
        m.show_adventurer_card();
        data.insert(
            format!("card-{index}"),
            render(&m, PixelSize::new(100, 60), ColorMode::Xterm256, true),
        );
    }
    // This is the existing quiet Delve story whose production golden changed.
    let mut golden = delve_world_fixture(StoryContext::fixed());
    let mut p = *golden.preferences();
    p.motion = Motion::None;
    golden.set_preferences(p);
    data.insert(
        "delve-story".into(),
        render(&golden, PixelSize::new(160, 90), ColorMode::Xterm256, true),
    );
    let mut exact = RgbBuffer::filled(0, 0, Rgb::BLACK);
    render_scene_for_story(
        &mixed_snapshot(),
        Some(WorldScene::Delve),
        PixelSize::new(160, 90),
        &mut exact,
    );
    data.insert("delve-golden".into(),json!({"width":160,"height":90,"pixels":exact.pixels().iter().map(|p|[p.r,p.g,p.b]).collect::<Vec<_>>()}));
    fs::write(output, serde_json::to_string(&data).unwrap()).unwrap();
    println!("Exported production pilot sprites, scene times, viewports and counsel outcomes.");
}
