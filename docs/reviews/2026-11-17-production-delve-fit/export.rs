//! Deterministic source projections; these are not native terminal screenshots.
use questmancer::{
    app::{Model, View},
    scene::{fit::{CellPixels,FitScene}, pixel::{PixelSize,Rgb,RgbBuffer},
        presentation::ScenePresentation, snapshot::SceneSnapshot, render_scene_for_world},
    storybook::fixtures::{FitParty,StoryContext,delve_fit_fixture},
};
use ratatui::layout::Rect;
use std::{env,path::Path};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let directory=env::args().nth(1).expect("review directory");let directory=Path::new(&directory);
    let empty=Model::new(View::Delve);
    let scene=FitScene::project(&empty,Rect::new(0,1,120,31),CellPixels::new(8,16).expect("cell pixels"))?;
    scene.transform.image(&scene.rgb).save(directory.join("expected-empty-delve.png"))?;
    let model=delve_fit_fixture(StoryContext::fixed(),FitParty::Five);
    for (w,h) in [(120,34),(96,28),(80,24)] {
        let scene=FitScene::project(&model,Rect::new(0,1,w,h-3),CellPixels::new(8,16).expect("cell pixels"))?;
        scene.transform.image(&scene.rgb).save(directory.join(format!("fit-five-{w}x{h}.png")))?;
    }
    let crowded=delve_fit_fixture(StoryContext::fixed(),FitParty::Crowded);
    let mut rgb=RgbBuffer::filled(0,0,Rgb::BLACK);
    let frame=render_scene_for_world(&SceneSnapshot::from_model(&crowded),&ScenePresentation::from_model(&crowded),PixelSize::new(96,56),&mut rgb);
    assert_eq!(frame.actors.len(),11);
    let img=image::RgbImage::from_fn(96,56,|x,y| {
        let p=rgb.get(i32::try_from(x).expect("bounded x"),i32::try_from(y).expect("bounded y")).expect("source pixel");
        image::Rgb([p.r,p.g,p.b])
    });
    image::DynamicImage::ImageRgb8(img).resize_exact(768,448,image::imageops::FilterType::Nearest)
        .save(directory.join("crowded-roster-source.png"))?;
    Ok(())
}
