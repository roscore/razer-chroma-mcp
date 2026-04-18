//! `pattern` subcommand — preview a named pattern without Claude Desktop.
//!
//! Useful for hardware verification and recording demos. Runs the requested
//! pattern on whatever Chroma device is connected, for `--seconds` seconds (or
//! until Ctrl+C), then shuts the session down cleanly.

use crate::signals::controller::{Incoming, StatefulController};
use anyhow::{anyhow, Result};
use std::time::Duration;

pub async fn run(name: String, seconds: u64) -> Result<()> {
    let incoming = match name.as_str() {
        "thinking" => Incoming::Thinking,
        "needs_input" => Incoming::NeedsInput,
        "idle" => Incoming::Idle,
        "complete" => Incoming::Complete,
        "error" => Incoming::Error,
        other => {
            return Err(anyhow!(
                "unknown pattern '{other}'. Known: thinking, needs_input, idle, complete, error"
            ));
        }
    };

    let controller = StatefulController::default();
    controller.transition(incoming).await?;
    println!("playing '{name}' for {seconds}s (Ctrl+C to stop)");

    tokio::select! {
        _ = tokio::time::sleep(Duration::from_secs(seconds)) => {}
        _ = tokio::signal::ctrl_c() => {
            println!("interrupted");
        }
    }

    controller.shutdown().await;
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn unknown_pattern_returns_error() {
        let err = run("bogus".into(), 1).await.unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("unknown pattern"), "got: {msg}");
        assert!(msg.contains("thinking"), "got: {msg}");
    }
}
