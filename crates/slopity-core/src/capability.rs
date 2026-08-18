use crate::{ServerId, ServerProfile};
use serde::{Deserialize, Serialize};
use std::collections::{BTreeMap, HashSet};

const ANDROID_HOST_RESERVE_MIB: u64 = 2_048;
const WINDOWS_HOST_RESERVE_MIB: u64 = 1_536;
const DEFAULT_HOST_RESERVE_MIB: u64 = 1_024;
const SAFE_BUDGET_PERCENT: u64 = 70;
const SERVER_MEMORY_UNIT_MIB: u64 = 768;
const MAX_RECOMMENDED_SERVERS: usize = 8;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySnapshot {
    pub platform: String,
    pub architecture: String,
    pub logical_cpus: usize,
    pub total_memory_mib: Option<u64>,
    pub available_memory_mib: Option<u64>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum ResourceWarningCode {
    DeviceTelemetryUnavailable,
    InsufficientAvailableMemory,
    RequestedMemoryExceedsSafeBudget,
    LowCpuHeadroom,
    ConflictingPort,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceWarning {
    pub code: ResourceWarningCode,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePlan {
    pub host_reserve_mib: u64,
    pub safe_server_budget_mib: Option<u64>,
    pub available_server_headroom_mib: Option<u64>,
    pub recommended_concurrent_servers: usize,
    pub warnings: Vec<ResourceWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct PortReservation {
    pub port: u16,
    pub server_ids: Vec<ServerId>,
    pub conflict: bool,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourceAccountingSnapshot {
    pub reserved_memory_mib: u64,
    pub remaining_safe_budget_mib: Option<u64>,
    pub active_or_reserved_server_count: usize,
    pub port_reservations: Vec<PortReservation>,
    pub warnings: Vec<ResourceWarning>,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct AllocationDecision {
    pub fits: bool,
    pub requested_memory_mib: u64,
    pub remaining_safe_budget_mib: Option<u64>,
    pub conflicting_server_ids: Vec<ServerId>,
    pub warnings: Vec<ResourceWarning>,
}

pub struct ResourcePlanner;

impl ResourcePlanner {
    pub fn plan(snapshot: &CapabilitySnapshot) -> ResourcePlan {
        let total_memory_mib = known_memory(snapshot.total_memory_mib);
        let available_memory_mib = known_memory(snapshot.available_memory_mib);
        let host_reserve_mib = host_reserve(snapshot.platform.as_str(), total_memory_mib);
        let safe_server_budget_mib =
            total_memory_mib.map(|total| safe_fraction(total.saturating_sub(host_reserve_mib)));
        let available_server_headroom_mib = available_memory_mib
            .map(|available| safe_fraction(available.saturating_sub(host_reserve_mib)));

        let cpu_limit = cpu_server_limit(snapshot.logical_cpus);
        let memory_limit = safe_server_budget_mib
            .map(|budget| usize::try_from(budget / SERVER_MEMORY_UNIT_MIB).unwrap_or(usize::MAX))
            .unwrap_or(0);
        let recommended_concurrent_servers =
            cpu_limit.min(memory_limit).min(MAX_RECOMMENDED_SERVERS);

        let mut warnings = Vec::new();
        if total_memory_mib.is_none() || available_memory_mib.is_none() {
            warnings.push(warning(
                ResourceWarningCode::DeviceTelemetryUnavailable,
                "Total or available memory telemetry is unavailable; Slopity cannot prove a safe memory allocation.",
            ));
        }
        if available_memory_mib.is_some_and(|available| available <= host_reserve_mib) {
            warnings.push(warning(
                ResourceWarningCode::InsufficientAvailableMemory,
                "Currently available memory does not exceed the protected host reserve.",
            ));
        }
        if snapshot.logical_cpus == 0 {
            warnings.push(warning(
                ResourceWarningCode::DeviceTelemetryUnavailable,
                "Logical CPU telemetry is unavailable; Slopity cannot recommend concurrency.",
            ));
        } else if snapshot.logical_cpus < 4 {
            warnings.push(warning(
                ResourceWarningCode::LowCpuHeadroom,
                "Low CPU headroom; prefer one lightweight server and watch host responsiveness.",
            ));
        }

        ResourcePlan {
            host_reserve_mib,
            safe_server_budget_mib,
            available_server_headroom_mib,
            recommended_concurrent_servers,
            warnings,
        }
    }
}

pub struct ResourceAccounting;

impl ResourceAccounting {
    pub fn summarize(
        snapshot: &CapabilitySnapshot,
        profiles: &[ServerProfile],
        active_server_ids: &[ServerId],
    ) -> ResourceAccountingSnapshot {
        let plan = ResourcePlanner::plan(snapshot);
        let reserved = reserved_profiles(profiles, active_server_ids, None);
        let reserved_memory_mib = reserved.iter().fold(0_u64, |total, profile| {
            total.saturating_add(u64::from(profile.memory_mib))
        });
        let port_reservations = port_reservations(&reserved);
        let remaining_safe_budget_mib = plan
            .safe_server_budget_mib
            .map(|budget| budget.saturating_sub(reserved_memory_mib));
        let mut warnings = plan.warnings.clone();

        if plan
            .safe_server_budget_mib
            .is_some_and(|budget| reserved_memory_mib > budget)
        {
            warnings.push(warning(
                ResourceWarningCode::RequestedMemoryExceedsSafeBudget,
                format!(
                    "Reserved server memory ({reserved_memory_mib} MiB) exceeds the safe server budget."
                ),
            ));
        }
        if plan
            .available_server_headroom_mib
            .is_some_and(|headroom| reserved_memory_mib > headroom)
        {
            warnings.push(warning(
                ResourceWarningCode::InsufficientAvailableMemory,
                "Current available-memory headroom is below the aggregate reserved server memory; starting every reservation now would be unsafe.",
            ));
        }
        let cpu_limit = cpu_server_limit(snapshot.logical_cpus);
        if snapshot.logical_cpus > 0 && reserved.len() > cpu_limit {
            warnings.push(warning(
                ResourceWarningCode::LowCpuHeadroom,
                format!(
                    "{} servers are active or reserved, above the CPU headroom limit of {}.",
                    reserved.len(),
                    cpu_limit
                ),
            ));
        }
        for reservation in port_reservations
            .iter()
            .filter(|reservation| reservation.conflict)
        {
            warnings.push(warning(
                ResourceWarningCode::ConflictingPort,
                format!(
                    "Port {} is reserved by multiple servers: {}.",
                    reservation.port,
                    reservation
                        .server_ids
                        .iter()
                        .map(|id| id.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }

        ResourceAccountingSnapshot {
            reserved_memory_mib,
            remaining_safe_budget_mib,
            active_or_reserved_server_count: reserved.len(),
            port_reservations,
            warnings,
        }
    }

    pub fn evaluate_allocation(
        snapshot: &CapabilitySnapshot,
        profiles: &[ServerProfile],
        active_server_ids: &[ServerId],
        candidate: &ServerProfile,
    ) -> AllocationDecision {
        let plan = ResourcePlanner::plan(snapshot);
        let reserved = reserved_profiles(profiles, active_server_ids, Some(&candidate.id));
        let existing_memory_mib = reserved.iter().fold(0_u64, |total, profile| {
            total.saturating_add(u64::from(profile.memory_mib))
        });
        let candidate_memory_mib = u64::from(candidate.memory_mib);
        let requested_memory_mib = existing_memory_mib.saturating_add(candidate_memory_mib);
        let remaining_safe_budget_mib = plan
            .safe_server_budget_mib
            .map(|budget| budget.saturating_sub(requested_memory_mib));
        let mut conflicting_server_ids = reserved
            .iter()
            .filter(|profile| profile.port == candidate.port)
            .map(|profile| profile.id.clone())
            .collect::<Vec<_>>();
        conflicting_server_ids.sort_by(|left, right| left.0.cmp(&right.0));
        let mut warnings = plan.warnings.clone();

        let within_safe_budget = plan
            .safe_server_budget_mib
            .is_some_and(|budget| requested_memory_mib <= budget);
        if !within_safe_budget {
            warnings.push(warning(
                ResourceWarningCode::RequestedMemoryExceedsSafeBudget,
                match plan.safe_server_budget_mib {
                    Some(budget) => format!(
                        "Allocating {} MiB would reserve {} MiB in total, above the safe budget of {} MiB.",
                        candidate_memory_mib, requested_memory_mib, budget
                    ),
                    None => "A safe server memory budget cannot be calculated until memory telemetry is available.".into(),
                },
            ));
        }

        let within_available_headroom = plan
            .available_server_headroom_mib
            .is_some_and(|headroom| candidate_memory_mib <= headroom);
        if !within_available_headroom {
            warnings.push(warning(
                ResourceWarningCode::InsufficientAvailableMemory,
                match plan.available_server_headroom_mib {
                    Some(headroom) => format!(
                        "The server requests {} MiB, but only {} MiB of immediate safe memory headroom is available.",
                        candidate_memory_mib, headroom
                    ),
                    None => "Available-memory telemetry is missing, so immediate allocation headroom cannot be verified.".into(),
                },
            ));
        }

        let requested_server_count = reserved.len().saturating_add(1);
        if snapshot.logical_cpus > 0
            && requested_server_count > cpu_server_limit(snapshot.logical_cpus)
        {
            warnings.push(warning(
                ResourceWarningCode::LowCpuHeadroom,
                format!(
                    "Allocating this server would reserve {requested_server_count} servers on a host with {} logical CPUs.",
                    snapshot.logical_cpus
                ),
            ));
        }

        if !conflicting_server_ids.is_empty() {
            warnings.push(warning(
                ResourceWarningCode::ConflictingPort,
                format!(
                    "Port {} is already reserved by {}.",
                    candidate.port,
                    conflicting_server_ids
                        .iter()
                        .map(|id| id.0.as_str())
                        .collect::<Vec<_>>()
                        .join(", ")
                ),
            ));
        }

        AllocationDecision {
            fits: within_safe_budget
                && within_available_headroom
                && conflicting_server_ids.is_empty(),
            requested_memory_mib,
            remaining_safe_budget_mib,
            conflicting_server_ids,
            warnings,
        }
    }
}

fn cpu_server_limit(logical_cpus: usize) -> usize {
    if logical_cpus == 0 {
        0
    } else {
        logical_cpus.saturating_sub(1).max(1)
    }
}

fn known_memory(value: Option<u64>) -> Option<u64> {
    value.filter(|value| *value > 0)
}

fn host_reserve(platform: &str, total_memory_mib: Option<u64>) -> u64 {
    let policy_reserve = match platform {
        "android" => ANDROID_HOST_RESERVE_MIB,
        "windows" => WINDOWS_HOST_RESERVE_MIB,
        _ => DEFAULT_HOST_RESERVE_MIB,
    };
    total_memory_mib
        .map(|total| policy_reserve.min(total))
        .unwrap_or(policy_reserve)
}

fn safe_fraction(memory_mib: u64) -> u64 {
    memory_mib.saturating_mul(SAFE_BUDGET_PERCENT) / 100
}

fn reserved_profiles<'a>(
    profiles: &'a [ServerProfile],
    active_server_ids: &[ServerId],
    excluded_id: Option<&ServerId>,
) -> Vec<&'a ServerProfile> {
    let active = active_server_ids
        .iter()
        .map(|id| id.0.as_str())
        .collect::<HashSet<_>>();
    let mut seen = HashSet::new();
    let mut reserved = profiles
        .iter()
        .filter(|profile| excluded_id != Some(&profile.id))
        .filter(|profile| profile.enabled || active.contains(profile.id.0.as_str()))
        .filter(|profile| seen.insert(profile.id.0.as_str()))
        .collect::<Vec<_>>();
    reserved.sort_by(|left, right| left.id.0.cmp(&right.id.0));
    reserved
}

fn port_reservations(profiles: &[&ServerProfile]) -> Vec<PortReservation> {
    let mut ports = BTreeMap::<u16, Vec<ServerId>>::new();
    for profile in profiles {
        ports
            .entry(profile.port)
            .or_default()
            .push(profile.id.clone());
    }
    ports
        .into_iter()
        .map(|(port, mut server_ids)| {
            server_ids.sort_by(|left, right| left.0.cmp(&right.0));
            PortReservation {
                port,
                conflict: server_ids.len() > 1,
                server_ids,
            }
        })
        .collect()
}

fn warning(code: ResourceWarningCode, message: impl Into<String>) -> ResourceWarning {
    ResourceWarning {
        code,
        message: message.into(),
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, RuntimeKind};

    fn snapshot(
        platform: &str,
        logical_cpus: usize,
        total_memory_mib: Option<u64>,
        available_memory_mib: Option<u64>,
    ) -> CapabilitySnapshot {
        CapabilitySnapshot {
            platform: platform.into(),
            architecture: "test-arch".into(),
            logical_cpus,
            total_memory_mib,
            available_memory_mib,
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

    fn has_warning(warnings: &[ResourceWarning], code: ResourceWarningCode) -> bool {
        warnings.iter().any(|warning| warning.code == code)
    }

    #[test]
    fn android_reserves_memory_for_the_os() {
        let plan = ResourcePlanner::plan(&snapshot("android", 8, Some(8_192), Some(6_000)));

        assert_eq!(plan.host_reserve_mib, 2_048);
        assert_eq!(plan.safe_server_budget_mib, Some(4_300));
        assert_eq!(plan.available_server_headroom_mib, Some(2_766));
        assert_eq!(plan.recommended_concurrent_servers, 5);
    }

    #[test]
    fn low_memory_device_has_no_immediate_headroom() {
        let plan = ResourcePlanner::plan(&snapshot("linux", 2, Some(2_048), Some(700)));

        assert_eq!(plan.host_reserve_mib, 1_024);
        assert_eq!(plan.safe_server_budget_mib, Some(716));
        assert_eq!(plan.available_server_headroom_mib, Some(0));
        assert!(has_warning(
            &plan.warnings,
            ResourceWarningCode::InsufficientAvailableMemory
        ));
        assert!(has_warning(
            &plan.warnings,
            ResourceWarningCode::LowCpuHeadroom
        ));
    }

    #[test]
    fn unknown_memory_telemetry_never_claims_a_budget() {
        let capability = snapshot("linux", 8, None, None);
        let plan = ResourcePlanner::plan(&capability);
        let candidate = profile("alpha", 3_000, 512, true);
        let decision = ResourceAccounting::evaluate_allocation(&capability, &[], &[], &candidate);

        assert_eq!(plan.safe_server_budget_mib, None);
        assert_eq!(plan.available_server_headroom_mib, None);
        assert!(!decision.fits);
        assert!(has_warning(
            &decision.warnings,
            ResourceWarningCode::DeviceTelemetryUnavailable
        ));
    }

    #[test]
    fn one_server_allocation_is_reserved() {
        let capability = snapshot("linux", 8, Some(8_192), Some(6_000));
        let profiles = [profile("alpha", 3_000, 512, true)];
        let accounting = ResourceAccounting::summarize(&capability, &profiles, &[]);

        assert_eq!(accounting.reserved_memory_mib, 512);
        assert_eq!(accounting.remaining_safe_budget_mib, Some(4_505));
        assert_eq!(accounting.active_or_reserved_server_count, 1);
        assert_eq!(accounting.port_reservations.len(), 1);
    }

    #[test]
    fn multiple_server_allocations_are_aggregated() {
        let capability = snapshot("linux", 8, Some(8_192), Some(6_000));
        let profiles = [
            profile("alpha", 3_000, 512, true),
            profile("beta", 3_001, 1_024, true),
        ];
        let accounting = ResourceAccounting::summarize(&capability, &profiles, &[]);

        assert_eq!(accounting.reserved_memory_mib, 1_536);
        assert_eq!(accounting.remaining_safe_budget_mib, Some(3_481));
        assert_eq!(accounting.active_or_reserved_server_count, 2);
    }

    #[test]
    fn allocation_exactly_at_budget_fits() {
        let capability = snapshot("linux", 8, Some(6_144), Some(6_144));
        let profiles = [profile("alpha", 3_000, 2_048, true)];
        let candidate = profile("beta", 3_001, 1_536, true);
        let decision =
            ResourceAccounting::evaluate_allocation(&capability, &profiles, &[], &candidate);

        assert_eq!(decision.requested_memory_mib, 3_584);
        assert_eq!(decision.remaining_safe_budget_mib, Some(0));
        assert!(decision.fits);
    }

    #[test]
    fn allocation_above_budget_is_rejected() {
        let capability = snapshot("linux", 8, Some(6_144), Some(6_144));
        let profiles = [profile("alpha", 3_000, 2_048, true)];
        let candidate = profile("beta", 3_001, 1_537, true);
        let decision =
            ResourceAccounting::evaluate_allocation(&capability, &profiles, &[], &candidate);

        assert!(!decision.fits);
        assert!(has_warning(
            &decision.warnings,
            ResourceWarningCode::RequestedMemoryExceedsSafeBudget
        ));
    }

    #[test]
    fn duplicate_port_reservations_are_reported_deterministically() {
        let capability = snapshot("linux", 8, Some(8_192), Some(6_000));
        let profiles = [
            profile("beta", 3_000, 512, true),
            profile("alpha", 3_000, 512, true),
        ];
        let accounting = ResourceAccounting::summarize(&capability, &profiles, &[]);
        let candidate = profile("gamma", 3_000, 512, true);
        let decision =
            ResourceAccounting::evaluate_allocation(&capability, &profiles, &[], &candidate);

        assert!(accounting.port_reservations[0].conflict);
        assert_eq!(
            accounting.port_reservations[0]
                .server_ids
                .iter()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
        assert!(!decision.fits);
        assert_eq!(
            decision
                .conflicting_server_ids
                .iter()
                .map(|id| id.0.as_str())
                .collect::<Vec<_>>(),
            vec!["alpha", "beta"]
        );
    }

    #[test]
    fn remaining_memory_uses_saturating_subtraction() {
        let capability = snapshot("linux", 8, Some(2_048), Some(2_048));
        let profiles = [profile("alpha", 3_000, 1_024, true)];
        let accounting = ResourceAccounting::summarize(&capability, &profiles, &[]);

        assert_eq!(accounting.remaining_safe_budget_mib, Some(0));
        assert!(has_warning(
            &accounting.warnings,
            ResourceWarningCode::RequestedMemoryExceedsSafeBudget
        ));
    }

    #[test]
    fn disabled_active_server_is_still_reserved() {
        let capability = snapshot("linux", 8, Some(8_192), Some(6_000));
        let profiles = [profile("alpha", 3_000, 512, false)];
        let active = [ServerId("alpha".into())];
        let accounting = ResourceAccounting::summarize(&capability, &profiles, &active);

        assert_eq!(accounting.reserved_memory_mib, 512);
        assert_eq!(accounting.active_or_reserved_server_count, 1);
    }

    #[test]
    fn cpu_constrained_device_returns_actionable_warning() {
        let plan = ResourcePlanner::plan(&snapshot("linux", 2, Some(8_192), Some(6_000)));

        assert_eq!(plan.recommended_concurrent_servers, 1);
        assert!(has_warning(
            &plan.warnings,
            ResourceWarningCode::LowCpuHeadroom
        ));
    }
}
