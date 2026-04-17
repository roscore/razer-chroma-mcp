use anyhow::Result;
use clap::Parser;
use razer_chroma_mcp::cli::{Cli, Command};

#[tokio::main]
async fn main() -> Result<()> {
    init_tracing();

    let cli = Cli::parse();
    match cli.command.unwrap_or(Command::Serve) {
        Command::Serve => razer_chroma_mcp::cli::serve::run().await,
        Command::Setup { lang } => razer_chroma_mcp::cli::setup::run(lang).await,
        Command::Doctor => razer_chroma_mcp::cli::doctor::run().await,
        Command::Demo => razer_chroma_mcp::cli::demo::run().await,
        Command::Pattern { name, seconds } => {
            razer_chroma_mcp::cli::pattern::run(name, seconds).await
        }
        Command::Uninstall => razer_chroma_mcp::cli::uninstall::run().await,
    }
}

fn init_tracing() {
    use tracing_subscriber::{fmt, EnvFilter};
    let filter =
        EnvFilter::try_from_env("RAZER_CHROMA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));
    fmt()
        .with_env_filter(filter)
        .with_writer(std::io::stderr)
        .init();
}
