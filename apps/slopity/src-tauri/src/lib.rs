#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{
    sample_profiles, validate_profile, CapabilitySnapshot, ResourcePlan, ResourcePlanner,
    RuntimeAvailability, RuntimeKind, ServerProfile, ValidationIssue,
};
use std::collections::HashSet;
use tauri::Manager;
use tauri_plugin_slopity_host::{HostServiceCapability, SlopityHostExt};

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardSnapshot {
    application: &'static str,
    platform: &'static str,
    architecture: &'static str,
    host_service: HostServiceCapability,
    runtimes: Vec<RuntimeAvailability>,
    samples: Vec<ServerProfile>,
    resource_plan: ResourcePlan,
}

#[tauri::command]
fn dashboard_snapshot(app: tauri::AppHandle) -> DashboardSnapshot {
    let capability = CapabilitySnapshot {
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        logical_cpus: std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1),
        total_memory_mib: 0,
        available_memory_mib: 0,
    };

    DashboardSnapshot {
        application: "Slopity",
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        host_service: app.slopity_host_capability(),
        runtimes: runtime_catalog(),
        samples: sample_profiles(),
        resource_plan: ResourcePlanner::plan(&capability),
    }
}

#[tauri::command]
fn validate_server_profile(profile: ServerProfile) -> Vec<ValidationIssue> {
    validate_profile(&profile, &HashSet::new())
}

fn runtime_catalog() -> Vec<RuntimeAvailability> {
    [
        RuntimeKind::Java,
        RuntimeKind::NodeJs,
        RuntimeKind::Python,
        RuntimeKind::Php,
        RuntimeKind::Native,
    ]
    .into_iter()
    .map(|runtime| RuntimeAvailability {
        available: false,
        runtime,
        reason: "No verified runtime provider is installed in the foundation build.".into(),
    })
    .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_slopity_host::init())
        .invoke_handler(tauri::generate_handler![
            dashboard_snapshot,
            validate_server_profile
        ])
        .setup(|app| {
            let _ = app.handle();
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("Slopity failed to start: {error}"));
}
