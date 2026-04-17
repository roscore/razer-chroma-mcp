//! CLI subcommand dispatch. See spec §4.

pub mod demo;
pub mod doctor;
pub mod pattern;
pub mod serve;
pub mod setup;
pub mod uninstall;

use clap::{Parser, Subcommand};

#[derive(Parser, Debug)]
#[command(name = "razer-chroma-mcp", version, about = "MCP server for Razer Chroma devices")]
pub struct Cli {
    #[command(subcommand)]
    pub command: Option<Command>,
}

#[derive(Subcommand, Debug)]
pub enum Command {
    /// Run the MCP stdio server (default if no subcommand given).
    Serve,
    /// Interactive setup wizard.
    Setup {
        /// Force a UI language instead of auto-detecting.
        #[arg(long, value_parser = ["ko", "en"])]
        lang: Option<String>,
    },
    /// Run environment diagnostics.
    Doctor,
    /// Play a 3-second terracotta sweep to verify hardware.
    Demo,
    /// Preview a named pattern without Claude Desktop.
    Pattern {
        /// Pattern name (thinking, needs_input, idle, complete, error).
        name: String,
        /// Seconds to run before exiting.
        #[arg(long, default_value_t = 8)]
        seconds: u64,
    },
    /// Remove Claude Desktop/Code config entries.
    Uninstall,
}
