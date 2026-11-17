//! Source render exports for fitted worlds beneath profiles.
use questmancer::{scene::fit::{CellPixels,FitScene},storybook::fixtures::{StoryContext,FitParty,hall_fit_fixture,delve_fit_fixture}};
use ratatui::layout::Rect;
use std::{env,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let directory=env::args().nth(1).expect("review directory");let directory=Path::new(&directory);
 for (name,mut model) in [("guild",hall_fit_fixture(StoryContext::fixed(),FitParty::Five)),("delve",delve_fit_fixture(StoryContext::fixed(),FitParty::Five))] {
  model.show_adventurer_card();
  let scene=FitScene::project(&model,Rect::new(0,1,120,31),CellPixels::new(8,16).expect("cell pixels"))?;
  scene.transform.image(&scene.rgb).save(directory.join(format!("{name}-profile-120x34.png")))?;
 }
 Ok(())
}
