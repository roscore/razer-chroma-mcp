//! NeedsInput pattern — 3-stage attention-grabber. Spec §5.4.
//!
//! Stage 1 (0..900ms): triple cream flash, each flash ≤300ms, all flashes ≥90% brightness.
//! Stage 2 (loops): full terracotta breathing, 1.2s cycle, 60–100% amplitude.
//! Stage 3 (overlay from start of stage 2): spacebar+Enter+Esc cream at 0.75s
//!          counter-rhythm. Polyrhythm against the 1.2s base — intentional.

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
pub const STAGE1_FLASHES: u64 = 3;
pub const FLASH_ON_MS: u64 = 150;
pub const FLASH_OFF_MS: u64 = 150;
pub const FLASH_PERIOD_MS: u64 = FLASH_ON_MS + FLASH_OFF_MS; // 300
pub const STAGE1_MS: u64 = STAGE1_FLASHES * FLASH_PERIOD_MS; // 900
pub const BREATH_CYCLE_MS: u64 = 1_200;
pub const ACCENT_CYCLE_MS: u64 = 750;

// Accent key grid coordinates (approximate BlackWidow US layout).
// Spacebar row: row 5, cols 4..16; Enter: row 3 col 19; Esc: row 0 col 0.
pub const SPACEBAR_ROW: usize = 5;
pub const SPACEBAR_COLS: std::ops::Range<usize> = 4..16;
pub const ENTER_ROW: usize = 3;
pub const ENTER_COL: usize = 19;
pub const ESC_ROW: usize = 0;
pub const ESC_COL: usize = 0;

pub struct NeedsInputPattern;

impl NeedsInputPattern {
    pub fn new() -> Self {
        Self
    }

    pub fn frame(tick: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let t_ms = tick * TICK_MS;
        if t_ms < STAGE1_MS {
            Self::stage1(t_ms)
        } else {
            Self::stage23(t_ms - STAGE1_MS)
        }
    }

    fn stage1(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let in_flash = (t_ms % FLASH_PERIOD_MS) < FLASH_ON_MS;
        // Use WARM_CREAM at full intensity — its max channel is 240/255 ≈ 0.94,
        // which satisfies the spec ≥90% brightness floor. `scale(_, 0.95)` would
        // round to 228/255 ≈ 0.89 and trip the assertion.
        let color = if in_flash { WARM_CREAM } else { Rgb::BLACK };
        [[color; KEYBOARD_COLS]; KEYBOARD_ROWS]
    }

    fn stage23(t_ms: u64) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
        let breath_phase = (t_ms % BREATH_CYCLE_MS) as f32 / BREATH_CYCLE_MS as f32;
        let breath = (breath_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let brightness = 0.60 + 0.40 * breath; // 60–100%
        let base = palette::scale(TERRACOTTA, brightness);

        let accent_phase = (t_ms % ACCENT_CYCLE_MS) as f32 / ACCENT_CYCLE_MS as f32;
        let accent_on = (accent_phase * std::f32::consts::TAU).sin() * 0.5 + 0.5;
        let accent_brightness = 0.5 + 0.5 * accent_on;
        let accent = palette::scale(WARM_CREAM, accent_brightness);

        let mut grid = [[base; KEYBOARD_COLS]; KEYBOARD_ROWS];
        for c in SPACEBAR_COLS {
            grid[SPACEBAR_ROW][c] = accent;
        }
        grid[ENTER_ROW][ENTER_COL] = accent;
        grid[ESC_ROW][ESC_COL] = accent;
        grid
    }
}

impl Default for NeedsInputPattern {
    fn default() -> Self {
        Self::new()
    }
}

impl Pattern for NeedsInputPattern {
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

    /// Acceptance criterion from spec §5.4 / §16:
    /// "≥3 cream flashes ≥90% brightness within 900 ms".
    #[test]
    fn stage1_produces_three_flashes_at_90_percent_within_900ms() {
        let mut flash_frames = 0u64;
        let mut saw_on_off_on = false;
        let mut last_on = false;
        let mut transitions = 0u64;
        let total_ticks = 900 / TICK_MS; // 18
        for tick in 0..total_ticks {
            let frame = NeedsInputPattern::frame(tick);
            let center = frame[3][11];
            let is_on = center.r > 0;
            if is_on {
                let max_channel = center.r.max(center.g).max(center.b) as f32 / 255.0;
                assert!(max_channel >= 0.90, "flash brightness {max_channel} < 90%");
                flash_frames += 1;
            }
            if is_on != last_on {
                transitions += 1;
                last_on = is_on;
            }
            if transitions >= 5 {
                saw_on_off_on = true;
            }
        }
        assert!(
            flash_frames >= 3,
            "need at least 3 lit frames, got {flash_frames}"
        );
        assert!(
            saw_on_off_on,
            "need at least 3 distinct flashes (≥5 on↔off transitions)"
        );
    }

    #[test]
    fn stage2_breathing_amplitude_at_least_40_percent() {
        // Spec §5.4 specifies "60–100% amplitude" on the *scale factor* applied
        // to TERRACOTTA, which gives a 40-point amplitude window. Divide the
        // observed RGB channel by TERRACOTTA's max channel (217) — not 255 —
        // to recover that scale factor from the rendered frame.
        let terra_max = TERRACOTTA.r.max(TERRACOTTA.g).max(TERRACOTTA.b) as f32;
        let start = STAGE1_MS / TICK_MS;
        let end = start + (BREATH_CYCLE_MS / TICK_MS);
        let mut min_b = 1.0f32;
        let mut max_b = 0.0f32;
        for tick in start..end {
            let frame = NeedsInputPattern::frame(tick);
            // Use a non-accent key: row 2, col 10.
            let c = frame[2][10];
            let b = c.r.max(c.g).max(c.b) as f32 / terra_max;
            min_b = min_b.min(b);
            max_b = max_b.max(b);
        }
        assert!(
            max_b - min_b >= 0.40,
            "breathing amplitude {} < 40%",
            max_b - min_b
        );
    }

    #[test]
    fn stage3_accents_differ_from_base_on_counter_rhythm() {
        // Pick a tick inside stage 2/3. Spacebar keys should differ from body keys.
        let tick = (STAGE1_MS + 400) / TICK_MS;
        let frame = NeedsInputPattern::frame(tick);
        let body = frame[2][10];
        let spacebar = frame[SPACEBAR_ROW][8];
        assert_ne!(
            body, spacebar,
            "spacebar should differ from body during stage 2+3"
        );
    }
}
