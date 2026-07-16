use anyhow::Result;
use questmancer::{storybook, terminal};

#[tokio::main]
async fn main() -> Result<()> {
    terminal::install_panic_hook();
    storybook::run().await
}
