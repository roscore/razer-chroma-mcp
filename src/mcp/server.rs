//! rmcp stdio server bootstrap. Wires three M3 tools (set_static_color,
//! clear_all, get_session_status) over stdio using the rmcp macro router.
//!
//! Deviation from plan pseudo-code (T3.4): the plan targeted a hypothetical
//! `rmcp::server::Server::builder().tool(...)` surface; rmcp 1.5 exposes
//! `#[tool]`/`#[tool_router]`/`#[tool_handler]` attribute macros instead.
//! Handler delegation semantics match the plan: each method calls into
//! `crate::mcp::tools::*` and wraps the response in `Json<serde_json::Value>`.

use crate::mcp::tools::{self, SetStaticColorArgs};
use crate::signals::controller::Controller;
use anyhow::Result;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::{Json, Parameters};
use rmcp::model::ErrorData;
use rmcp::transport::io::stdio;
use rmcp::{tool, tool_handler, tool_router, ServerHandler, ServiceExt};

#[derive(Clone)]
pub struct RazerChromaServer {
    ctrl: Controller,
    tool_router: ToolRouter<Self>,
}

impl std::fmt::Debug for RazerChromaServer {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RazerChromaServer").finish_non_exhaustive()
    }
}

impl RazerChromaServer {
    pub fn new(ctrl: Controller) -> Self {
        Self {
            ctrl,
            tool_router: Self::tool_router(),
        }
    }
}

#[tool_router(router = tool_router)]
impl RazerChromaServer {
    /// Set a single solid color on the keyboard (Chroma CHROMA_CUSTOM).
    #[tool(description = "Set a single solid color on the keyboard. Accepts hex strings (#RRGGBB / #RGB) or an {r,g,b} object with 0..=255 components.")]
    pub async fn set_static_color(
        &self,
        #[allow(clippy::needless_pass_by_value)] // Parameters<T> consumes by value per rmcp 1.5 API
        Parameters(args): Parameters<SetStaticColorArgs>,
    ) -> Result<Json<serde_json::Value>, ErrorData> {
        let value = tools::set_static_color(&self.ctrl, args)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(Json(value))
    }

    /// Release the keyboard back to system control (Chroma NONE).
    #[tool(description = "Release the keyboard back to system control (sends Chroma NONE).")]
    pub async fn clear_all(&self) -> Result<Json<serde_json::Value>, ErrorData> {
        let value = tools::clear_all(&self.ctrl)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(Json(value))
    }

    /// Inspect the Chroma session health.
    #[tool(description = "Return the current Chroma session state (initialized flag + debug state string).")]
    pub async fn get_session_status(&self) -> Result<Json<serde_json::Value>, ErrorData> {
        let value = tools::get_session_status(&self.ctrl)
            .await
            .map_err(|e| ErrorData::internal_error(e.to_string(), None))?;
        Ok(Json(value))
    }
}

#[tool_handler(router = self.tool_router)]
impl ServerHandler for RazerChromaServer {}

/// Blocking entry point: initialize the server, attach stdio, and run until
/// the peer disconnects or the process is cancelled. Used by the `serve` CLI.
pub async fn serve(ctrl: Controller) -> Result<()> {
    let server = RazerChromaServer::new(ctrl);
    let running = server
        .serve(stdio())
        .await
        .map_err(|e| anyhow::anyhow!("rmcp serve failed: {e}"))?;
    running.waiting().await?;
    Ok(())
}
