use anyhow::Result;

#[allow(
    clippy::unused_async,
    reason = "the Task 1 entrypoint preserves the async runtime interface implemented in Task 8"
)]
pub async fn run() -> Result<()> {
    Ok(())
}
