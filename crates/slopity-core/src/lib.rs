#![forbid(unsafe_code)]

mod capability;
mod model;
mod runtime;
mod validation;

pub use capability::{CapabilitySnapshot, ResourcePlan, ResourcePlanner};
pub use model::{
    NetworkScope, RuntimeKind, ServerId, ServerProfile, ServerState, sample_profiles,
};
pub use runtime::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeHandle, RuntimeRequest,
};
pub use validation::{ValidationIssue, ValidationSeverity, validate_profile};
