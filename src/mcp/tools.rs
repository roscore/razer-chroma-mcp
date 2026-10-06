//! MCP tool handlers. See spec §6. Each tool's input schema is derived via
//! `schemars::JsonSchema`; handlers delegate to `signals::controller::StatefulController`.

use crate::chroma::color::Rgb;
use crate::chroma::devices::{Device, KEYBOARD_COLS, KEYBOARD_ROWS};
use crate::chroma::effects::{
    keyboard_uniform, BreathingKind, EffectEnvelope, ReactiveDuration, WaveDirection,
};
use crate::signals::controller::{Incoming, StatefulController};
use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

// ── Input arg types ──────────────────────────────────────────────────────────

#[derive(Deserialize, JsonSchema)]
pub struct SetStaticColorArgs {
    /// Reserved for multi-device support; v0.1.0 targets `keyboard`.
    pub device: Option<String>,
    pub color: ColorInput,
}

#[derive(Deserialize, JsonSchema)]
#[serde(untagged)]
pub enum ColorInput {
    Hex(String),
    Rgb { r: u8, g: u8, b: u8 },
}

impl ColorInput {
    pub fn to_rgb(&self) -> Result<Rgb> {
        match self {
            ColorInput::Hex(s) => Rgb::from_hex(s).map_err(|e| anyhow!(e)),
            ColorInput::Rgb { r, g, b } => Ok(Rgb::new(*r, *g, *b)),
        }
    }
}

#[derive(Deserialize, JsonSchema)]
pub struct SetCustomGridArgs {
    /// Reserved; v0.1.0 accepts only the keyboard grid.
    pub device: Option<String>,
    /// 6 rows × 22 cols of `ColorInput`.
    pub grid: Vec<Vec<ColorInput>>,
}

#[derive(Deserialize, JsonSchema)]
pub struct PlayPresetArgs {
    pub device: Option<String>,
    /// One of: `WAVE`, `SPECTRUMCYCLING`, `BREATHING`, `REACTIVE`.
    pub effect_type: String,
    /// Effect-specific params. See spec §6.3.
    #[serde(default)]
    pub params: serde_json::Value,
}

#[derive(Deserialize, JsonSchema)]
pub struct ClearDeviceArgs {
    pub device: String,
}

#[derive(Deserialize, JsonSchema, Default)]
pub struct EmptyArgs {}

#[derive(Deserialize, JsonSchema, Default)]
pub struct SignalErrorArgs {
    pub message: Option<String>,
}

// ── Output types ─────────────────────────────────────────────────────────────

#[derive(Serialize, JsonSchema)]
pub struct StatusOutput {
    pub status: String,
}

impl StatusOutput {
    fn ok() -> Self {
        Self {
            status: "ok".to_string(),
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct SignalStatusOutput {
    pub status: String,
    pub state: String,
}

impl SignalStatusOutput {
    fn ok(state: &str) -> Self {
        Self {
            status: "ok".to_string(),
            state: state.to_string(),
        }
    }
}

#[derive(Serialize, JsonSchema)]
pub struct SessionStatusOutput {
    pub initialized: bool,
    pub session_status: String,
    pub state: String,
}

#[derive(Serialize, JsonSchema)]
pub struct DeviceInfoOutput {
    pub devices: serde_json::Value,
}

// ── Low-level handlers ───────────────────────────────────────────────────────

pub async fn set_static_color(
    c: &StatefulController,
    a: SetStaticColorArgs,
) -> Result<StatusOutput> {
    let rgb = a.color.to_rgb()?;
    let grid = keyboard_uniform(rgb);
    c.apply_user_frame(EffectEnvelope::keyboard_custom(&grid))
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(StatusOutput::ok())
}

pub async fn set_custom_grid(c: &StatefulController, a: SetCustomGridArgs) -> Result<StatusOutput> {
    if a.grid.len() != KEYBOARD_ROWS {
        return Err(anyhow!("grid must have {KEYBOARD_ROWS} rows"));
    }
    let mut arr = [[Rgb::BLACK; KEYBOARD_COLS]; KEYBOARD_ROWS];
    for (r, row) in a.grid.iter().enumerate() {
        if row.len() != KEYBOARD_COLS {
            return Err(anyhow!("row {r} must have {KEYBOARD_COLS} cols"));
        }
        for (col_idx, col) in row.iter().enumerate() {
            arr[r][col_idx] = col.to_rgb()?;
        }
    }
    c.apply_user_frame(EffectEnvelope::keyboard_custom(&arr))
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(StatusOutput::ok())
}

pub async fn play_preset_effect(c: &StatefulController, a: PlayPresetArgs) -> Result<StatusOutput> {
    let effect = match a.effect_type.to_ascii_uppercase().as_str() {
        "WAVE" => {
            let dir = a
                .params
                .get("direction")
                .and_then(|v| v.as_str())
                .unwrap_or("left_to_right");
            let d = if dir == "right_to_left" {
                WaveDirection::RightToLeft
            } else {
                WaveDirection::LeftToRight
            };
            EffectEnvelope::wave(d)
        }
        "SPECTRUMCYCLING" => EffectEnvelope::spectrumcycling(),
        "BREATHING" => {
            let kind = if let Some(c) = a.params.get("color") {
                let v: ColorInput = serde_json::from_value(c.clone())?;
                BreathingKind::OneColor(v.to_rgb()?)
            } else {
                BreathingKind::Random
            };
            EffectEnvelope::breathing(kind)
        }
        "REACTIVE" => {
            let color = a
                .params
                .get("color")
                .ok_or_else(|| anyhow!("reactive requires color"))?;
            let v: ColorInput = serde_json::from_value(color.clone())?;
            let duration = match a
                .params
                .get("duration")
                .and_then(|v| v.as_str())
                .unwrap_or("medium")
            {
                "short" => ReactiveDuration::Short,
                "long" => ReactiveDuration::Long,
                _ => ReactiveDuration::Medium,
            };
            EffectEnvelope::reactive(v.to_rgb()?, duration)
        }
        other => return Err(anyhow!("unsupported effect_type: {other}")),
    };
    c.apply_user_frame(effect).await.map_err(|e| anyhow!(e))?;
    Ok(StatusOutput::ok())
}

pub async fn clear_device(c: &StatefulController, a: ClearDeviceArgs) -> Result<StatusOutput> {
    let device = Device::parse(&a.device).map_err(|e| anyhow!(e))?;
    let session = c.session().await.map_err(|e| anyhow!(e))?;
    session
        .put_effect(device.path_segment(), &EffectEnvelope::none())
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(StatusOutput::ok())
}

pub async fn clear_all(c: &StatefulController) -> Result<StatusOutput> {
    c.force_transition(crate::signals::ClaudeState::Off)
        .await
        .map_err(|e| anyhow!(e))?;
    if let Ok(session) = c.session().await {
        for device in [
            "keyboard",
            "mouse",
            "mousepad",
            "headset",
            "keypad",
            "chromalink",
        ] {
            let _ = session.put_effect(device, &EffectEnvelope::none()).await;
        }
    }
    Ok(StatusOutput::ok())
}

pub async fn get_device_info() -> DeviceInfoOutput {
    DeviceInfoOutput {
        devices: serde_json::json!([
            { "name": "keyboard",   "rows": 6, "cols": 22 },
            { "name": "mouse",      "rows": 9, "cols": 7 },
            { "name": "keypad",     "rows": 4, "cols": 5 },
            { "name": "mousepad",   "leds": 20 },
            { "name": "headset",    "leds": 5 },
            { "name": "chromalink", "leds": 5 }
        ]),
    }
}

pub async fn get_session_status(c: &StatefulController) -> SessionStatusOutput {
    let status = c.session_status().await;
    let state = c.current_state().await;
    SessionStatusOutput {
        initialized: !matches!(status, crate::chroma::session::SessionStatus::Uninitialized),
        session_status: format!("{status:?}"),
        state: format!("{state:?}"),
    }
}

// ── High-level signal_* handlers ─────────────────────────────────────────────

pub async fn signal_thinking(c: &StatefulController) -> Result<SignalStatusOutput> {
    c.transition(Incoming::Thinking)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(SignalStatusOutput::ok("Thinking"))
}

pub async fn signal_needs_input(c: &StatefulController) -> Result<SignalStatusOutput> {
    c.transition(Incoming::NeedsInput)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(SignalStatusOutput::ok("NeedsInput"))
}

pub async fn signal_idle(c: &StatefulController) -> Result<SignalStatusOutput> {
    c.transition(Incoming::Idle).await.map_err(|e| anyhow!(e))?;
    Ok(SignalStatusOutput::ok("Idle"))
}

pub async fn signal_complete(c: &StatefulController) -> Result<SignalStatusOutput> {
    c.transition(Incoming::Complete)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(SignalStatusOutput::ok("Complete"))
}

pub async fn signal_error(
    c: &StatefulController,
    a: SignalErrorArgs,
) -> Result<SignalStatusOutput> {
    if let Some(m) = &a.message {
        tracing::warn!(target: "claude_signal", "signal_error: {m}");
    }
    c.transition(Incoming::Error)
        .await
        .map_err(|e| anyhow!(e))?;
    Ok(SignalStatusOutput::ok("Error"))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn color_input_parses_hex() {
        let v: SetStaticColorArgs =
            serde_json::from_value(serde_json::json!({ "color": "#D97757" })).unwrap();
        assert_eq!(v.color.to_rgb().unwrap(), Rgb::new(0xD9, 0x77, 0x57));
    }

    #[test]
    fn color_input_parses_rgb_object() {
        let v: SetStaticColorArgs =
            serde_json::from_value(serde_json::json!({ "color": { "r": 1, "g": 2, "b": 3 } }))
                .unwrap();
        assert_eq!(v.color.to_rgb().unwrap(), Rgb::new(1, 2, 3));
    }

    #[test]
    fn empty_args_deserializes_from_empty_object() {
        let _: EmptyArgs = serde_json::from_value(serde_json::json!({})).unwrap();
    }

    #[test]
    fn signal_error_args_message_optional() {
        let a: SignalErrorArgs = serde_json::from_value(serde_json::json!({})).unwrap();
        assert!(a.message.is_none());
        let a: SignalErrorArgs =
            serde_json::from_value(serde_json::json!({ "message": "boom" })).unwrap();
        assert_eq!(a.message.as_deref(), Some("boom"));
    }
}
