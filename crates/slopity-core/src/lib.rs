#![forbid(unsafe_code)]

mod capability;
mod model;
mod profile_store;
mod runtime;
mod validation;

pub use capability::{CapabilitySnapshot, ResourcePlan, ResourcePlanner};
pub use model::{sample_profiles, NetworkScope, RuntimeKind, ServerId, ServerProfile, ServerState};
pub use profile_store::{
    ProfileDocument, ProfileStore, ProfileStoreError, PROFILE_SCHEMA_VERSION,
};
pub use runtime::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeHandle, RuntimeRequest,
};
pub use validation::{validate_profile, ValidationIssue, ValidationSeverity};
