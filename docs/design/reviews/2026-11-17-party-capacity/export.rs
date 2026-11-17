//! Fixed experimental capacity renders; no Herdr or live state.
use questmancer::{app::View,storybook::{density::{self,PartySize,StudyScale},fixtures::StoryContext},scene::fit::CellPixels};
use ratatui::layout::Rect;
use std::{env,path::Path,fs};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let directory=env::args().nth(1).expect("review directory");let directory=Path::new(&directory);let mut rows=Vec::new();
 for (room,view) in [("hall",View::Guild),("delve",View::Delve)] {
  for (count,size,scale) in [(7,PartySize::Seven,StudyScale::Large),(7,PartySize::Seven,StudyScale::Small),(24,PartySize::TwentyFour,StudyScale::Small),(32,PartySize::ThirtyTwo,StudyScale::Small)] {
   let model=density::fixture(StoryContext::fixed(),size,view);
   for (width,height) in [(120,34),(96,28),(80,24)] {
    let scene=density::project(&model,Rect::new(0,1,width,height-3),CellPixels::new(8,16).expect("fixed cell"),scale)?;
    let name=format!("{room}-{count}-{scale:?}-{width}x{height}.png");scene.transform.image(&scene.rgb).save(directory.join(&name))?;
    rows.push(serde_json::json!({"room":room,"party":count,"scale":format!("{scale:?}"),"viewport_cells":[width,height],"visible_actors":scene.frame.actors.len(),"sprite_size":[scene.frame.actors[0].bounds.width,scene.frame.actors[0].bounds.height],"filename":name,"experimental":true}));
   }
  }
 }
 for (left,right,output) in [("hall-7-Large-120x34.png","hall-7-Small-120x34.png","01-hall-seven-comparison.png"),("delve-7-Large-120x34.png","delve-7-Small-120x34.png","02-delve-seven-comparison.png"),("hall-24-Small-120x34.png","delve-24-Small-120x34.png","03-twenty-four-both-rooms.png")] {
  let left=image::open(directory.join(left))?.to_rgb8();let right=image::open(directory.join(right))?.to_rgb8();
  let mut sheet=image::RgbImage::new(left.width()+right.width()+8,left.height());
  image::imageops::replace(&mut sheet,&left,0,0);image::imageops::replace(&mut sheet,&right,i64::from(left.width()+8),0);sheet.save(directory.join(output))?;
 }
 fs::write(directory.join("measurements.json"),serde_json::to_string_pretty(&rows)?+"\n")?;Ok(())
}
