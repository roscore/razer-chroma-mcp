//! rmcp stdio server bootstrap. Registers all 12 MCP tools (7 low-level +
//! 5 signal_*) over stdio using the rmcp macro router.
//!
//! Deviation from plan pseudo-code (T3.4/T4.12): the plan targeted a hypothetical
//! `rmcp::server::Server::builder().tool(...)` surface; rmcp 1.5 exposes
//! `#[tool]`/`#[tool_router]`/`#[tool_handler]` attribute macros instead. Schema
//! generation is driven by `schemars::JsonSchema` derives on each arg struct,
//! which covers the schema shapes enumerated in plan Step 2.

use crate::mcp::tools::{
    self, ClearDeviceArgs, DeviceInfoOutput, EmptyArgs, PlayPresetArgs, SessionStatusOutput,
    SetCustomGridArgs, SetStaticColorArgs, SignalErrorArgs, SignalStatusOutput, StatusOutput,
};
use crate::signals::controller::StatefulController;
use anyhow::Result;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::ErrorData;
use rmcp::transport::io::stdio;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};

#[derive(Clone)]
pub struct RazerChromaServer {
    ctrl: StatefulController,
    tool_router: ToolRouter<Self>,
}

impl std::fmt::Debug for RazerChromaServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RazerChromaServer").finish_non_exhaustive()
    }
}

impl RazerChromaServer {
    pub fn new(ctrl: StatefulController) -> Self {
        Self {
            ctrl,
            tool_router: Self::tool_router(),
        }
    }
}

fn internal<E: std::fmt::Display>(e: E) -> ErrorData {
    ErrorData::internal_error(e.to_string(), None)
}

#[tool_router(router = tool_router)]
impl RazerChromaServer {
    // ── Low-level tools ──────────────────────────────────────────────────────

    #[tool(
        description = "Set a single solid color on the keyboard. Accepts hex strings (#RRGGBB / #RGB) or an {r,g,b} object with 0..=255 components."
    )]
    pub async fn set_static_color(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(args): Parameters<SetStaticColorArgs>,
    ) -> Result<Json<StatusOutput>, ErrorData> {
        Ok(Json(
            tools::set_static_color(&self.ctrl, args)
                .await
                .map_err(internal)?,
        ))
    }

    #[tool(
        description = "Set a 6x22 custom color grid on the keyboard. `grid` is a 6-row array of 22-element rows, each element either a hex string or {r,g,b} object."
    )]
    pub async fn set_custom_grid(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(args): Parameters<SetCustomGridArgs>,
    ) -> Result<Json<StatusOutput>, ErrorData> {
        Ok(Json(
            tools::set_custom_grid(&self.ctrl, args)
                .await
                .map_err(internal)?,
        ))
    }

    #[tool(
        description = "Play a built-in Chroma preset effect. effect_type: WAVE | SPECTRUMCYCLING | BREATHING | REACTIVE. See spec §6.3 for params."
    )]
    pub async fn play_preset_effect(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(args): Parameters<PlayPresetArgs>,
    ) -> Result<Json<StatusOutput>, ErrorData> {
        Ok(Json(
            tools::play_preset_effect(&self.ctrl, args)
                .await
                .map_err(internal)?,
        ))
    }

    #[tool(
        description = "Release a single device back to system control (sends Chroma NONE to the given device path)."
    )]
    pub async fn clear_device(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(args): Parameters<ClearDeviceArgs>,
    ) -> Result<Json<StatusOutput>, ErrorData> {
        Ok(Json(
            tools::clear_device(&self.ctrl, args)
                .await
                .map_err(internal)?,
        ))
    }

    #[tool(
        description = "Release all devices back to system control and drop the current Claude state (Off)."
    )]
    pub async fn clear_all(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<StatusOutput>, ErrorData> {
        Ok(Json(tools::clear_all(&self.ctrl).await.map_err(internal)?))
    }

    #[tool(
        description = "Return static device catalog: grid dimensions for keyboard/mouse/keypad and LED counts for strip devices."
    )]
    pub async fn get_device_info(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<DeviceInfoOutput>, ErrorData> {
        Ok(Json(tools::get_device_info().await))
    }

    #[tool(
        description = "Return the current Chroma session state and ClaudeState (initialized flag + debug strings)."
    )]
    pub async fn get_session_status(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<SessionStatusOutput>, ErrorData> {
        Ok(Json(tools::get_session_status(&self.ctrl).await))
    }

    // ── High-level signal_* tools ────────────────────────────────────────────

    #[tool(description = "Signal that Claude is thinking (radial pulse pattern). See spec §5.4.")]
    pub async fn signal_thinking(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<SignalStatusOutput>, ErrorData> {
        Ok(Json(
            tools::signal_thinking(&self.ctrl).await.map_err(internal)?,
        ))
    }

    #[tool(
        description = "Signal that Claude needs user input (3-stage attention-grabber). See spec §5.4."
    )]
    pub async fn signal_needs_input(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<SignalStatusOutput>, ErrorData> {
        Ok(Json(
            tools::signal_needs_input(&self.ctrl)
                .await
                .map_err(internal)?,
        ))
    }

    #[tool(description = "Signal that Claude is idle (deep ember breathing). See spec §5.4.")]
    pub async fn signal_idle(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<SignalStatusOutput>, ErrorData> {
        Ok(Json(
            tools::signal_idle(&self.ctrl).await.map_err(internal)?,
        ))
    }

    #[tool(
        description = "Signal that Claude completed a task (1.8s sweep+asterisk one-shot, then restores previous state)."
    )]
    pub async fn signal_complete(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(_): Parameters<EmptyArgs>,
    ) -> Result<Json<SignalStatusOutput>, ErrorData> {
        Ok(Json(
            tools::signal_complete(&self.ctrl).await.map_err(internal)?,
        ))
    }

    #[tool(
        description = "Signal that Claude hit an error (burnt-rust slow breathing). Optional `message` is logged to the tracing warn channel."
    )]
    pub async fn signal_error(
        &self,
        #[allow(clippy::needless_pass_by_value)] Parameters(args): Parameters<SignalErrorArgs>,
    ) -> Result<Json<SignalStatusOutput>, ErrorData> {
        Ok(Json(
            tools::signal_error(&self.ctrl, args)
                .await
                .map_err(internal)?,
        ))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for RazerChromaServer {}

/// Blocking entry point: initialize the server, attach stdio, and run until
/// the peer disconnects or the process is cancelled.
pub async fn serve(ctrl: StatefulController) -> Result<()> {
    let server = RazerChromaServer::new(ctrl);
    let running = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("rmcp serve failed: {e}"))?;
    running.waiting().await?;
    Ok(())
}
