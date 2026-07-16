use anyhow::Result;

pub mod assets;
pub mod atlas;
pub mod catalogue;
pub mod fixtures;

pub use assets::{AssetId, CompatibilityAsset, SceneAsset, WidgetAsset, asset_inventory};

#[allow(
    clippy::unused_async,
    reason = "the Task 1 entrypoint preserves the async runtime interface implemented in Task 8"
)]
pub async fn run() -> Result<()> {
    Ok(())
}
