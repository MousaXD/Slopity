use crate::{RuntimeKind, ServerId, ServerProfile};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAvailability {
    pub available: bool,
    pub runtime: RuntimeKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeRequest {
    pub profile: ServerProfile,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeHandle {
    pub server_id: ServerId,
    pub process_id: Option<u32>,
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime is unavailable: {0}")]
    Unavailable(String),
    #[error("invalid runtime request: {0}")]
    InvalidRequest(String),
    #[error("server is already running: {0}")]
    AlreadyRunning(String),
    #[error("server is not running: {0}")]
    NotRunning(String),
    #[error("process operation failed: {0}")]
    Process(String),
}

pub trait RuntimeAdapter: Send {
    fn runtime_kind(&self) -> RuntimeKind;
    fn availability(&self) -> RuntimeAvailability;
    fn start(&mut self, request: RuntimeRequest) -> Result<RuntimeHandle, RuntimeError>;
    fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError>;
}
