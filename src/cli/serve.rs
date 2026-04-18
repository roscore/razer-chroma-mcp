//! `serve` subcommand — run the MCP stdio server with a `StatefulController`.

use crate::mcp::server;
use crate::signals::controller::StatefulController;
use anyhow::Result;

pub async fn run() -> Result<()> {
    let controller = StatefulController::default();
    controller.start_idle_watcher().await;

    // Ctrl-C: hard exit after shutdown (the spawned task doesn't cooperate with
    // the serve future; process::exit is the cleanest kill switch on Windows).
    let ctrl_for_signal = controller.clone();
    tokio::spawn(async move {
        if tokio::signal::ctrl_c().await.is_ok() {
            tracing::info!("received ctrl-c, shutting down");
            ctrl_for_signal.shutdown().await;
            std::process::exit(0);
        }
    });

    // Normal stdio-EOF path: rmcp `.serve()` returns when the client closes
    // stdin. Fall through to a graceful shutdown (spec §8.2) so the Chroma
    // session gets a clean DELETE and the heartbeat task joins.
    let result = server::serve(controller.clone()).await;
    controller.shutdown().await;
    result
}
