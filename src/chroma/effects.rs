//! Razer Chroma SDK effect payloads. One struct per effect kind. See spec §4.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{Device, KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::error::{ChromaError, Result};
use serde::Serialize;

/// Common `{ "effect": "...", "param": ... }` wrapper as sent to Chroma REST API.
#[derive(Serialize, Debug, Clone)]
pub struct EffectEnvelope {
    pub effect: &'static str,
    #[serde(skip_serializing_if = "Option::is_none")]
    pub param: Option<serde_json::Value>,
}

impl EffectEnvelope {
    pub fn none() -> Self {
        Self { effect: "CHROMA_NONE", param: None }
    }

    pub fn r#static(color: Rgb) -> Self {
        Self {
            effect: "CHROMA_STATIC",
            param: Some(serde_json::json!({ "color": color.to_bgr_u32() })),
        }
    }

    pub fn keyboard_custom(grid: &[[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS]) -> Self {
        let rows: Vec<Vec<u32>> = grid
            .iter()
            .map(|row| row.iter().map(|rgb| rgb.to_bgr_u32()).collect())
            .collect();
        Self {
            effect: "CHROMA_CUSTOM",
            param: Some(serde_json::Value::Array(
                rows.into_iter()
                    .map(|r| serde_json::Value::Array(r.into_iter().map(|v| serde_json::json!(v)).collect()))
                    .collect(),
            )),
        }
    }

    pub fn spectrumcycling() -> Self {
        Self { effect: "CHROMA_SPECTRUMCYCLING", param: None }
    }

    pub fn wave(direction: WaveDirection) -> Self {
        Self {
            effect: "CHROMA_WAVE",
            param: Some(serde_json::json!({ "direction": direction as u8 })),
        }
    }

    pub fn breathing(kind: BreathingKind) -> Self {
        let v = match kind {
            BreathingKind::Random => serde_json::json!({ "type": 1 }),
            BreathingKind::OneColor(c) => serde_json::json!({ "type": 2, "color": c.to_bgr_u32() }),
            BreathingKind::TwoColors(a, b) => serde_json::json!({
                "type": 3, "color1": a.to_bgr_u32(), "color2": b.to_bgr_u32()
            }),
        };
        Self { effect: "CHROMA_BREATHING", param: Some(v) }
    }

    pub fn reactive(color: Rgb, duration: ReactiveDuration) -> Self {
        Self {
            effect: "CHROMA_REACTIVE",
            param: Some(serde_json::json!({
                "color": color.to_bgr_u32(),
                "duration": duration as u8,
            })),
        }
    }
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum WaveDirection { LeftToRight = 1, RightToLeft = 2 }

#[derive(Clone, Copy, Debug)]
pub enum BreathingKind {
    Random,
    OneColor(Rgb),
    TwoColors(Rgb, Rgb),
}

#[derive(Clone, Copy, Debug)]
#[repr(u8)]
pub enum ReactiveDuration { Short = 1, Medium = 2, Long = 3 }

/// Build a full-keyboard custom grid from a single color.
pub fn keyboard_uniform(color: Rgb) -> [[Rgb; KEYBOARD_COLS]; KEYBOARD_ROWS] {
    [[color; KEYBOARD_COLS]; KEYBOARD_ROWS]
}

/// Validate a runtime-dynamic grid matches the device's expected dimensions.
pub fn validate_grid(device: Device, rows: usize, cols: usize) -> Result<()> {
    if let Some(expected) = device.grid() {
        if expected != (rows, cols) {
            return Err(ChromaError::InvalidGridSize { expected, actual: (rows, cols) });
        }
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::chroma::color::Rgb;

    #[test]
    fn static_effect_snapshot() {
        let payload = EffectEnvelope::r#static(Rgb::new(0xD9, 0x77, 0x57));
        insta::assert_json_snapshot!(payload);
    }

    #[test]
    fn none_effect_snapshot() {
        insta::assert_json_snapshot!(EffectEnvelope::none());
    }

    #[test]
    fn breathing_two_colors_snapshot() {
        let p = EffectEnvelope::breathing(BreathingKind::TwoColors(
            Rgb::new(0x3E, 0x27, 0x23),
            Rgb::new(0xF0, 0xEE, 0xE6),
        ));
        insta::assert_json_snapshot!(p);
    }

    #[test]
    fn keyboard_custom_serializes_as_6x22() {
        let grid = keyboard_uniform(Rgb::new(1, 2, 3));
        let p = EffectEnvelope::keyboard_custom(&grid);
        let v = serde_json::to_value(&p).unwrap();
        let param = v.get("param").unwrap();
        assert!(param.is_array());
        assert_eq!(param.as_array().unwrap().len(), 6);
        assert_eq!(param.as_array().unwrap()[0].as_array().unwrap().len(), 22);
    }

    #[test]
    fn validate_grid_rejects_wrong_size() {
        assert!(matches!(
            validate_grid(Device::Keyboard, 5, 22),
            Err(ChromaError::InvalidGridSize { .. })
        ));
    }

    #[test]
    fn validate_grid_accepts_correct_size() {
        validate_grid(Device::Keyboard, 6, 22).unwrap();
    }
}
