//! MCP tool handlers. Each tool's input schema is declared inline; the handler
//! delegates to `signals::controller::Controller`.

use crate::chroma::color::Rgb;
use crate::signals::controller::Controller;
use anyhow::{anyhow, Result};
use schemars::JsonSchema;
use serde::{Deserialize, Serialize};

#[derive(Deserialize, JsonSchema)]
pub struct SetStaticColorArgs {
    pub device: Option<String>, // reserved — ignored in M3, used from M4
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

/// Structured output for tools that only signal success/failure.
#[derive(Serialize, JsonSchema)]
pub struct StatusOutput {
    pub status: String,
}

/// Structured output for `get_session_status`.
#[derive(Serialize, JsonSchema)]
pub struct SessionStatusOutput {
    pub initialized: bool,
    pub state: String,
}

pub async fn set_static_color(ctrl: &Controller, args: SetStaticColorArgs) -> Result<StatusOutput> {
    let rgb = args.color.to_rgb()?;
    ctrl.set_static(rgb).await.map_err(|e| anyhow!(e))?;
    Ok(StatusOutput { status: "ok".to_string() })
}

pub async fn clear_all(ctrl: &Controller) -> Result<StatusOutput> {
    ctrl.clear_all().await.map_err(|e| anyhow!(e))?;
    Ok(StatusOutput { status: "ok".to_string() })
}

pub async fn get_session_status(ctrl: &Controller) -> Result<SessionStatusOutput> {
    let status = ctrl.session_status().await;
    Ok(SessionStatusOutput {
        initialized: !matches!(status, crate::chroma::session::SessionStatus::Uninitialized),
        state: format!("{status:?}"),
    })
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
}
