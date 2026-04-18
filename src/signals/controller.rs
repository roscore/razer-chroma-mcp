//! Central owner of the Chroma session and (in M4) the ClaudeState machine.
//!
//! M3 surface: just enough to dispatch set_static_color / clear_all / get_session_status.

use crate::chroma::client::ChromaClient;
use crate::chroma::color::Rgb;
use crate::chroma::effects::{keyboard_uniform, EffectEnvelope};
use crate::chroma::session::{Session, SessionConfig, SessionStatus};
use crate::error::Result;
use std::sync::Arc;
use tokio::sync::OnceCell;

#[derive(Clone)]
pub struct Controller {
    client: ChromaClient,
    session: Arc<OnceCell<Session>>,
}

impl Default for Controller {
    fn default() -> Self {
        Self {
            client: ChromaClient::default(),
            session: Arc::new(OnceCell::new()),
        }
    }
}

impl Controller {
    pub async fn session(&self) -> Result<Session> {
        let s = self
            .session
            .get_or_try_init(|| Session::init(self.client.clone(), SessionConfig::default()))
            .await?;
        Ok(s.clone())
    }

    pub async fn session_status(&self) -> SessionStatus {
        match self.session.get() {
            Some(s) => s.status().await,
            None => SessionStatus::Uninitialized,
        }
    }

    pub async fn set_static(&self, color: Rgb) -> Result<()> {
        let session = self.session().await?;
        let grid = keyboard_uniform(color);
        session
            .put_effect("keyboard", &EffectEnvelope::keyboard_custom(&grid))
            .await
    }

    pub async fn clear_all(&self) -> Result<()> {
        if let Some(s) = self.session.get() {
            s.put_effect("keyboard", &EffectEnvelope::none()).await?;
        }
        Ok(())
    }

    pub async fn shutdown(&self) {
        if let Some(s) = self.session.get() {
            s.shutdown().await;
        }
    }
}
