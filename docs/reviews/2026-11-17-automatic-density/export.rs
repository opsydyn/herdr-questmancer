//! Production fitted-room exports at the size-tier boundaries.
use questmancer::{app::View,domain::{AgentKey,GuildAttention,Presence},scene::fit::{CellPixels,FitScene},storybook::{density::{self,PartySize},fixtures::StoryContext}};
use ratatui::layout::Rect;
use std::{env,fs,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>> {
 let directory=env::args().nth(1).expect("review directory");let directory=Path::new(&directory);let mut rows=Vec::new();
 for (room,view) in [("guild",View::Guild),("delve",View::Delve)] {
  for count in [7,11,12,24,32,33,64] {
   let mut model=density::fixture(StoryContext::fixed(),PartySize::ThirtyTwo,view);
   let seeds=model.domain().agents.values().cloned().collect::<Vec<_>>();
   model.domain_mut().agents.clear();
   for i in 0..count {
    let mut agent=seeds[i%seeds.len()].clone();agent.key=AgentKey::new(format!("capacity-{i:02}"));
    if count<=11 {agent.presence=if i==5{Presence::Unknown}else{Presence::Idle};agent.attention=GuildAttention::Clear;}
    model.domain_mut().agents.insert(agent.key.clone(),agent);
   }
   for (width,height) in [(120,34),(96,28),(80,24)] {
    let scene=FitScene::project(&model,Rect::new(0,1,width,height-3),CellPixels::new(8,16).expect("fixed cell"))?;
    let name=format!("{room}-{count}-{width}x{height}.png");scene.transform.image(&scene.rgb).save(directory.join(&name))?;
    rows.push(serde_json::json!({"room":room,"party":count,"viewport_cells":[width,height],"visible_actors":scene.frame.actors.len(),"sprite_size":[scene.frame.actors[0].bounds.width,scene.frame.actors[0].bounds.height],"librarian_targets":scene.frame.interactables.len(),"filename":name}));
   }
  }
 }
 fs::write(directory.join("measurements.json"),serde_json::to_string_pretty(&rows)?+"\n")?;Ok(())
}
