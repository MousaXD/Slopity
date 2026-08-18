use crate::{RuntimeKind, ServerId, ServerProfile, ServerState};
use serde::{Deserialize, Serialize};
use thiserror::Error;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeAvailability {
    pub available: bool,
    pub runtime: RuntimeKind,
    pub reason: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeIdentity {
    pub runtime: RuntimeKind,
    pub adapter: String,
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

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeLogLevel {
    Info,
    Warning,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeLogEntry {
    pub sequence: u64,
    pub level: RuntimeLogLevel,
    pub message: String,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeExitReason {
    UserRequested,
    Completed,
    Failed,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeExit {
    pub reason: RuntimeExitReason,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RuntimeObservation {
    pub server_id: ServerId,
    pub state: ServerState,
    pub bind_address: String,
    pub urls: Vec<String>,
    pub request_count: u64,
    pub logs: Vec<RuntimeLogEntry>,
    pub last_error: Option<String>,
    pub exit: Option<RuntimeExit>,
}

impl RuntimeObservation {
    pub fn running(server_id: ServerId) -> Self {
        Self {
            server_id,
            state: ServerState::Running,
            bind_address: String::new(),
            urls: Vec::new(),
            request_count: 0,
            logs: Vec::new(),
            last_error: None,
            exit: None,
        }
    }

    pub fn stopped(
        server_id: ServerId,
        reason: RuntimeExitReason,
        message: impl Into<String>,
    ) -> Self {
        Self {
            server_id,
            state: ServerState::Stopped,
            bind_address: String::new(),
            urls: Vec::new(),
            request_count: 0,
            logs: Vec::new(),
            last_error: None,
            exit: Some(RuntimeExit {
                reason,
                message: message.into(),
            }),
        }
    }
}

#[derive(Debug, Error)]
pub enum RuntimeError {
    #[error("runtime is unavailable: {0}")]
    Unavailable(String),
    #[error("invalid runtime request: {0}")]
    InvalidRequest(String),
    #[error("runtime adapter is already registered: {0}")]
    AdapterAlreadyRegistered(String),
    #[error("server is already running: {0}")]
    AlreadyRunning(String),
    #[error("server is not running: {0}")]
    NotRunning(String),
    #[error("process operation failed: {0}")]
    Process(String),
}

pub trait RuntimeAdapter: Send {
    fn runtime_kind(&self) -> RuntimeKind;

    fn runtime_identity(&self) -> RuntimeIdentity {
        RuntimeIdentity {
            runtime: self.runtime_kind(),
            adapter: format!("{:?}", self.runtime_kind()),
        }
    }

    fn availability(&self) -> RuntimeAvailability;
    fn start(&mut self, request: RuntimeRequest) -> Result<RuntimeHandle, RuntimeError>;
    fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError>;

    fn observe(&mut self, _server_id: &ServerId) -> Option<RuntimeObservation> {
        None
    }

    fn observations(&mut self) -> Vec<RuntimeObservation> {
        Vec::new()
    }
}
