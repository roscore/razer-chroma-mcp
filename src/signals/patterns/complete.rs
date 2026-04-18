//! Complete pattern — 1.8s one-shot: left→right terracotta sweep → central cream
//! asterisk flash → fade to previous. Spec §5.4 and §5.5 (shape matrix).

use crate::chroma::color::Rgb;
use crate::chroma::devices::{KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::EffectEnvelope;
use crate::chroma::session::Session;
use crate::error::Result;
use crate::signals::animator::{Pattern, FPS};
use crate::signals::palette::{self, TERRACOTTA, WARM_CREAM};
use std::future::Future;
use std::pin::Pin;

const TICK_MS: u64 = 1_000 / FPS;
const SWEEP_MS: u64 = 900;
const ASTERISK_HOLD_MS: u64 = 180;
const FADE_MS: u64 = 400;
const TOTAL_MS: u64 = SWEEP_MS + ASTERISK_HOLD_MS + FADE_MS + 320; // ≈1.8s

// Spec §5.5 asterisk mask. `1` = lit, `0` = dark.
#[rustfmt::skip]
const ASTERISK: [[u8; KEYBOARD_COLS]; KEYBOARD_ROWS] = [
    [0,0,0,0,0,0,0,0,0,0,1,0,0,0,0,0,0,0,0,0,0,0],
    [0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0],
    [0,0,0,0,0,1,0,0,0,1,1,1,0,0,0,1,0,0,0,0,0,0],
    [0,0,0,0,0,0,1,0,1,1,1,1,1,1,0,0,0,0,0,0,0,0],
    [0,0,0,0,0,1,0,0,0,1,1,1,0,0,0,1,0,0,0,0,0,0],
    [0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0,1,0,0,0,0,0],
];

pub struct CompletePattern;

impl CompletePattern {
    pub fn new() -> Self {
        Self
    }

    pub fn total_ticks() -> u64 {
        TOTAL_MS / TICK_MS
    }

    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t_ms = tick * TICK_MS;
        if t_ms < SWEEP_MS {
            Self::sweep(t_ms)
        } else if t_ms < SWEEP_MS + ASTERISK_HOLD_MS {
            Self::asterisk(0.85)
        } else if t_ms < SWEEP_MS + ASTERISK_HOLD_MS + FADE_MS {
            let fade_t = (t_ms - SWEEP_MS - ASTERISK_HOLD_MS) as f32 / FADE_MS as f32;
            Self::asterisk(0.85 * (1.0 - fade_t))
        } else {
            [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS]
        }
    }

    fn sweep(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t = t_ms as f32 / SWEEP_MS as f32;
        let leading = (t * (KEYBOARD_COLS as f32 + 4.0)) as i32 - 2;
        let mut grid = [[palette::scale(TERRACOTTA, 0.60); KEYBOARD_COLS]; KEYBOARD_ROWS];
        for row in grid.iter_mut() {
            for (c, cell) in row.iter_mut().enumerate() {
                let dist = (c as i32 - leading).abs();
                if dist <= 2 {
                    *cell = palette::lerp(TERRACOTTA, WARM_CREAM, 1.0 - dist as f32 / 2.0);
                }
            }
        }
        grid
    }

    fn asterisk(brightness: f32) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let mut grid = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
        let lit = palette::scale(WARM_CREAM, brightness);
        for (r, row) in grid.iter_mut().enumerate() {
            for (c, cell) in row.iter_mut().enumerate() {
                if ASTERISK[r][c] == 1 {
                    *cell = lit;
                }
            }
        }
        grid
    }
}

impl Default for CompletePattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for CompletePattern {
    fn next_frame<'a>(
        &'a mut self,
        session: &'a Session,
        tick: u64,
    ) -> Pin<Box<dyn Future<Output = Result<bool>> + Send + 'a>> {
        let total = Self::total_ticks();
        let grid = Self::frame(tick);
        Box::pin(async move {
            session
                .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
                .await?;
            Ok(tick + 1 < total) // false → animator reports finished
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn total_runs_approximately_1_8_seconds() {
        // Compile-time-ish sanity: 1.7–2.0s window.
        let total_ms = CompletePattern::total_ticks() * TICK_MS;
        assert!((1_700..=2_000).contains(&total_ms));
        assert!((1_700..=2_000).contains(&TOTAL_MS));
    }

    #[test]
    fn asterisk_mask_is_6x22() {
        assert_eq!(ASTERISK.len(), KEYBOARD_ROWS);
        assert_eq!(ASTERISK[0].len(), KEYBOARD_COLS);
    }

    #[test]
    fn asterisk_has_lit_center_ray() {
        // Row 3 (mid) should have the horizontal ray of the asterisk lit.
        let count: u8 = ASTERISK[3].iter().sum();
        assert!(count >= 4, "center row should have ≥4 lit keys, got {count}");
    }

    #[test]
    fn last_tick_returns_dark_frame() {
        let f = CompletePattern::frame(CompletePattern::total_ticks());
        for row in &f {
            for c in row {
                assert_eq!(*c, Rgb::BLACK);
            }
        }
    }
}
