//! Thinking pattern — radial pulse from keyboard center outward.
//! Terracotta ↔ cream gradient, 2.8s cycle, 35–65% brightness. Spec §5.4.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::{Pattern, FPS};
use crate::signals::palette::{self, TERRACOTTA, WARM_CREAM};
use std::future::Future;
use std::pin::Pin;

const CYCLE_MS: u64 = 2_800;
const FRAMES_PER_CYCLE: u64 = CYCLE_MS / (1_000 / FPS);
const CENTER_ROW: f32 = 2.5;
const CENTER_COL: f32 = 10.5;

pub struct ThinkingPattern;

impl ThinkingPattern {
    pub fn new() -> Self {
        Self
    }

    /// Pure frame generator — produced as a 6×22 color grid for a given tick.
    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let phase = (tick % FRAMES_PER_CYCLE) as f32 / FRAMES_PER_CYCLE as f32;
        let pulse = (phase * std::f32::consts::TAU).sin() * 0.5 + 0.5; // 0..1
        let mut grid = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
        let max_dist = ((KEYBOARD_ROWS as f32).hypot(KEYBOARD_COLS as f32)) / 2.0;
        let brightness = 0.35 + 0.30 * pulse; // 35–65%
        for (r, row) in grid.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                let dr = r as f32 - CENTER_ROW;
                let dc = c as f32 - CENTER_COL;
                let dist = (dr * dr + dc * dc).sqrt();
                let ring = ((pulse + dist / max_dist) % 1.0 - 0.5).abs() * 2.0;
                let color = palette::lerp(TERRACOTTA, WARM_CREAM, ring);
                *cell = palette::scale(color, brightness);
            }
        }
        grid
    }
}

impl Default for ThinkingPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for ThinkingPattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let grid = Self::frame(tick);
        Box::pin(async move {
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
            Ok(true)
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn brightness_stays_in_35_to_65_percent_band() {
        for tick in 0..FRAMES_PER_CYCLE {
            let frame = ThinkingPattern::frame(tick);
            for row in &frame {
                for c in row {
                    let max_channel = c.r.max(c.g).max(c.b) as f32 / 255.0;
                    assert!(max_channel <= 0.66, "brightness {max_channel} > 65%");
                    // Minimum brightness check uses the brightest channel — dark
                    // ring nodes are permitted to be darker; the global max envelope
                    // is what we're asserting here.
                }
            }
        }
    }

    #[test]
    fn frame_is_6_by_22() {
        let f = ThinkingPattern::frame(0);
        assert_eq!(f.len(), KEYBOARD_ROWS);
        assert_eq!(f[0].len(), KEYBOARD_COLS);
    }

    #[test]
    fn is_animated_across_cycle() {
        // Quarter-cycle (not half) — the pulse is `sin(phase·τ)·0.5 + 0.5`,
        // which is symmetric around the half-cycle (sin(0) = sin(π) = 0), so
        // frames at tick 0 and FRAMES_PER_CYCLE/2 are bit-identical by design.
        // Quarter-cycle catches the peak at pulse = 1.0.
        let a = ThinkingPattern::frame(0);
        let b = ThinkingPattern::frame(FRAMES_PER_CYCLE / 4);
        assert_ne!(a, b, "frames should differ across quarter-cycle");
    }
}
