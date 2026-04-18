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
    use tracing_subscriber::{fmt, layer::SubscriberExt, util::SubscriberInitExt, EnvFilter};

    let filter = EnvFilter::try_from_env("RAZER_CHROMA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    // File appender at %LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log (daily rotation, 7-day retention).
    let log_dir = std::env::var("LOCALAPPDATA")
        .map(|p| std::path::PathBuf::from(p).join("razer-chroma-mcp").join("logs"))
        .unwrap_or_else(|_| std::path::PathBuf::from("./logs"));
    let _ = std::fs::create_dir_all(&log_dir);
    let file_appender = tracing_appender::rolling::daily(&log_dir, "mcp.log");

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr).with_ansi(false))
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .init();
}
