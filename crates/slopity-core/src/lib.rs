#![forbid(unsafe_code)]

mod admission;
mod capability;
mod model;
mod orchestrator;
mod profile_store;
mod runtime;
mod validation;

pub use admission::{
    authorize_start, StartAdmissionPermit, StartAdmissionReason, StartAdmissionReasonCode,
    StartAdmissionRejection,
};
pub use capability::{
    AllocationDecision, CapabilitySnapshot, PortReservation, ResourceAccounting,
    ResourceAccountingSnapshot, ResourcePlan, ResourcePlanner, ResourceWarning,
    ResourceWarningCode,
};
pub use model::{sample_profiles, NetworkScope, RuntimeKind, ServerId, ServerProfile, ServerState};
pub use orchestrator::{
    DesiredServerState, ObservedServerState, RuntimeEvent, RuntimeEventKind,
    RuntimeFailureEvidence, ServerOrchestrator, ServerRuntimeSnapshot,
};
pub use profile_store::{
    ProfileDocument, ProfileRecoveryNotice, ProfileStore, ProfileStoreError, PROFILE_SCHEMA_VERSION,
};
pub use runtime::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeExit, RuntimeExitReason,
    RuntimeHandle, RuntimeIdentity, RuntimeLogEntry, RuntimeLogLevel, RuntimeObservation,
    RuntimeRequest,
};
pub use validation::{
    is_valid_profile_id, validate_profile, ValidationIssue, ValidationSeverity, MAX_ARGUMENT_COUNT,
    MAX_ARGUMENT_LENGTH, MAX_ARGUMENT_PAYLOAD_SIZE, MAX_EXECUTABLE_PATH_LENGTH,
    MAX_PROFILE_ID_LENGTH, MAX_PROFILE_NAME_LENGTH, MAX_WORKING_DIRECTORY_PATH_LENGTH,
};
