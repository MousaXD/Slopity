use crate::{NetworkScope, ServerProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ValidationSeverity {
    Error,
    Warning,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ValidationIssue {
    pub code: String,
    pub severity: ValidationSeverity,
    pub message: String,
}

pub fn validate_profile(
    profile: &ServerProfile,
    reserved_ports: &HashSet<u16>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if profile.id.0.trim().is_empty() {
        issues.push(error("profile-id-empty", "Profile ID cannot be empty."));
    }
    if profile.name.trim().is_empty() {
        issues.push(error("profile-name-empty", "Profile name cannot be empty."));
    }
    if profile.port == 0 {
        issues.push(error(
            "port-zero",
            "Port 0 is not accepted for persisted server profiles.",
        ));
    } else if reserved_ports.contains(&profile.port) {
        issues.push(error(
            "port-conflict",
            "The selected port is already reserved by another profile.",
        ));
    }
    if profile.memory_mib < 128 {
        issues.push(error("memory-too-low", "Allocate at least 128 MiB."));
    }
    if profile.memory_mib > 32_768 {
        issues.push(warning(
            "memory-unusually-high",
            "The requested memory budget is unusually high.",
        ));
    }
    if profile.executable.is_none() {
        issues.push(warning(
            "runtime-not-installed",
            "No executable/runtime provider is configured; this profile is not runnable yet.",
        ));
    }
    if matches!(profile.network_scope, NetworkScope::Lan) {
        issues.push(warning(
            "lan-exposure",
            "LAN exposure allows other devices on the local network to connect.",
        ));
    }

    issues
}

fn error(code: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        severity: ValidationSeverity::Error,
        message: message.into(),
    }
}

fn warning(code: &str, message: &str) -> ValidationIssue {
    ValidationIssue {
        code: code.into(),
        severity: ValidationSeverity::Warning,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, RuntimeKind, ServerId};

    fn profile() -> ServerProfile {
        ServerProfile {
            id: ServerId("test".into()),
            name: "Test".into(),
            runtime: RuntimeKind::Native,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port: 8_080,
            memory_mib: 512,
            network_scope: NetworkScope::Loopback,
            enabled: false,
        }
    }

    #[test]
    fn missing_runtime_is_a_warning_not_a_false_success() {
        let issues = validate_profile(&profile(), &HashSet::new());
        assert!(issues
            .iter()
            .any(|issue| issue.code == "runtime-not-installed"));
    }

    #[test]
    fn reserved_port_is_an_error() {
        let issues = validate_profile(&profile(), &HashSet::from([8_080]));
        assert!(issues.iter().any(|issue| issue.code == "port-conflict"));
    }
}
