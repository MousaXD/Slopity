use crate::{
    validate_profile, AllocationDecision, CapabilitySnapshot, ResourceAccounting, ResourceWarning,
    RuntimeAvailability, RuntimeKind, ServerId, ServerProfile, ValidationIssue, ValidationSeverity,
};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum StartAdmissionReasonCode {
    ProfileDisabled,
    ProfileInvalid,
    RuntimeUnavailable,
    PortConflict,
    ResourceAllocationUnsafe,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartAdmissionReason {
    pub code: StartAdmissionReasonCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct StartAdmissionRejection {
    pub server_id: ServerId,
    pub runtime: RuntimeKind,
    pub reasons: Vec<StartAdmissionReason>,
    pub validation_issues: Vec<ValidationIssue>,
    pub allocation: AllocationDecision,
    pub warnings: Vec<ResourceWarning>,
}

#[derive(Debug)]
pub struct StartAdmissionPermit {
    server_id: ServerId,
    runtime: RuntimeKind,
    port: u16,
    memory_mib: u32,
}

impl StartAdmissionPermit {
    pub(crate) fn matches(&self, profile: &ServerProfile) -> bool {
        self.server_id == profile.id
            && self.runtime == profile.runtime
            && self.port == profile.port
            && self.memory_mib == profile.memory_mib
    }
}

pub fn authorize_start(
    capability: &CapabilitySnapshot,
    profiles: &[ServerProfile],
    active_server_ids: &[ServerId],
    profile: &ServerProfile,
    runtime_availability: &RuntimeAvailability,
) -> Result<StartAdmissionPermit, StartAdmissionRejection> {
    let active_ids = active_server_ids
        .iter()
        .map(|id| id.0.as_str())
        .collect::<HashSet<_>>();
    let reserved_ports = profiles
        .iter()
        .filter(|candidate| candidate.id != profile.id)
        .filter(|candidate| candidate.enabled || active_ids.contains(candidate.id.0.as_str()))
        .map(|candidate| candidate.port)
        .collect::<HashSet<_>>();

    let validation_issues = validate_profile(profile, &reserved_ports);
    let allocation =
        ResourceAccounting::evaluate_allocation(capability, profiles, active_server_ids, profile);
    let warnings = allocation.warnings.clone();
    let mut reasons = Vec::new();

    if !profile.enabled {
        reasons.push(reason(
            StartAdmissionReasonCode::ProfileDisabled,
            "Enable this profile before starting it.",
        ));
    }

    if validation_issues
        .iter()
        .any(|issue| issue.severity == ValidationSeverity::Error)
    {
        reasons.push(reason(
            StartAdmissionReasonCode::ProfileInvalid,
            "The saved profile has validation errors that must be fixed before launch.",
        ));
    }

    if runtime_availability.runtime != profile.runtime || !runtime_availability.available {
        reasons.push(reason(
            StartAdmissionReasonCode::RuntimeUnavailable,
            if runtime_availability.runtime != profile.runtime {
                "The runtime availability result does not match this profile's runtime.".into()
            } else if runtime_availability.reason.trim().is_empty() {
                "No verified runtime provider is available for this profile.".into()
            } else {
                runtime_availability.reason.clone()
            },
        ));
    }

    if !allocation.conflicting_server_ids.is_empty() {
        reasons.push(reason(
            StartAdmissionReasonCode::PortConflict,
            format!(
                "Port {} is already reserved by {}.",
                profile.port,
                allocation
                    .conflicting_server_ids
                    .iter()
                    .map(|id| id.0.as_str())
                    .collect::<Vec<_>>()
                    .join(", ")
            ),
        ));
    }

    if !allocation.fits && allocation.conflicting_server_ids.is_empty() {
        reasons.push(reason(
            StartAdmissionReasonCode::ResourceAllocationUnsafe,
            "Slopity cannot prove that the requested memory allocation fits the current safe host budget and immediate headroom.",
        ));
    }

    if reasons.is_empty() {
        Ok(StartAdmissionPermit {
            server_id: profile.id.clone(),
            runtime: profile.runtime,
            port: profile.port,
            memory_mib: profile.memory_mib,
        })
    } else {
        Err(StartAdmissionRejection {
            server_id: profile.id.clone(),
            runtime: profile.runtime,
            reasons,
            validation_issues,
            allocation,
            warnings,
        })
    }
}

fn reason(code: StartAdmissionReasonCode, message: impl Into<String>) -> StartAdmissionReason {
    StartAdmissionReason {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, ResourceWarningCode};

    fn capability(total: Option<u64>, available: Option<u64>) -> CapabilitySnapshot {
        CapabilitySnapshot {
            platform: "linux".into(),
            architecture: "x86_64".into(),
            logical_cpus: 8,
            total_memory_mib: total,
            available_memory_mib: available,
        }
    }

    fn profile(id: &str, port: u16, memory_mib: u32, enabled: bool) -> ServerProfile {
        ServerProfile {
            id: ServerId(id.into()),
            name: id.into(),
            runtime: RuntimeKind::BuiltInHttp,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port,
            memory_mib,
            network_scope: NetworkScope::Loopback,
            enabled,
        }
    }

    fn available() -> RuntimeAvailability {
        RuntimeAvailability {
            available: true,
            runtime: RuntimeKind::BuiltInHttp,
            reason: "test runtime".into(),
        }
    }

    #[test]
    fn grants_a_safe_enabled_profile() {
        let candidate = profile("alpha", 8_080, 512, true);
        let result = authorize_start(
            &capability(Some(8_192), Some(6_000)),
            std::slice::from_ref(&candidate),
            &[],
            &candidate,
            &available(),
        );
        assert!(result.is_ok());
    }

    #[test]
    fn rejects_disabled_profile_before_adapter_start() {
        let candidate = profile("alpha", 8_080, 512, false);
        let rejection = authorize_start(
            &capability(Some(8_192), Some(6_000)),
            std::slice::from_ref(&candidate),
            &[],
            &candidate,
            &available(),
        )
        .expect_err("disabled profile must be rejected");
        assert!(rejection
            .reasons
            .iter()
            .any(|reason| reason.code == StartAdmissionReasonCode::ProfileDisabled));
    }

    #[test]
    fn rejects_port_reserved_by_another_enabled_profile() {
        let candidate = profile("alpha", 8_080, 512, true);
        let other = profile("beta", 8_080, 512, true);
        let profiles = vec![candidate.clone(), other];
        let rejection = authorize_start(
            &capability(Some(8_192), Some(6_000)),
            &profiles,
            &[],
            &candidate,
            &available(),
        )
        .expect_err("conflicting port must be rejected");
        assert!(rejection
            .reasons
            .iter()
            .any(|reason| reason.code == StartAdmissionReasonCode::PortConflict));
    }

    #[test]
    fn rejects_missing_memory_telemetry_fail_closed() {
        let candidate = profile("alpha", 8_080, 512, true);
        let rejection = authorize_start(
            &capability(None, None),
            std::slice::from_ref(&candidate),
            &[],
            &candidate,
            &available(),
        )
        .expect_err("unknown memory budget must fail closed");
        assert!(rejection
            .reasons
            .iter()
            .any(|reason| { reason.code == StartAdmissionReasonCode::ResourceAllocationUnsafe }));
        assert!(rejection
            .warnings
            .iter()
            .any(|warning| warning.code == ResourceWarningCode::DeviceTelemetryUnavailable));
    }

    #[test]
    fn rejects_unavailable_runtime() {
        let candidate = profile("alpha", 8_080, 512, true);
        let rejection = authorize_start(
            &capability(Some(8_192), Some(6_000)),
            std::slice::from_ref(&candidate),
            &[],
            &candidate,
            &RuntimeAvailability {
                available: false,
                runtime: RuntimeKind::BuiltInHttp,
                reason: "adapter failed verification".into(),
            },
        )
        .expect_err("unavailable runtime must be rejected");
        assert!(rejection
            .reasons
            .iter()
            .any(|reason| reason.code == StartAdmissionReasonCode::RuntimeUnavailable));
    }

    #[test]
    fn cpu_headroom_remains_a_warning_not_a_hard_rejection() {
        let candidate = profile("alpha", 8_080, 128, true);
        let mut low_cpu = capability(Some(8_192), Some(6_000));
        low_cpu.logical_cpus = 1;
        let permit = authorize_start(
            &low_cpu,
            std::slice::from_ref(&candidate),
            &[],
            &candidate,
            &available(),
        );
        assert!(permit.is_ok());
    }
}
