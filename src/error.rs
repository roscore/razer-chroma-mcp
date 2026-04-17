//! Crate-wide error type. Per spec §8.1.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChromaError {
    #[error("Razer Synapse is not running. Please start Synapse and try again.\nRazer Synapse가 실행 중이 아닙니다.")]
    SynapseNotRunning,

    #[error("Chroma SDK module is disabled. Enable it in Synapse settings.\nSynapse 설정에서 Chroma 모듈을 켜주세요.")]
    ChromaModuleDisabled,

    #[error("Session init failed: {0}")]
    SessionInitFailed(String),

    #[error("Heartbeat failed ({failures}× in a row): {cause}")]
    HeartbeatFailed { failures: u32, cause: String },

    #[error("Network error: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid grid size: expected {expected:?}, got {actual:?}")]
    InvalidGridSize {
        expected: (usize, usize),
        actual: (usize, usize),
    },

    #[error("Invalid color: {0}")]
    InvalidColor(String),

    #[error("Unknown device: {0}")]
    UnknownDevice(String),

    #[error("Config file corrupt: {path}. A copy has been saved to {quarantine}.")]
    ConfigCorrupt { path: String, quarantine: String },
}

pub type Result<T> = std::result::Result<T, ChromaError>;
