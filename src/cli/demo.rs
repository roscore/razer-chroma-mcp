//! `demo` subcommand — 3-second terracotta→cream left-to-right sweep on keyboard.
//!
//! Acts as a hardware smoke test: init session, play sweep, tear down.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig};
use anyhow::{Context, Result};
use std::time::{Duration, Instant};

const TERRACOTTA: Rgb = Rgb::new(0xD9, 0x77, 0x57);
const CREAM: Rgb = Rgb::new(0xF0, 0xEE, 0xE6);

pub async fn run() -> Result<()> {
    let client = ChromaClient::default();
    let session = Session::init(client, SessionConfig::default())
        .await
        .context("could not start Chroma session — is Synapse running?")?;

    // Run the sweep inside a scoped block so that `shutdown()` always runs,
    // even if a `put_effect` call errors out mid-animation. This guarantees
    // the Synapse-side session is deleted cleanly.
    let result: Result<()> = async {
        // Initial fill to terracotta
        let grid0 = keyboard_uniform(TERRACOTTA);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid0))
            .await?;

        let start = Instant::now();
        let total = Duration::from_millis(3_000);
        let mut tick = tokio::time::interval(Duration::from_millis(50));
        tick.set_missed_tick_behavior(tokio::time::MissedTickBehavior::Skip);

        while Instant::now().duration_since(start) < total {
            tick.tick().await;
            let t = Instant::now().duration_since(start).as_millis() as f32 / total.as_millis() as f32;
            let leading_col = (t * (KEYBOARD_COLS as f32 + 4.0)) as i32 - 2;
            let grid = sweep_frame(leading_col);
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
        }
        Ok(())
    }
    .await;

    session.shutdown().await;
    result?;
    println!("demo complete");
    Ok(())
}

#[allow(clippy::needless_range_loop)]
fn sweep_frame(leading_col: i32) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
    let mut grid = [[TERRACOTTA; KEYBOARD_COLS]; KEYBOARD_ROWS];
    for row in 0..KEYBOARD_ROWS {
        for col in 0..KEYBOARD_COLS {
            let dist = (col as i32 - leading_col).abs();
            if dist <= 2 {
                grid[row][col] = CREAM;
            }
        }
    }
    grid
}
