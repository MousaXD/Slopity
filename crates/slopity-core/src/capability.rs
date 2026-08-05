use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct CapabilitySnapshot {
    pub platform: String,
    pub architecture: String,
    pub logical_cpus: usize,
    pub total_memory_mib: u64,
    pub available_memory_mib: u64,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ResourcePlan {
    pub host_reserve_mib: u64,
    pub safe_server_budget_mib: u64,
    pub recommended_concurrent_servers: usize,
    pub warning: Option<String>,
}

pub struct ResourcePlanner;

impl ResourcePlanner {
    pub fn plan(snapshot: &CapabilitySnapshot) -> ResourcePlan {
        let host_reserve_mib = match snapshot.platform.as_str() {
            "android" => 2_048,
            "windows" => 1_536,
            _ => 1_024,
        }
        .min(snapshot.total_memory_mib);

        let memory_after_reserve = snapshot
            .available_memory_mib
            .saturating_sub(host_reserve_mib);
        let safe_server_budget_mib = memory_after_reserve.saturating_mul(70) / 100;
        let cpu_limit = snapshot.logical_cpus.saturating_sub(1).max(1);
        let memory_limit = usize::try_from(safe_server_budget_mib / 768).unwrap_or(usize::MAX);
        let recommended_concurrent_servers = cpu_limit.min(memory_limit).clamp(0, 8);

        let warning = if snapshot.available_memory_mib <= host_reserve_mib {
            Some("Not enough currently available memory for a safe server allocation.".into())
        } else if snapshot.logical_cpus < 4 {
            Some("Low CPU headroom; prefer one lightweight server.".into())
        } else {
            None
        };

        ResourcePlan {
            host_reserve_mib,
            safe_server_budget_mib,
            recommended_concurrent_servers,
            warning,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn android_reserves_memory_for_the_os() {
        let plan = ResourcePlanner::plan(&CapabilitySnapshot {
            platform: "android".into(),
            architecture: "aarch64".into(),
            logical_cpus: 8,
            total_memory_mib: 8_192,
            available_memory_mib: 6_000,
        });

        assert_eq!(plan.host_reserve_mib, 2_048);
        assert!(plan.safe_server_budget_mib < 4_000);
        assert!(plan.recommended_concurrent_servers <= 5);
    }

    #[test]
    fn no_budget_when_available_memory_is_below_reserve() {
        let plan = ResourcePlanner::plan(&CapabilitySnapshot {
            platform: "linux".into(),
            architecture: "x86_64".into(),
            logical_cpus: 2,
            total_memory_mib: 2_048,
            available_memory_mib: 500,
        });

        assert_eq!(plan.safe_server_budget_mib, 0);
        assert_eq!(plan.recommended_concurrent_servers, 0);
        assert!(plan.warning.is_some());
    }
}
