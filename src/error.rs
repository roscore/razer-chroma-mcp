//! Crate-wide error type. Per spec §8.1.

use thiserror::Error;

#[derive(Error, Debug)]
pub enum ChromaError {
    #[error("Razer Synapse is not running. Please start Synapse and try again.\nRazer Synapse가 실행 중이 아닙니다.")]
    SynapseNotRunning,

    #[error("Chroma SDK module is disabled. Enable it in Synapse settings.\nSynapse 설정에서 Chroma 모듈을 켜주세요.")]
    ChromaModuleDisabled,

    #[error("Session init failed: {0}\n세션 초기화 실패: {0}")]
    SessionInitFailed(String),

    #[error("Chroma effect PUT failed: {0}\nChroma 효과 전송 실패: {0}")]
    EffectPutFailed(String),

    #[error("Heartbeat failed ({failures}× in a row): {cause}\n하트비트 연속 {failures}회 실패: {cause}")]
    HeartbeatFailed { failures: u32, cause: String },

    #[error("Network error: {0}\n네트워크 오류: {0}")]
    Network(#[from] reqwest::Error),

    #[error("Invalid grid size: expected {expected:?}, got {actual:?}\n잘못된 그리드 크기: 기대 {expected:?}, 실제 {actual:?}")]
    InvalidGridSize {
        expected: (usize, usize),
        actual: (usize, usize),
    },

    #[error("Invalid color: {0}. Expected `#RRGGBB` / `#RGB` hex or {{r,g,b}} 0..=255.\n잘못된 색상 값: {0}. `#RRGGBB` / `#RGB` 16진수 또는 {{r,g,b}} 0..=255 형식을 사용하세요.")]
    InvalidColor(String),

    #[error("Unknown device: {0}. Known: keyboard, mouse, mousepad, headset, keypad, chromalink.\n알 수 없는 디바이스: {0}. 지원 목록: keyboard, mouse, mousepad, headset, keypad, chromalink.")]
    UnknownDevice(String),

    #[error("Config file corrupt: {path}. A copy has been saved to {quarantine}.\n설정 파일 손상: {path}. 사본을 {quarantine} 에 저장했습니다.")]
    ConfigCorrupt { path: String, quarantine: String },
}

pub type Result<T> = std::result::Result<T, ChromaError>;
