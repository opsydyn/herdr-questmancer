use questmancer::{storybook::fixtures::{StoryContext,OwnedAuthoringState,owned_authoring_fixture},scene::{pixel::{PixelSize,Rgb,RgbBuffer},snapshot::SceneSnapshot,presentation::ScenePresentation,render_scene_for_world},ui::{scene_adapter::flush_rgb,scene_overlays::{render_scene_identity_labels,render_scene_overlays}}};
use ratatui::{Terminal,backend::TestBackend};
fn main() {
    let target=std::env::args().nth(1).expect("output directory");
    for (name,variant,w,h) in [("actions",OwnedAuthoringState::Actions,120,36),("empty-campaign",OwnedAuthoringState::EmptyCampaign,120,36),("answer",OwnedAuthoringState::Answer,120,36),("answer-narrow",OwnedAuthoringState::Narrow,40,20),("answer-narrow-editing",OwnedAuthoringState::Narrow,40,20),("view-pending",OwnedAuthoringState::ViewPending,120,36),("indeterminate",OwnedAuthoringState::Indeterminate,120,36),("answered",OwnedAuthoringState::Answered,120,36),("below-minimum",OwnedAuthoringState::Small,27,11)] {
        let mut model=owned_authoring_fixture(StoryContext::fixed(),variant);model.set_work_viewport_size(w,h);if name=="answer-narrow-editing" {model.authoring_mut().follow_focus=true;}
        let mut terminal=Terminal::new(TestBackend::new(w,h)).unwrap();
        terminal.draw(|frame| {
            let area=frame.area();let mut rgb=RgbBuffer::filled(0,0,Rgb::BLACK);
            let snapshot=SceneSnapshot::from_model(&model);let presentation=ScenePresentation::from_model(&model);
            let scene_frame=render_scene_for_world(&snapshot,&presentation,PixelSize::new(area.width,area.height*2),&mut rgb);
            flush_rgb(frame.buffer_mut(),area,&rgb,Rgb::BLACK,model.preferences().color_mode);
            render_scene_identity_labels(frame,&model,&scene_frame);
            render_scene_overlays(frame,&model,&presentation,None);
        }).unwrap();
        let buffer=terminal.backend().buffer();let mut cells=Vec::new();
        for y in 0..h {for x in 0..w {let cell=&buffer[(x,y)];cells.push(serde_json::json!({"x":x,"y":y,"text":cell.symbol(),"fg":format!("{:?}",cell.fg),"bg":format!("{:?}",cell.bg)}));}}
        std::fs::write(format!("{target}/{name}.json"),serde_json::to_vec(&serde_json::json!({"width":w,"height":h,"cells":cells})).unwrap()).unwrap();
    }
}
