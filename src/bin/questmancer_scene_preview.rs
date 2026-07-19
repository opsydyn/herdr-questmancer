use anyhow::Result;

#[tokio::main]
async fn main() -> Result<()> {
    questmancer::terminal::install_panic_hook();
    questmancer::terminal::run_scene_preview().await
}
