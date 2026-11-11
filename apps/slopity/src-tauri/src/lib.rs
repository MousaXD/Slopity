#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{
    authorize_start, sample_profiles, CapabilitySnapshot, ProfileRecoveryNotice, ProfileStore,
    ProfileStoreError, ResourceAccounting, ResourceAccountingSnapshot, ResourcePlan,
    ResourcePlanner, RuntimeAvailability, RuntimeFailureEvidence, RuntimeKind, ServerId,
    ServerOrchestrator, ServerProfile, ServerRuntimeSnapshot, ServerState, StartAdmissionRejection,
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

// Serializes runtime lifecycle changes with native host-service reconciliation.
// A native start may take time to acknowledge; without this gate another Rust
// command can stop/start a server and subsequently be overwritten by a stale
// foreground host count. Never hold the profile/orchestrator locks across
// native calls; this independent operation lock spans the complete command.
static HOST_OPERATION_GATE: Mutex<()> = Mutex::new(());

fn lock_host_operation() -> Result<std::sync::MutexGuard<'static, ()>, String> {
    HOST_OPERATION_GATE
        .lock()
        .map_err(|_| "host operation gate lock is poisoned".to_string())
}

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
    runtime_failures: Vec<RuntimeFailureEvidence>,
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

#[derive(Debug, Serialize)]
#[serde(tag = "kind", rename_all = "kebab-case")]
enum StartServerCommandError {
    Admission { rejection: StartAdmissionRejection },
    Runtime { message: String },
    HostService { message: String },
    Internal { message: String },
}

impl StartServerCommandError {
    fn internal(message: impl Into<String>) -> Self {
        Self::Internal {
            message: message.into(),
        }
    }

    fn runtime(message: impl Into<String>) -> Self {
        Self::Runtime {
            message: message.into(),
        }
    }

    fn host_service(message: impl Into<String>) -> Self {
        Self::HostService {
            message: message.into(),
        }
    }
}

#[tauri::command]
fn dashboard_snapshot(
    app: tauri::AppHandle,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<DashboardSnapshot, String> {
    let _host_operation = lock_host_operation()?;
    let device_telemetry = read_device_telemetry(&app);
    let capability = capability_snapshot(&device_telemetry);
    let (profiles, profile_recovery_notices) = {
        let store = store
            .lock()
            .map_err(|_| "profile store lock is poisoned".to_string())?;
        (store.profiles().to_vec(), store.recovery_notices().to_vec())
    };
    let host_service_status = reconcile_host_service(&app, &orchestrator)?;
    let (server_snapshots, runtime_failures, runtimes) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let server_snapshots = orchestrator.snapshots();
        let runtime_failures = orchestrator.failures();
        let runtimes = runtime_catalog(&orchestrator);
        (server_snapshots, runtime_failures, runtimes)
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
        runtime_failures,
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
    let _host_operation = lock_host_operation()?;
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
    let id = profile.id.clone();
    mutate_stopped_profile(&store, &orchestrator, &id, |store| store.update(profile))
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
    mutate_stopped_profile(&store, &orchestrator, &id, |store| {
        store.set_enabled(&id, enabled)
    })
}

#[tauri::command]
fn delete_server_profile(
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerProfile>, String> {
    mutate_stopped_profile(&store, &orchestrator, &id, |store| store.delete(&id))
}

#[tauri::command]
fn list_builtin_http_servers(
    app: tauri::AppHandle,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<Vec<ServerRuntimeSnapshot>, String> {
    let _host_operation = lock_host_operation()?;
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
) -> Result<ServerRuntimeSnapshot, StartServerCommandError> {
    let _host_operation = lock_host_operation().map_err(StartServerCommandError::internal)?;
    // Read native telemetry before taking either lock. Profile lookup, admission, and
    // actual launch are one store -> orchestrator transaction; host calls remain outside.
    let device_telemetry = read_device_telemetry(&app);
    let capability = capability_snapshot(&device_telemetry);
    let (snapshot, active_count, profile_name) =
        start_profile_transaction(&store, &orchestrator, &capability, &id)?;

    if active_count == 0 {
        app.slopity_host_stop()
            .map_err(StartServerCommandError::host_service)?;
        return Ok(snapshot);
    }

    let label = hosting_label(Some(&profile_name), active_count);
    if let Err(error) = app.slopity_host_start(label, host_count(active_count)) {
        let rollback_error = {
            let mut orchestrator = orchestrator.lock().map_err(|_| {
                StartServerCommandError::internal("server orchestrator lock is poisoned")
            })?;
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
        return Err(StartServerCommandError::host_service(details.join("; ")));
    }

    Ok(snapshot)
}

#[tauri::command]
fn stop_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<ServerRuntimeSnapshot, String> {
    let _host_operation = lock_host_operation()?;
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

fn read_device_telemetry(app: &tauri::AppHandle) -> HostDeviceTelemetry {
    app.slopity_host_telemetry().unwrap_or_else(|error| {
        HostDeviceTelemetry::unavailable(
            std::env::consts::OS,
            format!("host telemetry unavailable: {error}"),
        )
    })
}

fn capability_snapshot(device_telemetry: &HostDeviceTelemetry) -> CapabilitySnapshot {
    CapabilitySnapshot {
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        logical_cpus: std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(0),
        total_memory_mib: device_telemetry.total_memory_mib,
        available_memory_mib: device_telemetry.available_memory_mib,
    }
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

fn observed_active_count(orchestrator: &State<'_, SharedOrchestrator>) -> Result<usize, String> {
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
        details.push(format!(
            "host cleanup/recovery also failed: {cleanup_error}"
        ));
    }
    Err(details.join("; "))
}

fn incomplete_native_stop_message(
    remaining_active: usize,
    stop_errors: &[String],
    status: &HostServiceStatus,
) -> String {
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

    if status.stop_request_pending {
        // Kotlin rejects registerStartRequest while stopRequestPending is set.
        // Attempting slopity_host_start here cannot restore or update the host.
        // Do not stop the existing foreground host while Rust listeners remain.
        details.push(
            concat!(
                "Android stop is still pending; the existing foreground host is not stopped, ",
                "and a new native host start/update is intentionally not attempted"
            )
            .into(),
        );
    } else {
        // Notification visibility is unavailable. A new start/update cannot be
        // used to claim safe hosting when that prerequisite is missing.
        details.push(
            concat!(
                "foreground notification delivery is unavailable; native host ",
                "start/update is not attempted while visibility is unsafe"
            )
            .into(),
        );
    }
    details.push(format!(
        "{remaining_active} server(s) may still be active; shutdown was not confirmed"
    ));
    details.join("; ")
}

fn stop_servers_after_native_request(
    app: &tauri::AppHandle,
    orchestrator: &State<'_, SharedOrchestrator>,
    status: &HostServiceStatus,
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

    Err(incomplete_native_stop_message(
        remaining_active,
        &stop_errors,
        status,
    ))
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
            stop_servers_after_native_request(app, orchestrator, &status)
        }
    }
}

// All profile mutations that require a stopped server use the same lock order as
// startup. The store guard remains held through the active-runtime check and
// durable write, preventing a concurrent start from reading a stale revision.
fn mutate_stopped_profile<T>(
    store: &SharedProfileStore,
    orchestrator: &SharedOrchestrator,
    id: &ServerId,
    action: impl FnOnce(&mut ProfileStore) -> Result<T, ProfileStoreError>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(id, orchestrator)?;
    action(&mut store).map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

// The test hook runs after reading a profile but *before* taking the orchestrator
// lock. Production passes a no-op; tests coordinate a concurrent mutation at the
// exact former TOCTOU window. Both locks remain held until the runtime is started.
fn start_profile_transaction(
    store: &SharedProfileStore,
    orchestrator: &SharedOrchestrator,
    capability: &CapabilitySnapshot,
    id: &ServerId,
) -> Result<(ServerRuntimeSnapshot, usize, String), StartServerCommandError> {
    start_profile_transaction_after_read(store, orchestrator, capability, id, || {})
}

fn start_profile_transaction_after_read(
    store: &SharedProfileStore,
    orchestrator: &SharedOrchestrator,
    capability: &CapabilitySnapshot,
    id: &ServerId,
    after_profile_read: impl FnOnce(),
) -> Result<(ServerRuntimeSnapshot, usize, String), StartServerCommandError> {
    let store = store
        .lock()
        .map_err(|_| StartServerCommandError::internal("profile store lock is poisoned"))?;
    let profile = store
        .profile(id)
        .cloned()
        .ok_or_else(|| StartServerCommandError::runtime(format!("profile not found: {}", id.0)))?;
    if profile.runtime != RuntimeKind::BuiltInHttp {
        return Err(StartServerCommandError::runtime(
            "the built-in HTTP command cannot start another runtime kind",
        ));
    }

    after_profile_read();
    let mut orchestrator = orchestrator
        .lock()
        .map_err(|_| StartServerCommandError::internal("server orchestrator lock is poisoned"))?;
    let existing = orchestrator.snapshots();
    let active_server_ids = existing
        .iter()
        .filter(|server| is_active_state(server.state))
        .map(|server| server.server_id.clone())
        .collect::<Vec<_>>();
    let availability = orchestrator.runtime_availability(profile.runtime);
    let permit = authorize_start(
        capability,
        store.profiles(),
        &active_server_ids,
        &profile,
        &availability,
    )
    .map_err(|rejection| StartServerCommandError::Admission { rejection })?;
    let snapshot = orchestrator
        .start(&profile, permit)
        .map_err(|error| StartServerCommandError::runtime(error.to_string()))?;
    let active_count = orchestrator.active_count();
    Ok((snapshot, active_count, profile.name))
}

fn ensure_server_stopped(id: &ServerId, orchestrator: &SharedOrchestrator) -> Result<(), String> {
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
    fn incomplete_native_stop_reports_surviving_servers_without_host_restart() {
        let mut status = android_status(true, 2);
        status.stop_request_pending = true;
        assert_eq!(
            host_reconciliation_action(2, &status),
            HostReconciliationAction::StopServersAndHost
        );
        let detail = super::incomplete_native_stop_message(
            1,
            &["alpha: runtime stop failed".to_string()],
            &status,
        );
        assert!(detail.contains("alpha: runtime stop failed"));
        assert!(detail.contains("1 server(s) may still be active"));
        assert!(detail.contains("native host start/update is intentionally not attempted"));
        assert!(detail.contains("shutdown was not confirmed"));
    }

    #[test]
    fn incomplete_stop_rejects_restarting_a_host_without_notifications() {
        let mut status = android_status(true, 2);
        status.notification_visible = false;
        assert_eq!(
            host_reconciliation_action(2, &status),
            HostReconciliationAction::StopServersAndHost
        );
        let detail = super::incomplete_native_stop_message(2, &[], &status);
        assert!(detail.contains("2 server(s) active"));
        assert!(detail.contains("foreground notification delivery is unavailable"));
        assert!(detail.contains("shutdown was not confirmed"));
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

    // These tests exercise the same transaction and mutation functions as the
    // Tauri commands without invoking native host APIs or binding a TCP socket.
    use super::{
        mutate_stopped_profile, start_profile_transaction, start_profile_transaction_after_read,
        SharedOrchestrator, SharedProfileStore, StartServerCommandError,
    };
    use slopity_core::{
        CapabilitySnapshot, NetworkScope, ProfileStore, RuntimeAdapter, RuntimeAvailability,
        RuntimeError, RuntimeHandle, RuntimeKind, RuntimeObservation, RuntimeRequest, ServerId,
        ServerOrchestrator, ServerProfile, ServerState,
    };
    use std::{
        collections::HashSet,
        fs,
        path::PathBuf,
        sync::{
            atomic::{AtomicUsize, Ordering},
            mpsc, Arc, Mutex,
        },
        thread,
        time::Duration,
    };

    #[derive(Default)]
    struct TestHttpAdapter {
        active: HashSet<ServerId>,
        started: Arc<Mutex<Vec<ServerProfile>>>,
    }

    impl RuntimeAdapter for TestHttpAdapter {
        fn runtime_kind(&self) -> RuntimeKind {
            RuntimeKind::BuiltInHttp
        }

        fn availability(&self) -> RuntimeAvailability {
            RuntimeAvailability {
                runtime: RuntimeKind::BuiltInHttp,
                available: true,
                reason: "deterministic test adapter".into(),
            }
        }

        fn start(&mut self, request: RuntimeRequest) -> Result<RuntimeHandle, RuntimeError> {
            let id = request.profile.id.clone();
            self.started.lock().unwrap().push(request.profile);
            self.active.insert(id.clone());
            Ok(RuntimeHandle {
                server_id: id,
                process_id: None,
            })
        }

        fn stop(&mut self, id: &ServerId) -> Result<(), RuntimeError> {
            if self.active.remove(id) {
                Ok(())
            } else {
                Err(RuntimeError::NotRunning(id.0.clone()))
            }
        }

        fn observe(&mut self, id: &ServerId) -> Option<RuntimeObservation> {
            self.active
                .contains(id)
                .then(|| RuntimeObservation::running(id.clone()))
        }

        fn observations(&mut self) -> Vec<RuntimeObservation> {
            self.active
                .iter()
                .cloned()
                .map(RuntimeObservation::running)
                .collect()
        }
    }

    struct TestProfileDir(PathBuf);

    impl Drop for TestProfileDir {
        fn drop(&mut self) {
            let _ = fs::remove_dir_all(&self.0);
        }
    }

    fn test_profile() -> ServerProfile {
        ServerProfile {
            id: ServerId("atomic".into()),
            name: "Atomic start test".into(),
            runtime: RuntimeKind::BuiltInHttp,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port: 8_890,
            memory_mib: 128,
            network_scope: NetworkScope::Loopback,
            enabled: true,
        }
    }

    fn test_capability() -> CapabilitySnapshot {
        CapabilitySnapshot {
            platform: "linux".into(),
            architecture: "x86_64".into(),
            logical_cpus: 8,
            total_memory_mib: Some(16_384),
            available_memory_mib: Some(12_288),
        }
    }

    struct TestFixture {
        store: Arc<SharedProfileStore>,
        orchestrator: Arc<SharedOrchestrator>,
        started: Arc<Mutex<Vec<ServerProfile>>>,
        _dir: TestProfileDir,
    }

    fn test_fixture() -> TestFixture {
        static NEXT: AtomicUsize = AtomicUsize::new(0);
        let path = std::env::temp_dir().join(format!(
            "slopity-tauri-start-{}-{}",
            std::process::id(),
            NEXT.fetch_add(1, Ordering::Relaxed)
        ));
        fs::create_dir(&path).unwrap();
        let store =
            ProfileStore::load_or_create(path.join("profiles.json"), vec![test_profile()]).unwrap();
        let started = Arc::new(Mutex::new(Vec::new()));
        let mut orchestrator = ServerOrchestrator::default();
        orchestrator
            .register_adapter(Box::new(TestHttpAdapter {
                active: HashSet::new(),
                started: Arc::clone(&started),
            }))
            .unwrap();
        TestFixture {
            store: Arc::new(Mutex::new(store)),
            orchestrator: Arc::new(Mutex::new(orchestrator)),
            started,
            _dir: TestProfileDir(path),
        }
    }

    #[test]
    fn host_gate_prevents_stale_start_count_after_concurrent_stop() {
        let TestFixture {
            store,
            orchestrator,
            _dir,
            ..
        } = test_fixture();
        let reported_count = Arc::new(AtomicUsize::new(usize::MAX));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let start_store = Arc::clone(&store);
        let start_orchestrator = Arc::clone(&orchestrator);
        let start_reported = Arc::clone(&reported_count);
        let starter = thread::spawn(move || {
            let _gate = super::lock_host_operation().unwrap();
            let (_, count, _) = start_profile_transaction(
                &start_store,
                &start_orchestrator,
                &test_capability(),
                &ServerId("atomic".into()),
            )
            .unwrap();
            started_tx.send(count).unwrap();
            release_rx.recv().unwrap();
            // Simulates the native start acknowledgement after Rust launch.
            start_reported.store(count, Ordering::SeqCst);
        });

        assert_eq!(started_rx.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
        let (blocked_tx, blocked_rx) = mpsc::channel();
        let stop_orchestrator = Arc::clone(&orchestrator);
        let stop_reported = Arc::clone(&reported_count);
        let stopper = thread::spawn(move || {
            blocked_tx
                .send(matches!(
                    super::HOST_OPERATION_GATE.try_lock(),
                    Err(std::sync::TryLockError::WouldBlock)
                ))
                .unwrap();
            let _gate = super::lock_host_operation().unwrap();
            let mut orchestrator = stop_orchestrator.lock().unwrap();
            orchestrator.stop(&ServerId("atomic".into())).unwrap();
            stop_reported.store(orchestrator.active_count(), Ordering::SeqCst);
        });

        assert!(blocked_rx.recv_timeout(Duration::from_secs(5)).unwrap());
        release_tx.send(()).unwrap();
        starter.join().unwrap();
        stopper.join().unwrap();
        assert_eq!(reported_count.load(Ordering::SeqCst), 0);
        assert_eq!(orchestrator.lock().unwrap().active_count(), 0);
    }

    #[test]
    fn host_gate_prevents_older_start_from_overwriting_second_server_count() {
        let TestFixture {
            store,
            orchestrator,
            _dir,
            ..
        } = test_fixture();
        let mut second = test_profile();
        second.id = ServerId("second".into());
        second.name = "Second host".into();
        second.port = 8_891;
        store.lock().unwrap().create(second).unwrap();

        let reported_count = Arc::new(AtomicUsize::new(0));
        let (started_tx, started_rx) = mpsc::channel();
        let (release_tx, release_rx) = mpsc::channel();
        let start_store = Arc::clone(&store);
        let start_orchestrator = Arc::clone(&orchestrator);
        let start_reported = Arc::clone(&reported_count);
        let first = thread::spawn(move || {
            let _gate = super::lock_host_operation().unwrap();
            let (_, count, _) = start_profile_transaction(
                &start_store,
                &start_orchestrator,
                &test_capability(),
                &ServerId("atomic".into()),
            )
            .unwrap();
            started_tx.send(count).unwrap();
            release_rx.recv().unwrap();
            start_reported.store(count, Ordering::SeqCst);
        });

        assert_eq!(started_rx.recv_timeout(Duration::from_secs(5)).unwrap(), 1);
        let (blocked_tx, blocked_rx) = mpsc::channel();
        let second_store = Arc::clone(&store);
        let second_orchestrator = Arc::clone(&orchestrator);
        let second_reported = Arc::clone(&reported_count);
        let second_start = thread::spawn(move || {
            blocked_tx
                .send(matches!(
                    super::HOST_OPERATION_GATE.try_lock(),
                    Err(std::sync::TryLockError::WouldBlock)
                ))
                .unwrap();
            let _gate = super::lock_host_operation().unwrap();
            let (_, count, _) = start_profile_transaction(
                &second_store,
                &second_orchestrator,
                &test_capability(),
                &ServerId("second".into()),
            )
            .unwrap();
            second_reported.store(count, Ordering::SeqCst);
        });

        assert!(blocked_rx.recv_timeout(Duration::from_secs(5)).unwrap());
        release_tx.send(()).unwrap();
        first.join().unwrap();
        second_start.join().unwrap();
        assert_eq!(reported_count.load(Ordering::SeqCst), 2);
        assert_eq!(orchestrator.lock().unwrap().active_count(), 2);
    }

    #[derive(Clone, Copy, Debug)]
    enum TestMutation {
        Disable,
        Delete,
        ChangePort,
        ExposeLan,
    }

    fn apply_test_mutation(
        mutation: TestMutation,
        store: &SharedProfileStore,
        orchestrator: &SharedOrchestrator,
    ) -> Result<Vec<ServerProfile>, String> {
        let id = ServerId("atomic".into());
        mutate_stopped_profile(store, orchestrator, &id, |store| match mutation {
            TestMutation::Disable => store.set_enabled(&id, false).map(|_| ()),
            TestMutation::Delete => store.delete(&id).map(|_| ()),
            TestMutation::ChangePort | TestMutation::ExposeLan => {
                let mut profile = store.profile(&id).unwrap().clone();
                match mutation {
                    TestMutation::ChangePort => profile.port = 8_891,
                    TestMutation::ExposeLan => profile.network_scope = NetworkScope::Lan,
                    _ => unreachable!(),
                }
                store.update(profile).map(|_| ())
            }
        })
    }

    fn assert_start_wins_over_concurrent_mutation(mutation: TestMutation) {
        let TestFixture {
            store,
            orchestrator,
            started,
            _dir,
        } = test_fixture();
        let (read_tx, read_rx) = mpsc::channel();
        let (resume_tx, resume_rx) = mpsc::channel();
        let start_store = Arc::clone(&store);
        let start_orchestrator = Arc::clone(&orchestrator);
        let starter = thread::spawn(move || {
            start_profile_transaction_after_read(
                &start_store,
                &start_orchestrator,
                &test_capability(),
                &ServerId("atomic".into()),
                || {
                    read_tx.send(()).unwrap();
                    resume_rx.recv().unwrap();
                },
            )
        });
        read_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            store.try_lock().is_err(),
            "start must retain the profile lock after reading the profile"
        );

        let (mutate_tx, mutate_rx) = mpsc::channel();
        let mutation_store = Arc::clone(&store);
        let mutation_orchestrator = Arc::clone(&orchestrator);
        let mutator = thread::spawn(move || {
            mutate_tx
                .send(apply_test_mutation(
                    mutation,
                    &mutation_store,
                    &mutation_orchestrator,
                ))
                .unwrap();
        });
        // Mutation cannot finish during the test-controlled in-flight start.
        assert!(
            mutate_rx.recv_timeout(Duration::from_millis(50)).is_err(),
            "profile mutation completed before the start transaction released its locks"
        );
        resume_tx.send(()).unwrap();
        let (snapshot, count, _) = starter.join().unwrap().unwrap();
        assert_eq!(snapshot.state, ServerState::Running);
        assert_eq!(count, 1);

        let mutation_result = mutate_rx.recv_timeout(Duration::from_secs(5)).unwrap();
        assert!(
            mutation_result
                .unwrap_err()
                .contains("stop atomic before editing"),
            "{mutation:?} must be rejected after start linearizes"
        );
        mutator.join().unwrap();
        assert_eq!(store.lock().unwrap().profiles(), &[test_profile()]);
        assert_eq!(started.lock().unwrap().as_slice(), &[test_profile()]);
    }

    fn assert_mutation_wins_before_start(mutation: TestMutation) {
        let TestFixture {
            store,
            orchestrator,
            started,
            _dir,
        } = test_fixture();
        let saved = apply_test_mutation(mutation, &store, &orchestrator).unwrap();
        let result = start_profile_transaction(
            &store,
            &orchestrator,
            &test_capability(),
            &ServerId("atomic".into()),
        );

        match mutation {
            TestMutation::Disable => {
                assert!(matches!(
                    result,
                    Err(StartServerCommandError::Admission { .. })
                ));
                assert!(!saved[0].enabled);
                assert!(started.lock().unwrap().is_empty());
            }
            TestMutation::Delete => {
                assert!(matches!(
                    result,
                    Err(StartServerCommandError::Runtime { .. })
                ));
                assert!(saved.is_empty());
                assert!(started.lock().unwrap().is_empty());
            }
            TestMutation::ChangePort | TestMutation::ExposeLan => {
                let (snapshot, count, name) = result.unwrap();
                assert_eq!(snapshot.state, ServerState::Running);
                assert_eq!(count, 1);
                assert_eq!(name, saved[0].name);
                assert_eq!(started.lock().unwrap().as_slice(), saved.as_slice());
                match mutation {
                    TestMutation::ChangePort => assert_eq!(saved[0].port, 8_891),
                    TestMutation::ExposeLan => {
                        assert_eq!(saved[0].network_scope, NetworkScope::Lan)
                    }
                    _ => unreachable!(),
                }
            }
        }
    }

    #[test]
    fn concurrent_disable_cannot_start_obsolete_enabled_profile() {
        assert_start_wins_over_concurrent_mutation(TestMutation::Disable);
        assert_mutation_wins_before_start(TestMutation::Disable);
    }

    #[test]
    fn concurrent_delete_cannot_start_missing_profile() {
        assert_start_wins_over_concurrent_mutation(TestMutation::Delete);
        assert_mutation_wins_before_start(TestMutation::Delete);
    }

    #[test]
    fn concurrent_port_change_cannot_start_old_port() {
        assert_start_wins_over_concurrent_mutation(TestMutation::ChangePort);
        assert_mutation_wins_before_start(TestMutation::ChangePort);
    }

    #[test]
    fn concurrent_loopback_to_lan_change_cannot_use_stale_scope() {
        assert_start_wins_over_concurrent_mutation(TestMutation::ExposeLan);
        assert_mutation_wins_before_start(TestMutation::ExposeLan);
    }
}
