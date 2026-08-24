#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{
    sample_profiles, CapabilitySnapshot, ProfileRecoveryNotice, ProfileStore, ResourceAccounting,
    ResourceAccountingSnapshot, ResourcePlan, ResourcePlanner, RuntimeAvailability, RuntimeKind,
    ServerId, ServerOrchestrator, ServerProfile, ServerRuntimeSnapshot, ServerState,
    ValidationIssue, PROFILE_SCHEMA_VERSION,
};
use slopity_runtime_http::HttpServerManager;
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_slopity_host::{
    HostDeviceTelemetry, HostServiceCapability, HostServiceStatus, SlopityHostExt,
};

type SharedProfileStore = Mutex<ProfileStore>;
type SharedOrchestrator = Mutex<ServerOrchestrator>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardSnapshot {
    application: &'static str,
    platform: &'static str,
    architecture: &'static str,
    host_service: HostServiceCapability,
    host_service_status: HostServiceStatus,
    device_telemetry: HostDeviceTelemetry,
    capability: CapabilitySnapshot,
    runtimes: Vec<RuntimeAvailability>,
    profiles: Vec<ServerProfile>,
    profile_recovery_notices: Vec<ProfileRecoveryNotice>,
    servers: Vec<ServerRuntimeSnapshot>,
    profile_schema_version: u32,
    resource_plan: ResourcePlan,
    resource_accounting: ResourceAccountingSnapshot,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum HostReconciliationAction {
    None,
    StopHost,
    StartHost,
    UpdateHost,
    StopServersAndHost,
}

#[tauri::command]
fn dashboard_snapshot(
    app: tauri::AppHandle,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<DashboardSnapshot, String> {
    let device_telemetry = app.slopity_host_telemetry().unwrap_or_else(|error| {
        HostDeviceTelemetry::unavailable(
            std::env::consts::OS,
            format!("host telemetry unavailable: {error}"),
        )
    });
    let capability = CapabilitySnapshot {
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        logical_cpus: std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(0),
        total_memory_mib: device_telemetry.total_memory_mib,
        available_memory_mib: device_telemetry.available_memory_mib,
    };
    let (profiles, profile_recovery_notices) = {
        let store = store
            .lock()
            .map_err(|_| "profile store lock is poisoned".to_string())?;
        (store.profiles().to_vec(), store.recovery_notices().to_vec())
    };

    let host_service_status = reconcile_host_service(&app, &orchestrator)?;
    let (server_snapshots, runtimes) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let server_snapshots = orchestrator.snapshots();
        let runtimes = runtime_catalog(&orchestrator);
        (server_snapshots, runtimes)
    };
    let active_server_ids = server_snapshots
        .iter()
        .filter(|server| is_active_state(server.state))
        .map(|server| server.server_id.clone())
        .collect::<Vec<_>>();
    let resource_plan = ResourcePlanner::plan(&capability);
    let resource_accounting =
        ResourceAccounting::summarize(&capability, &profiles, &active_server_ids);

    Ok(DashboardSnapshot {
        application: "Slopity",
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        host_service: app.slopity_host_capability(),
        host_service_status,
        device_telemetry,
        capability,
        runtimes,
        profiles,
        profile_recovery_notices,
        servers: server_snapshots,
        profile_schema_version: PROFILE_SCHEMA_VERSION,
        resource_plan,
        resource_accounting,
    })
}

#[tauri::command]
fn host_service_status(
    app: tauri::AppHandle,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<HostServiceStatus, String> {
    reconcile_host_service(&app, &orchestrator)
}

#[tauri::command]
fn list_server_profiles(
    store: State<'_, SharedProfileStore>,
) -> Result<Vec<ServerProfile>, String> {
    let store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn validate_server_profile(
    profile: ServerProfile,
    store: State<'_, SharedProfileStore>,
) -> Result<Vec<ValidationIssue>, String> {
    let store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    Ok(store.validation_issues(&profile))
}

#[tauri::command]
fn create_server_profile(
    profile: ServerProfile,
    store: State<'_, SharedProfileStore>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    store.create(profile).map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn update_server_profile(
    profile: ServerProfile,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&profile.id, &orchestrator)?;
    store.update(profile).map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn clone_server_profile(
    source_id: ServerId,
    new_id: ServerId,
    new_name: String,
    store: State<'_, SharedProfileStore>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    store
        .clone_profile(&source_id, new_id, new_name)
        .map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn set_server_profile_enabled(
    id: ServerId,
    enabled: bool,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&id, &orchestrator)?;
    store
        .set_enabled(&id, enabled)
        .map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn delete_server_profile(
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&id, &orchestrator)?;
    store.delete(&id).map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn list_builtin_http_servers(
    app: tauri::AppHandle,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerRuntimeSnapshot>, String> {
    reconcile_host_service(&app, &orchestrator)?;
    let mut orchestrator = orchestrator
        .lock()
        .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
    Ok(orchestrator.snapshots())
}

#[tauri::command]
fn start_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<ServerRuntimeSnapshot, String> {
    let profile = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?
        .profile(&id)
        .cloned()
        .ok_or_else(|| format!("profile not found: {}", id.0))?;
    if profile.runtime != RuntimeKind::BuiltInHttp {
        return Err("the built-in HTTP command cannot start another runtime kind".into());
    }

    let (snapshot, active_count) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let snapshot = orchestrator
            .start(&profile)
            .map_err(|error| error.to_string())?;
        let active_count = orchestrator.active_count();
        (snapshot, active_count)
    };

    if active_count == 0 {
        app.slopity_host_stop()?;
        return Ok(snapshot);
    }

    let label = hosting_label(Some(&profile.name), active_count);
    if let Err(error) = app.slopity_host_start(label, host_count(active_count)) {
        let rollback_error = {
            let mut orchestrator = orchestrator
                .lock()
                .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
            orchestrator.stop(&id).err().map(|error| error.to_string())
        };
        let reconciliation_error = reconcile_host_service(&app, &orchestrator).err();
        let mut details = vec![format!(
            "server runtime was rolled back because the host service failed: {error}"
        )];
        if let Some(rollback_error) = rollback_error {
            details.push(format!("runtime rollback also failed: {rollback_error}"));
        }
        if let Some(reconciliation_error) = reconciliation_error {
            details.push(format!(
                "remaining host state could not be reconciled: {reconciliation_error}"
            ));
        }
        return Err(details.join("; "));
    }

    Ok(snapshot)
}

#[tauri::command]
fn stop_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<ServerRuntimeSnapshot, String> {
    let snapshot = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        orchestrator.stop(&id).map_err(|error| error.to_string())?
    };
    reconcile_host_service(&app, &orchestrator)
        .map_err(|error| format!("server stopped, but host reconciliation failed: {error}"))?;
    Ok(snapshot)
}

fn hosting_label(profile_name: Option<&str>, active_count: usize) -> String {
    match (active_count, profile_name) {
        (1, Some(name)) => format!("Hosting {name}"),
        (1, None) => "Hosting 1 Slopity server".into(),
        (count, _) => format!("Hosting {count} Slopity servers"),
    }
}

fn host_count(active_count: usize) -> u32 {
    u32::try_from(active_count).unwrap_or(u32::MAX)
}

fn host_reconciliation_action(
    active_count: usize,
    status: &HostServiceStatus,
) -> HostReconciliationAction {
    if active_count == 0 {
        if status.active || status.start_request_pending || status.stop_request_pending {
            return HostReconciliationAction::StopHost;
        }
        return HostReconciliationAction::None;
    }

    if status.stop_request_pending || !status.notification_delivery_available() {
        return HostReconciliationAction::StopServersAndHost;
    }
    if !status.active {
        return HostReconciliationAction::StartHost;
    }
    if status.active_server_count != host_count(active_count) {
        return HostReconciliationAction::UpdateHost;
    }
    HostReconciliationAction::None
}

fn observed_active_count(
    orchestrator: &State<'_, SharedOrchestrator>,
) -> Result<usize, String> {
    let mut orchestrator = orchestrator
        .lock()
        .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
    Ok(orchestrator
        .snapshots()
        .iter()
        .filter(|server| is_active_state(server.state))
        .count())
}

fn stop_all_active_servers(
    orchestrator: &State<'_, SharedOrchestrator>,
) -> Result<(usize, Vec<String>), String> {
    let mut orchestrator = orchestrator
        .lock()
        .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
    let active_ids = orchestrator
        .snapshots()
        .into_iter()
        .filter(|server| is_active_state(server.state))
        .map(|server| server.server_id)
        .collect::<Vec<_>>();
    let mut errors = Vec::new();
    for id in active_ids {
        if let Err(error) = orchestrator.stop(&id) {
            errors.push(format!("{}: {error}", id.0));
        }
    }
    Ok((orchestrator.active_count(), errors))
}

fn fail_safe_after_missing_host(
    app: &tauri::AppHandle,
    orchestrator: &State<'_, SharedOrchestrator>,
    host_error: String,
) -> Result<HostServiceStatus, String> {
    let (remaining_active, stop_errors) = stop_all_active_servers(orchestrator)?;
    let cleanup_result = if remaining_active == 0 {
        app.slopity_host_stop().map(|_| ())
    } else {
        app.slopity_host_start(
            hosting_label(None, remaining_active),
            host_count(remaining_active),
        )
        .map(|_| ())
    };

    let mut details = vec![format!(
        "Android host service could not be restored while servers were active: {host_error}"
    )];
    if !stop_errors.is_empty() {
        details.push(format!(
            "failed to stop every active server: {}",
            stop_errors.join(", ")
        ));
    }
    if remaining_active > 0 {
        details.push(format!(
            "{remaining_active} server(s) remain active and still require a visible host service"
        ));
    }
    if let Err(cleanup_error) = cleanup_result {
        details.push(format!("host cleanup/recovery also failed: {cleanup_error}"));
    }
    Err(details.join("; "))
}

fn stop_servers_after_native_request(
    app: &tauri::AppHandle,
    orchestrator: &State<'_, SharedOrchestrator>,
) -> Result<HostServiceStatus, String> {
    let (remaining_active, stop_errors) = stop_all_active_servers(orchestrator)?;
    if remaining_active == 0 {
        let status = app.slopity_host_stop()?;
        if stop_errors.is_empty() {
            return Ok(status);
        }
        return Err(format!(
            "host stop request completed, but one or more runtime stops reported errors: {}",
            stop_errors.join(", ")
        ));
    }

    let host_update = app.slopity_host_start(
        hosting_label(None, remaining_active),
        host_count(remaining_active),
    );
    let mut details = if stop_errors.is_empty() {
        vec![format!(
            "native host stop request left {remaining_active} server(s) active"
        )]
    } else {
        vec![format!(
            "native host stop request could not stop every server: {}",
            stop_errors.join(", ")
        )]
    };
    if let Err(error) = host_update {
        details.push(format!(
            "foreground notification could not be restored for remaining servers: {error}"
        ));
    }
    Err(details.join("; "))
}

fn reconcile_host_service(
    app: &tauri::AppHandle,
    orchestrator: &State<'_, SharedOrchestrator>,
) -> Result<HostServiceStatus, String> {
    let status = app.slopity_host_status()?;
    let active_count = observed_active_count(orchestrator)?;
    match host_reconciliation_action(active_count, &status) {
        HostReconciliationAction::None => Ok(status),
        HostReconciliationAction::StopHost => app.slopity_host_stop(),
        HostReconciliationAction::StartHost => {
            match app.slopity_host_start(hosting_label(None, active_count), host_count(active_count)) {
                Ok(status) => Ok(status),
                Err(error) => fail_safe_after_missing_host(app, orchestrator, error),
            }
        }
        HostReconciliationAction::UpdateHost => app
            .slopity_host_start(hosting_label(None, active_count), host_count(active_count))
            .map_err(|error| {
                format!(
                    "active foreground host service could not update its server count; existing hosting remains foregrounded and the next observation will retry: {error}"
                )
            }),
        HostReconciliationAction::StopServersAndHost => {
            stop_servers_after_native_request(app, orchestrator)
        }
    }
}

fn ensure_server_stopped(
    id: &ServerId,
    orchestrator: &State<'_, SharedOrchestrator>,
) -> Result<(), String> {
    let mut orchestrator = orchestrator
        .lock()
        .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
    if orchestrator.is_active(id) {
        Err(format!(
            "stop {} before editing, disabling, or deleting its profile",
            id.0
        ))
    } else {
        Ok(())
    }
}

fn is_active_state(state: ServerState) -> bool {
    matches!(
        state,
        ServerState::Starting | ServerState::Running | ServerState::Stopping
    )
}

fn runtime_catalog(orchestrator: &ServerOrchestrator) -> Vec<RuntimeAvailability> {
    [
        RuntimeKind::BuiltInHttp,
        RuntimeKind::Java,
        RuntimeKind::NodeJs,
        RuntimeKind::Python,
        RuntimeKind::Php,
        RuntimeKind::Native,
        RuntimeKind::Custom,
    ]
    .into_iter()
    .map(|runtime| orchestrator.runtime_availability(runtime))
    .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_slopity_host::init())
        .invoke_handler(tauri::generate_handler![
            dashboard_snapshot,
            host_service_status,
            list_server_profiles,
            validate_server_profile,
            create_server_profile,
            update_server_profile,
            clone_server_profile,
            set_server_profile_enabled,
            delete_server_profile,
            list_builtin_http_servers,
            start_builtin_http_server,
            stop_builtin_http_server
        ])
        .setup(|app| {
            let profile_path = app.path().app_data_dir()?.join("profiles-v1.json");
            let profile_store = ProfileStore::load_or_create(profile_path, sample_profiles())?;
            let mut orchestrator = ServerOrchestrator::default();
            orchestrator.register_adapter(Box::<HttpServerManager>::default())?;
            app.manage(Mutex::new(profile_store));
            app.manage(Mutex::new(orchestrator));
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("Slopity failed to start: {error}"));
}

#[cfg(test)]
mod tests {
    use super::{host_reconciliation_action, HostReconciliationAction};
    use tauri_plugin_slopity_host::HostServiceStatus;

    fn android_status(active: bool, active_server_count: u32) -> HostServiceStatus {
        HostServiceStatus {
            platform: "android".into(),
            active,
            start_request_pending: false,
            notification_visible: active,
            notification_permission_granted: true,
            notification_permission_required: true,
            notifications_enabled: true,
            notification_channel_enabled: true,
            label: "Hosting".into(),
            active_server_count,
            stop_request_pending: false,
            reason: String::new(),
        }
    }

    #[test]
    fn reconciliation_stops_stale_host_when_no_servers_remain() {
        assert_eq!(
            host_reconciliation_action(0, &android_status(true, 1)),
            HostReconciliationAction::StopHost
        );
    }

    #[test]
    fn reconciliation_restarts_missing_host_for_active_servers() {
        assert_eq!(
            host_reconciliation_action(2, &android_status(false, 0)),
            HostReconciliationAction::StartHost
        );
    }

    #[test]
    fn reconciliation_updates_stale_multi_server_count() {
        assert_eq!(
            host_reconciliation_action(3, &android_status(true, 1)),
            HostReconciliationAction::UpdateHost
        );
    }

    #[test]
    fn reconciliation_consumes_native_stop_request() {
        let mut status = android_status(true, 2);
        status.stop_request_pending = true;
        assert_eq!(
            host_reconciliation_action(2, &status),
            HostReconciliationAction::StopServersAndHost
        );
    }

    #[test]
    fn reconciliation_refuses_hidden_android_hosting() {
        let mut status = android_status(true, 1);
        status.notifications_enabled = false;
        status.notification_visible = false;
        assert_eq!(
            host_reconciliation_action(1, &status),
            HostReconciliationAction::StopServersAndHost
        );
    }

    #[test]
    fn reconciliation_leaves_aligned_host_state_untouched() {
        assert_eq!(
            host_reconciliation_action(2, &android_status(true, 2)),
            HostReconciliationAction::None
        );
    }
}
