//! Review-only reproduction of the resting-party layout policies.
use image::{DynamicImage, RgbImage, imageops::FilterType};
use questmancer::{
    app::{Model, View},
    domain::{GuildAttention, Presence},
    scene::{
        fit::{CellPixels, FitFailure, FitScene},
        pixel::{PixelSize, Rgb, RgbBuffer},
        presentation::ScenePresentation,
        render_scene_for_world,
        snapshot::SceneSnapshot,
    },
    storybook::fixtures::{FitParty, StoryContext, hall_fit_fixture},
};
use ratatui::layout::Rect;
use serde_json::json;
use std::{env, fs, path::Path};

fn fixture(count: usize, resting: bool) -> Model {
    let mut model = hall_fit_fixture(StoryContext::fixed(), FitParty::Eleven);
    let keys = model
        .domain()
        .agents
        .keys()
        .take(count)
        .cloned()
        .collect::<Vec<_>>();
    model
        .domain_mut()
        .agents
        .retain(|key, _| keys.contains(key));
    for campaign in model.domain_mut().campaigns.values_mut() {
        campaign.party.retain(|key| keys.contains(key));
    }
    if resting {
        for (index, agent) in model.domain_mut().agents.values_mut().enumerate() {
            agent.presence = if index == count - 2 {
                Presence::Unknown
            } else {
                Presence::Idle
            };
            agent.attention = GuildAttention::Clear;
        }
    }
    model
}

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let directory = env::args().nth(1).expect("review directory");
    let directory = Path::new(&directory);
    let mut measurements = Vec::new();
    for (world, view) in [("hall", View::Guild), ("delve", View::Delve)] {
        for (count, resting) in [(5, true), (7, true), (7, false), (11, true), (11, false)] {
            let name = format!(
                "{world}-{count}-{}",
                if resting { "resting-unknown" } else { "mixed" }
            );
            let mut model = fixture(count, resting);
            model.switch_to(view);
            let snapshot = SceneSnapshot::from_model(&model);
            let presentation = ScenePresentation::from_model(&model);
            for (width, height) in [(206, 50), (120, 34), (96, 28), (80, 24)] {
                let fit = FitScene::project(
                    &model,
                    Rect::new(0, 1, width, height - 3),
                    CellPixels::new(8, 16).expect("fixed cell"),
                );
                let capacity = match &fit {
                    Ok(scene) => Some(scene.frame.actors.len()),
                    Err(FitFailure::Incomplete { visible, .. }) => Some(*visible),
                    Err(_) => None,
                };
                let failure = fit.as_ref().err().map(ToString::to_string);
                let mut pixels = RgbBuffer::filled(0, 0, Rgb::BLACK);
                let (image, frame) = match fit {
                    Ok(scene) => {
                        let transform = scene.transform;
                        let rendered = transform.image(&scene.rgb).to_rgb8();
                        let mut canvas =
                            RgbImage::new(u32::from(width) * 8, u32::from(height) * 16);
                        image::imageops::replace(
                            &mut canvas,
                            &rendered,
                            i64::from(transform.area.x) * 8,
                            i64::from(transform.area.y) * 16,
                        );
                        (DynamicImage::ImageRgb8(canvas), scene.frame)
                    }
                    Err(_) => {
                        let frame = render_scene_for_world(
                            &snapshot,
                            &presentation,
                            PixelSize::new(width, height * 2),
                            &mut pixels,
                        );
                        let rgb =
                            RgbImage::from_fn(u32::from(width), u32::from(height) * 2, |x, y| {
                                let c = pixels
                                    .get(
                                        i32::try_from(x).expect("bounded x"),
                                        i32::try_from(y).expect("bounded y"),
                                    )
                                    .expect("render pixel");
                                image::Rgb([c.r, c.g, c.b])
                            });
                        (
                            DynamicImage::ImageRgb8(rgb).resize_exact(
                                u32::from(width) * 8,
                                u32::from(height) * 16,
                                FilterType::Nearest,
                            ),
                            frame,
                        )
                    }
                };
                let filename = format!("{name}-{width}x{height}.png");
                image.save(directory.join(&filename))?;
                let mut actor_rows = frame.actors.iter().map(|a| a.bounds.y).collect::<Vec<_>>();
                actor_rows.sort_unstable();
                actor_rows.dedup();
                measurements.push(json!({
                    "world": world, "party": count, "distribution": if resting { "all resting except one unknown" } else { "mixed source fixture" },
                    "viewport_cells": [width, height], "cell_pixels": [8, 16], "canonical_capacity": capacity,
                    "fit_failure": failure, "native_fit": failure.is_none(), "visible_actors": frame.actors.len(),
                    "actor_top_rows": actor_rows, "actor_bounds": frame.actors.iter().map(|a| json!([a.bounds.x, a.bounds.y, a.bounds.width, a.bounds.height])).collect::<Vec<_>>(),
                    "filename": filename,
                }));
            }
        }
    }
    fs::write(
        directory.join("measurements.json"),
        serde_json::to_string_pretty(&measurements)? + "\n",
    )?;
    Ok(())
}
