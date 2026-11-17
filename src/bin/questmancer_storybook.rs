use anyhow::Result;
use clap::Parser;
use questmancer::{storybook, terminal};

#[derive(Debug, Parser)]
struct Options {
    #[arg(long)]
    story: Option<String>,
    #[arg(long)]
    inspect: bool,
    #[arg(long)]
    no_graphics: bool,
}

#[tokio::main]
async fn main() -> Result<()> {
    terminal::install_panic_hook();
    let options = Options::parse();
    storybook::run_with_options(storybook::RunOptions {
        initial_story: options.story,
        inspect: options.inspect,
        graphics: if options.no_graphics {
            storybook::GraphicsMode::Off
        } else {
            storybook::GraphicsMode::Detect
        },
    })
    .await
}
