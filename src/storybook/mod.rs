pub mod app;
pub mod assets;
pub mod catalogue;
pub mod fixtures;
pub mod input;
mod runtime;
pub mod ui;

pub use assets::{AssetId, SceneFirstAsset, asset_inventory};
pub use runtime::run;
