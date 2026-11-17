//! Export the same empty offline Hall projected by the production PTY probe.
use questmancer::{app::{Model, View}, scene::fit::{CellPixels,FitScene}};
use ratatui::layout::Rect;
use std::{env,path::Path};
fn main() -> Result<(),Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("review directory");
    let model = Model::new(View::Guild);
    let scene = FitScene::project(&model,Rect::new(0,1,120,31),CellPixels::new(8,16).expect("cell geometry"))?;
    scene.transform.image(&scene.rgb).save(Path::new(&directory).join("expected-empty-hall.png"))?;
    Ok(())
}
