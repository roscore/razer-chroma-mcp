//! razer-chroma-mcp — MCP server for Razer Chroma devices.
//!
//! See `docs/superpowers/specs/2026-04-17-razer-chroma-mcp-design.md`.

pub mod chroma;
pub mod cli;
pub mod config;
pub mod error;
pub mod mcp;
pub mod signals;

pub use error::ChromaError;
