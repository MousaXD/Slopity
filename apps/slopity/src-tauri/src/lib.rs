#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{
    authorize_start, sample_profiles, CapabilitySnapshot, ProfileRecoveryNotice, ProfileStore,
    ResourceAccounting, ResourceAccountingSnapshot, ResourcePlan, ResourcePlanner,
    RuntimeAvailability, RuntimeFailureEvidence, RuntimeKind, ServerId, ServerOrchestrator,
    ServerProfile, ServerRuntimeSnapshot, ServerState, StartAdmissionRejection, ValidationIssue,
    PROFILE_SCHEMA_VERSION,
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
    runtime_failures: Vec<RuntimeFailureEvidence>,
    profile_schema_version: u32,
    resource_plan: ResourcePlan,
    resource_accounting: ResourceAccountingSnapshot,
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
    let device_telemetry = read_device_telemetry(&app);
    let capability = capability_snapshot(&device_telemetry);
    let (profiles, profile_recovery_notices) = {
        let store = store
            .lock()
            .map_err(|_| "profile store lock is poisoned".to_string())?;
        (store.profiles().to_vec(), store.recovery_notices().to_vec())
    };
    let (server_snapshots, runtime_failures, active_count, runtimes) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let server_snapshots = orchestrator.snapshots();
        let active_count = server_snapshots
            .iter()
            .filter(|server| is_active_state(server.state))
            .count();
        let runtimes = runtime_catalog(&orchestrator);
        let runtime_failures = orchestrator.failures();
        (server_snapshots, runtime_failures, active_count, runtimes)
    };
    let active_server_ids = server_snapshots
        .iter()
        .filter(|server| is_active_state(server.state))
        .map(|server| server.server_id.clone())
        .collect::<Vec<_>>();
    let resource_plan = ResourcePlanner::plan(&capability);
    let resource_accounting =
        ResourceAccounting::summarize(&capability, &profiles, &active_server_ids);
    let host_service_status = host_status_after_observation(&app, active_count)?;

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
fn host_service_status(app: tauri::AppHandle) -> Result<HostServiceStatus, String> {
    app.slopity_host_status()
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
    let (snapshots, active_count) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let snapshots = orchestrator.snapshots();
        let active_count = snapshots
            .iter()
            .filter(|server| is_active_state(server.state))
            .count();
        (snapshots, active_count)
    };
    if active_count == 0 {
        app.slopity_host_stop()?;
    }
    Ok(snapshots)
}

#[tauri::command]
fn start_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<ServerRuntimeSnapshot, StartServerCommandError> {
    let (profile, profiles) = {
        let store = store
            .lock()
            .map_err(|_| StartServerCommandError::internal("profile store lock is poisoned"))?;
        let profile = store.profile(&id).cloned().ok_or_else(|| {
            StartServerCommandError::runtime(format!("profile not found: {}", id.0))
        })?;
        (profile, store.profiles().to_vec())
    };
    if profile.runtime != RuntimeKind::BuiltInHttp {
        return Err(StartServerCommandError::runtime(
            "the built-in HTTP command cannot start another runtime kind",
        ));
    }

    let device_telemetry = read_device_telemetry(&app);
    let capability = capability_snapshot(&device_telemetry);
    let (snapshot, active_count) = {
        let mut orchestrator = orchestrator.lock().map_err(|_| {
            StartServerCommandError::internal("server orchestrator lock is poisoned")
        })?;
        let existing = orchestrator.snapshots();
        let active_server_ids = existing
            .iter()
            .filter(|server| is_active_state(server.state))
            .map(|server| server.server_id.clone())
            .collect::<Vec<_>>();
        let availability = orchestrator.runtime_availability(profile.runtime);
        let permit = authorize_start(
            &capability,
            &profiles,
            &active_server_ids,
            &profile,
            &availability,
        )
        .map_err(|rejection| StartServerCommandError::Admission { rejection })?;
        let snapshot = orchestrator
            .start(&profile, permit)
            .map_err(|error| StartServerCommandError::runtime(error.to_string()))?;
        let active_count = orchestrator.active_count();
        (snapshot, active_count)
    };

    if active_count == 0 {
        app.slopity_host_stop()
            .map_err(StartServerCommandError::host_service)?;
        return Ok(snapshot);
    }

    let label = hosting_label(Some(&profile.name), active_count);
    if let Err(error) = app.slopity_host_start(label, host_count(active_count)) {
        let remaining_active = {
            let mut orchestrator = orchestrator.lock().map_err(|_| {
                StartServerCommandError::internal("server orchestrator lock is poisoned")
            })?;
            let _ = orchestrator.stop(&id);
            orchestrator.active_count()
        };
        if remaining_active == 0 {
            let _ = app.slopity_host_stop();
        }
        return Err(StartServerCommandError::host_service(format!(
            "server runtime was rolled back because the host service failed: {error}"
        )));
    }

    Ok(snapshot)
}

#[tauri::command]
fn stop_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    orchestrator: State<'_, SharedOrchestrator>,
) -> Result<ServerRuntimeSnapshot, String> {
    let (snapshot, active_count) = {
        let mut orchestrator = orchestrator
            .lock()
            .map_err(|_| "server orchestrator lock is poisoned".to_string())?;
        let snapshot = orchestrator.stop(&id).map_err(|error| error.to_string())?;
        let active_count = orchestrator.active_count();
        (snapshot, active_count)
    };
    if active_count == 0 {
        app.slopity_host_stop()?;
    } else {
        app.slopity_host_start(hosting_label(None, active_count), host_count(active_count))?;
    }
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

fn host_status_after_observation(
    app: &tauri::AppHandle,
    active_count: usize,
) -> Result<HostServiceStatus, String> {
    let status = app.slopity_host_status()?;
    if active_count == 0 && status.active {
        app.slopity_host_stop()
    } else {
        Ok(status)
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
