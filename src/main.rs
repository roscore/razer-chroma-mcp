use anyhow::Result;
use clap::Parser;
use razer_chroma_mcp::cli::{Cli, Command};
use std::path::Path;
use std::time::{Duration, SystemTime};

/// Retain daily log files for 7 days. Older files are pruned on startup so
/// the `%LOCALAPPDATA%\razer-chroma-mcp\logs` directory stays bounded even
/// if the user never cleans it up manually. Failures are swallowed — logging
/// must never block a release build from starting.
const LOG_RETENTION: Duration = Duration::from_secs(7 * 24 * 60 * 60);

fn prune_old_logs(dir: &Path) {
    let Ok(entries) = std::fs::read_dir(dir) else {
        return;
    };
    let now = SystemTime::now();
    for entry in entries.flatten() {
        let Ok(meta) = entry.metadata() else { continue };
        if !meta.is_file() {
            continue;
        }
        let Ok(modified) = meta.modified() else {
            continue;
        };
        if now
            .duration_since(modified)
            .map(|age| age > LOG_RETENTION)
            .unwrap_or(false)
        {
            let _ = std::fs::remove_file(entry.path());
        }
    }
}

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

    let filter =
        EnvFilter::try_from_env("RAZER_CHROMA_LOG").unwrap_or_else(|_| EnvFilter::new("info"));

    // File appender at %LOCALAPPDATA%\razer-chroma-mcp\logs\mcp.log (daily rotation, 7-day retention).
    let log_dir = std::env::var("LOCALAPPDATA")
        .map(|p| {
            std::path::PathBuf::from(p)
                .join("razer-chroma-mcp")
                .join("logs")
        })
        .unwrap_or_else(|_| std::path::PathBuf::from("./logs"));
    let _ = std::fs::create_dir_all(&log_dir);
    prune_old_logs(&log_dir);
    let file_appender = tracing_appender::rolling::daily(&log_dir, "mcp.log");

    tracing_subscriber::registry()
        .with(filter)
        .with(fmt::layer().with_writer(std::io::stderr).with_ansi(false))
        .with(fmt::layer().with_writer(file_appender).with_ansi(false))
        .init();
}

#[cfg(test)]
mod tests {
    use super::*;
    use filetime::{set_file_mtime, FileTime};
    use std::fs;

    #[test]
    fn prune_removes_files_older_than_retention_window() {
        let dir = tempfile::tempdir().unwrap();
        let fresh = dir.path().join("fresh.log");
        let stale = dir.path().join("stale.log");
        fs::write(&fresh, b"fresh").unwrap();
        fs::write(&stale, b"stale").unwrap();

        // Backdate `stale` to 8 days ago (outside the 7-day window).
        let eight_days_ago = SystemTime::now() - Duration::from_secs(8 * 24 * 60 * 60);
        set_file_mtime(&stale, FileTime::from_system_time(eight_days_ago)).unwrap();

        prune_old_logs(dir.path());

        assert!(fresh.exists(), "fresh log should be retained");
        assert!(!stale.exists(), "stale log should be pruned");
    }

    #[test]
    fn prune_is_silent_on_missing_dir() {
        // Must not panic or propagate — logging must never block startup.
        prune_old_logs(Path::new("<this/path/does/not/exist>"));
    }
}
