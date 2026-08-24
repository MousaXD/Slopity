use crate::{NetworkScope, RuntimeKind, ServerProfile};
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

pub const MAX_PROFILE_ID_LENGTH: usize = 128;
pub const MAX_PROFILE_NAME_LENGTH: usize = 256;
pub const MAX_ARGUMENT_COUNT: usize = 256;
pub const MAX_ARGUMENT_LENGTH: usize = 8_192;
pub const MAX_ARGUMENT_PAYLOAD_SIZE: usize = 65_536;
pub const MAX_EXECUTABLE_PATH_LENGTH: usize = 4_096;
pub const MAX_WORKING_DIRECTORY_PATH_LENGTH: usize = 4_096;

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

pub fn is_valid_profile_id(id: &str) -> bool {
    !id.is_empty()
        && id.len() <= MAX_PROFILE_ID_LENGTH
        && id
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
}

pub fn validate_profile(
    profile: &ServerProfile,
    reserved_ports: &HashSet<u16>,
) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if profile.id.0.trim().is_empty() {
        issues.push(error("profile-id-empty", "Profile ID cannot be empty."));
    } else {
        if profile.id.0.len() > MAX_PROFILE_ID_LENGTH {
            issues.push(error(
                "profile-id-too-long",
                &format!("Profile ID must not exceed {MAX_PROFILE_ID_LENGTH} characters."),
            ));
        }
        if !profile
            .id
            .0
            .chars()
            .all(|c| c.is_ascii_alphanumeric() || c == '.' || c == '_' || c == '-')
        {
            issues.push(error(
                "profile-id-invalid",
                "Profile ID may only contain ASCII alphanumeric characters, '.', '_', and '-'.",
            ));
        }
    }

    if profile.name.trim().is_empty() {
        issues.push(error("profile-name-empty", "Profile name cannot be empty."));
    }
    if profile.name.len() > MAX_PROFILE_NAME_LENGTH {
        issues.push(error(
            "profile-name-too-long",
            &format!("Profile name cannot exceed {MAX_PROFILE_NAME_LENGTH} UTF-8 bytes."),
        ));
    }

    if profile.arguments.len() > MAX_ARGUMENT_COUNT {
        issues.push(error(
            "too-many-arguments",
            &format!("A profile cannot contain more than {MAX_ARGUMENT_COUNT} arguments."),
        ));
    }

    let mut argument_payload_size = 0usize;
    for (index, argument) in profile.arguments.iter().enumerate() {
        argument_payload_size = argument_payload_size.saturating_add(argument.len());
        if argument.len() > MAX_ARGUMENT_LENGTH {
            issues.push(error(
                "argument-too-long",
                &format!("Argument {index} cannot exceed {MAX_ARGUMENT_LENGTH} UTF-8 bytes."),
            ));
        }
    }
    if argument_payload_size > MAX_ARGUMENT_PAYLOAD_SIZE {
        issues.push(error(
            "argument-payload-too-large",
            &format!(
                "The combined argument payload cannot exceed {MAX_ARGUMENT_PAYLOAD_SIZE} UTF-8 bytes."
            ),
        ));
    }

    if let Some(executable) = &profile.executable {
        if executable.to_string_lossy().len() > MAX_EXECUTABLE_PATH_LENGTH {
            issues.push(error(
                "executable-path-too-long",
                &format!(
                    "Executable path cannot exceed {MAX_EXECUTABLE_PATH_LENGTH} UTF-8 bytes after portable path conversion."
                ),
            ));
        }
    }
    if let Some(working_directory) = &profile.working_directory {
        if working_directory.to_string_lossy().len() > MAX_WORKING_DIRECTORY_PATH_LENGTH {
            issues.push(error(
                "working-directory-path-too-long",
                &format!(
                    "Working-directory path cannot exceed {MAX_WORKING_DIRECTORY_PATH_LENGTH} UTF-8 bytes after portable path conversion."
                ),
            ));
        }
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

    if profile.runtime == RuntimeKind::BuiltInHttp {
        if profile.executable.is_some() {
            issues.push(warning(
                "built-in-runtime-ignores-executable",
                "The built-in HTTP runtime does not execute the configured executable path.",
            ));
        }
        if !profile.arguments.is_empty() {
            issues.push(warning(
                "built-in-runtime-ignores-arguments",
                "The built-in HTTP runtime does not execute profile arguments.",
            ));
        }
    } else if profile.executable.is_none() {
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
    use std::path::PathBuf;

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

    fn has_error(issues: &[ValidationIssue], code: &str) -> bool {
        issues
            .iter()
            .any(|issue| issue.code == code && issue.severity == ValidationSeverity::Error)
    }

    #[test]
    fn missing_runtime_is_a_warning_not_a_false_success() {
        let issues = validate_profile(&profile(), &HashSet::new());
        assert!(issues
            .iter()
            .any(|issue| issue.code == "runtime-not-installed"));
    }

    #[test]
    fn built_in_http_does_not_require_an_executable() {
        let mut built_in = profile();
        built_in.runtime = RuntimeKind::BuiltInHttp;
        let issues = validate_profile(&built_in, &HashSet::new());
        assert!(!issues
            .iter()
            .any(|issue| issue.code == "runtime-not-installed"));
    }

    #[test]
    fn reserved_port_is_an_error() {
        let issues = validate_profile(&profile(), &HashSet::from([8_080]));
        assert!(has_error(&issues, "port-conflict"));
    }

    #[test]
    fn profile_id_length_is_enforced() {
        let mut candidate = profile();
        candidate.id = ServerId("x".repeat(MAX_PROFILE_ID_LENGTH + 1));
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "profile-id-too-long"));
    }

    #[test]
    fn profile_name_length_is_enforced() {
        let mut candidate = profile();
        candidate.name = "x".repeat(MAX_PROFILE_NAME_LENGTH + 1);
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "profile-name-too-long"));
    }

    #[test]
    fn argument_count_is_enforced() {
        let mut candidate = profile();
        candidate.arguments = vec!["x".into(); MAX_ARGUMENT_COUNT + 1];
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "too-many-arguments"));
    }

    #[test]
    fn individual_argument_length_is_enforced() {
        let mut candidate = profile();
        candidate.arguments = vec!["x".repeat(MAX_ARGUMENT_LENGTH + 1)];
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "argument-too-long"));
    }

    #[test]
    fn total_argument_payload_is_enforced() {
        let mut candidate = profile();
        candidate.arguments = vec!["x".repeat(MAX_ARGUMENT_LENGTH); 9];
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "argument-payload-too-large"));
        assert!(!has_error(&issues, "argument-too-long"));
    }

    #[test]
    fn executable_path_length_is_enforced() {
        let mut candidate = profile();
        candidate.executable = Some(PathBuf::from("x".repeat(MAX_EXECUTABLE_PATH_LENGTH + 1)));
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "executable-path-too-long"));
    }

    #[test]
    fn working_directory_path_length_is_enforced() {
        let mut candidate = profile();
        candidate.working_directory = Some(PathBuf::from(
            "x".repeat(MAX_WORKING_DIRECTORY_PATH_LENGTH + 1),
        ));
        let issues = validate_profile(&candidate, &HashSet::new());
        assert!(has_error(&issues, "working-directory-path-too-long"));
    }

    #[test]
    fn valid_profile_ids() {
        let valid_ids = [
            "a",
            "server-1",
            "server_2",
            "my.server.3",
            "SERVER-A_b.1",
            &"a".repeat(128),
        ];

        for id in valid_ids {
            assert!(is_valid_profile_id(id), "expected valid: {id}");
            let mut p = profile();
            p.id = ServerId(id.to_string());
            let issues = validate_profile(&p, &HashSet::new());
            assert!(
                !issues.iter().any(|i| i.code == "profile-id-empty"
                    || i.code == "profile-id-too-long"
                    || i.code == "profile-id-invalid"),
                "expected no profile-id error for: {id}"
            );
        }
    }

    #[test]
    fn empty_and_whitespace_profile_ids_are_rejected() {
        for empty in ["", "   ", "\t", "\n"] {
            assert!(!is_valid_profile_id(empty));
            let mut p = profile();
            p.id = ServerId(empty.to_string());
            let issues = validate_profile(&p, &HashSet::new());
            assert!(
                issues.iter().any(|i| i.code == "profile-id-empty"),
                "expected profile-id-empty for: {empty:?}"
            );
        }
    }

    #[test]
    fn oversized_profile_id_is_rejected() {
        let oversized = "a".repeat(129);
        assert!(!is_valid_profile_id(&oversized));
        let mut p = profile();
        p.id = ServerId(oversized);
        let issues = validate_profile(&p, &HashSet::new());
        assert!(issues.iter().any(|i| i.code == "profile-id-too-long"));
    }

    #[test]
    fn profile_id_with_nul_byte_is_rejected() {
        let with_nul = "server\0crash";
        assert!(!is_valid_profile_id(with_nul));
        let mut p = profile();
        p.id = ServerId(with_nul.to_string());
        let issues = validate_profile(&p, &HashSet::new());
        assert!(issues.iter().any(|i| i.code == "profile-id-invalid"));
    }

    #[test]
    fn profile_id_with_control_chars_is_rejected() {
        for invalid in [
            "server\nname",
            "server\rname",
            "server\tname",
            "\x1b[31mred",
        ] {
            assert!(!is_valid_profile_id(invalid));
            let mut p = profile();
            p.id = ServerId(invalid.to_string());
            let issues = validate_profile(&p, &HashSet::new());
            assert!(
                issues.iter().any(|i| i.code == "profile-id-invalid"),
                "expected profile-id-invalid for: {invalid:?}"
            );
        }
    }

    #[test]
    fn profile_id_with_spaces_or_path_traversal_is_rejected() {
        for invalid in [
            "server name",
            "../etc/passwd",
            "foo/bar",
            "foo\\bar",
            "server:1",
            "server;rm",
        ] {
            assert!(!is_valid_profile_id(invalid));
            let mut p = profile();
            p.id = ServerId(invalid.to_string());
            let issues = validate_profile(&p, &HashSet::new());
            assert!(
                issues.iter().any(|i| i.code == "profile-id-invalid"),
                "expected profile-id-invalid for: {invalid:?}"
            );
        }
    }
}
