//! Use the library directly without MCP.
//!
//! Run: `cargo run --example standalone_demo`
//!
//! Walks through four state transitions (Thinking → NeedsInput →
//! Complete → Idle) with 3-second dwells, then shuts down cleanly.
//! Useful for embedders who want to drive the controller from their
//! own orchestration rather than through the MCP stdio server.

use razer_chroma_mcp::signals::controller::{Incoming, StatefulController};
use std::time::Duration;

#[tokio::main]
async fn main() -> anyhow::Result<()> {
    let controller = StatefulController::default();
    for step in [
        Incoming::Thinking,
        Incoming::NeedsInput,
        Incoming::Complete,
        Incoming::Idle,
    ] {
        println!("transition: {step:?}");
        controller.transition(step).await?;
        tokio::time::sleep(Duration::from_secs(3)).await;
    }
    controller.shutdown().await;
    Ok(())
}
