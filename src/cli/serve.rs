//! `serve` subcommand — run the MCP stdio server with the shared Controller.

use crate::mcp::server;
use crate::signals::controller::Controller;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let controller = Controller::default();

    // Graceful shutdown on Ctrl+C / stdio EOF (EOF is handled inside the transport).
    let ctrl_for_shutdown = controller.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("received ctrl-c, shutting down");
            ctrl_for_shutdown.shutdown().await;
            std::process::exit(0);
        }
    });

    server::serve(controller).await
}
