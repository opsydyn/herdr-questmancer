//! Source render exports for the reported resting-crowd regression.
use questmancer::{
    app::{Model,View},
    scene::fit::{CellPixels,FitScene},
    storybook::fixtures::{StoryContext,FitParty,delve_fit_fixture,delve_resting_fit_fixture},
};
use ratatui::layout::Rect;
use std::{env,path::Path};
fn main()->Result<(),Box<dyn std::error::Error>> {
    let directory=env::args().nth(1).expect("review directory");let directory=Path::new(&directory);
    for (name,model) in [("resting-five",delve_resting_fit_fixture(StoryContext::fixed())),
                         ("working-eleven",delve_fit_fixture(StoryContext::fixed(),FitParty::Crowded)),
                         ("ordinary-five",delve_fit_fixture(StoryContext::fixed(),FitParty::Five)),
                         ("empty",Model::new(View::Delve))] {
        for (w,h) in [(120,34),(96,28),(80,24)] {
            let scene=FitScene::project(&model,Rect::new(0,1,w,h-3),CellPixels::new(8,16).expect("cell pixels"))?;
            scene.transform.image(&scene.rgb).save(directory.join(format!("{name}-{w}x{h}.png")))?;
        }
    }
    Ok(())
}
