use questmancer::{
    app::{ColorMode, Model}, domain::{Chronicle, ChronicleEntry, ChronicleEvent, Timestamp},
    interaction::reduce_action,
    scene::{pixel::{PixelSize, Rgb, RgbBuffer}, presentation::ScenePresentation, render_scene_for_world, snapshot::SceneSnapshot},
    storybook::fixtures::{StoryContext, guild_world_fixture},
    ui::{input::Action, scene_adapter::flush_rgb, scene_overlays::{render_scene_identity_labels, render_scene_overlays}},
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
    let mut configured = model.clone();
    let mut preferences = *configured.preferences();
    preferences.color_mode = mode;
    configured.set_preferences(preferences);
    let model = &configured;
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

fn main() {
    let mut model=guild_world_fixture(StoryContext::fixed());
    let end=1788951600000_i64; // 2026-09-09 11:00 UTC; fixed fixture time.
    model.set_now(Timestamp::from_millis(end));
    let mut preferences=*model.preferences();
    preferences.motion=questmancer::app::Motion::None;
    model.set_preferences(preferences);
    for agent in model.domain_mut().agents.values_mut() {
        agent.presence_since=Timestamp::from_millis(end-30_000);
        agent.attention=questmancer::domain::GuildAttention::Clear;
    }
    let key=model.selected_agent_key().cloned().unwrap();
    let second=model.domain().agents.keys().find(|candidate| **candidate != key).unwrap().clone();
    for (identity,name) in [(&key,"Ari"),(&second,"Bram")] {
        let agent=model.domain_mut().agents.get_mut(identity).unwrap();
        agent.name=name.to_owned();
        agent.persona.name=name.to_owned();
    }
    let records=[
        (key.clone(),ChronicleEvent::SpoilsReturned,"Ari returned with spoils",end-1_800_000),
        (key.clone(),ChronicleEvent::SpoilsReturned,"Ari returned with spoils",end-1_200_000),
        (key.clone(),ChronicleEvent::SpoilsReturned,"Ari returned with spoils",end-600_000),
        (second,ChronicleEvent::CounselRequested,"Bram requested counsel",end-300_000),
    ];
    model.domain_mut().chronicle=Chronicle::default();
    for (revision,(agent,event,summary,at)) in records.into_iter().enumerate() {
        model.domain_mut().chronicle.append(ChronicleEntry::new(Timestamp::from_millis(at),Some(agent),None,None,revision as u64,event,summary));
    }
    use questmancer::domain::{CapturedObservation, ObservationStamp, CaptureRunId, ObservationSubject, ObservationEvidence, SnapshotObservation, ObservedPresence, RemovalEvidence, WorkspaceId, PaneId};
    use questmancer::snapshot_refresh::{ConnectionEpoch, SnapshotRequest, SnapshotRequestId, LiveFactGeneration};
    let key=model.selected_agent_key().cloned().unwrap();
    let kinds = [
        SnapshotObservation::AdventurerObserved { presence: ObservedPresence::Working },
        SnapshotObservation::PresenceObserved { presence: ObservedPresence::Blocked },
        SnapshotObservation::PresenceObserved { presence: ObservedPresence::Unknown },
        SnapshotObservation::AdventurerNoLongerVisible,
        SnapshotObservation::CampaignRemoved { evidence: RemovalEvidence::CloseHintConfirmed },
    ];
    for (i,kind) in kinds.into_iter().enumerate() {
        let ordinal = i as u64+1;
        let subject=if matches!(kind,SnapshotObservation::CampaignRemoved { .. }) {
            ObservationSubject::Campaign { id:WorkspaceId::new("w-review"), name:"Lantern archive".into() }
        } else {
            ObservationSubject::Adventurer { key:key.clone(),campaign:WorkspaceId::new("w-review"),pane:PaneId::new("w-review:p1"),incarnation:"review-incarnation".into(),revision:ordinal,name:"Ari".into() }
        };
        let observation=CapturedObservation {
            stamp: ObservationStamp { run:CaptureRunId::new("visual-review"),epoch:ConnectionEpoch(1),ordinal },
            subject,
            evidence:ObservationEvidence::Snapshot { request:SnapshotRequest { epoch:ConnectionEpoch(1),id:SnapshotRequestId(ordinal),generation:LiveFactGeneration(ordinal) },observation:kind },
        };
        let record=ChronicleEntry::observed(Timestamp::from_millis(end-240_000+i as i64*30_000),observation).unwrap();
        model.domain_mut().chronicle.append(record);
    }
    let _=reduce_action(&mut model,Action::OpenChronicle);
    let records=render(&model,PixelSize::new(120,72),ColorMode::Xterm256,true);
    let _=reduce_action(&mut model,Action::ToggleChronicleChapter);
    let chapter=render(&model,PixelSize::new(120,72),ColorMode::Xterm256,true);
    for _ in 0..10 { let _=reduce_action(&mut model,Action::ScrollDown); }
    let sources=render(&model,PixelSize::new(120,72),ColorMode::Xterm256,true);
    for _ in 0..10 { let _=reduce_action(&mut model,Action::ScrollUp); }
    let compact=render(&model,PixelSize::new(80,48),ColorMode::Xterm256,true);
    let narrow=render(&model,PixelSize::new(40,36),ColorMode::Xterm256,true);
    let _=reduce_action(&mut model,Action::ToggleChronicleChapter);
    model.set_now(Timestamp::from_millis(end+7_200_000));
    let _=reduce_action(&mut model,Action::ToggleChronicleChapter);
    let empty=render(&model,PixelSize::new(120,72),ColorMode::Xterm256,true);
    fs::write(env::args().nth(1).expect("output path"),serde_json::to_vec(&json!({"records":records,"chapter":chapter,"sources":sources,"compact":compact,"narrow":narrow,"empty":empty})).unwrap()).unwrap();
}
