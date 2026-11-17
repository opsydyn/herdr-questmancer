//! Export actual Storybook fit images; no terminal or Herdr access.
use questmancer::storybook::{
    fit::{CellPixels, FitScene},
    fixtures::{FitParty, StoryContext, hall_fit_fixture},
};
use ratatui::layout::Rect;
use std::{env, fs, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("review directory");
    let directory = Path::new(&directory);
    let mut receipts = Vec::new();
    for (name, party) in [("five", FitParty::Five), ("eleven", FitParty::Eleven)] {
        let model = hall_fit_fixture(StoryContext::fixed(), party);
        for (columns, rows) in [(120, 34), (96, 28), (80, 24)] {
            let scene = FitScene::project(
                &model,
                Rect::new(0, 1, columns, rows - 3),
                CellPixels::new(8, 16).expect("fixture geometry"),
            )?;
            let filename = format!("fit-{name}-{columns}x{rows}.png");
            scene
                .transform
                .image(&scene.rgb)
                .save(directory.join(&filename))?;
            receipts.push(serde_json::json!({"file":filename,"columns":columns,"rows":rows,
                "scale":scene.transform.scale,"image_cells":[scene.transform.area.x,scene.transform.area.y,scene.transform.area.width,scene.transform.area.height],
                "actors":scene.frame.actors.len(),"librarian":scene.frame.interactables.len()}));
        }
    }
    fs::write(
        directory.join("renders.json"),
        serde_json::to_string_pretty(&receipts)?,
    )?;
    Ok(())
}
