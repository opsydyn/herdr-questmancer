//! Normalize the approved generated card for the existing embedded asset contract.
use std::{env, path::Path};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let arguments = env::args().collect::<Vec<_>>();
    let source = image::open(Path::new(&arguments[1]))?;
    assert_eq!((source.width(), source.height()), (1086, 1448));
    assert!(source.color().has_alpha(), "retain generated transparency");
    // Exact 3:4 resampling only: no cropping, painting, palette or alpha edits.
    let output = source.resize_exact(384, 512, image::imageops::FilterType::Nearest);
    assert_eq!(output.to_rgba8().get_pixel(0, 0)[3], 0);
    output.save_with_format(Path::new(&arguments[2]), image::ImageFormat::Png)?;
    Ok(())
}
