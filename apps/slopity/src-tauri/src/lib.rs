#![forbid(unsafe_code)]

use serde::Serialize;
use slopity_core::{
    sample_profiles, CapabilitySnapshot, ProfileStore, ResourcePlan, ResourcePlanner,
    RuntimeAvailability, RuntimeKind, ServerId, ServerProfile, ValidationIssue,
    PROFILE_SCHEMA_VERSION,
};
use slopity_runtime_http::{HttpServerManager, HttpServerSnapshot};
use std::sync::Mutex;
use tauri::{Manager, State};
use tauri_plugin_slopity_host::{HostServiceCapability, SlopityHostExt};

type SharedProfileStore = Mutex<ProfileStore>;
type SharedHttpServers = Mutex<HttpServerManager>;

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct DashboardSnapshot {
    application: &'static str,
    platform: &'static str,
    architecture: &'static str,
    host_service: HostServiceCapability,
    runtimes: Vec<RuntimeAvailability>,
    profiles: Vec<ServerProfile>,
    servers: Vec<HttpServerSnapshot>,
    profile_schema_version: u32,
    resource_plan: ResourcePlan,
}

#[tauri::command]
fn dashboard_snapshot(
    app: tauri::AppHandle,
    store: State<'_, SharedProfileStore>,
    servers: State<'_, SharedHttpServers>,
) -> Result<DashboardSnapshot, String> {
    let capability = CapabilitySnapshot {
        platform: std::env::consts::OS.into(),
        architecture: std::env::consts::ARCH.into(),
        logical_cpus: std::thread::available_parallelism()
            .map(std::num::NonZeroUsize::get)
            .unwrap_or(1),
        total_memory_mib: 0,
        available_memory_mib: 0,
    };
    let profiles = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?
        .profiles()
        .to_vec();
    let server_snapshots = servers
        .lock()
        .map_err(|_| "HTTP server manager lock is poisoned".to_string())?
        .snapshots();

    Ok(DashboardSnapshot {
        application: "Slopity",
        platform: std::env::consts::OS,
        architecture: std::env::consts::ARCH,
        host_service: app.slopity_host_capability(),
        runtimes: runtime_catalog(),
        profiles,
        servers: server_snapshots,
        profile_schema_version: PROFILE_SCHEMA_VERSION,
        resource_plan: ResourcePlanner::plan(&capability),
    })
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
    servers: State<'_, SharedHttpServers>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&profile.id, &servers)?;
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
    servers: State<'_, SharedHttpServers>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&id, &servers)?;
    store
        .set_enabled(&id, enabled)
        .map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn delete_server_profile(
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    servers: State<'_, SharedHttpServers>,
) -> Result<Vec<ServerProfile>, String> {
    let mut store = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?;
    ensure_server_stopped(&id, &servers)?;
    store.delete(&id).map_err(|error| error.to_string())?;
    Ok(store.profiles().to_vec())
}

#[tauri::command]
fn list_builtin_http_servers(
    servers: State<'_, SharedHttpServers>,
) -> Result<Vec<HttpServerSnapshot>, String> {
    let mut servers = servers
        .lock()
        .map_err(|_| "HTTP server manager lock is poisoned".to_string())?;
    Ok(servers.snapshots())
}

#[tauri::command]
fn start_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    store: State<'_, SharedProfileStore>,
    servers: State<'_, SharedHttpServers>,
) -> Result<HttpServerSnapshot, String> {
    let profile = store
        .lock()
        .map_err(|_| "profile store lock is poisoned".to_string())?
        .profile(&id)
        .cloned()
        .ok_or_else(|| format!("profile not found: {}", id.0))?;

    let (snapshot, active_count) = {
        let mut servers = servers
            .lock()
            .map_err(|_| "HTTP server manager lock is poisoned".to_string())?;
        let snapshot = servers.start(&profile).map_err(|error| error.to_string())?;
        let active_count = servers.active_count();
        (snapshot, active_count)
    };

    let label = if active_count == 1 {
        format!("Hosting {}", profile.name)
    } else {
        format!("Hosting {active_count} Slopity servers")
    };
    if let Err(error) = app.slopity_host_start(label) {
        let mut servers = servers
            .lock()
            .map_err(|_| "HTTP server manager lock is poisoned".to_string())?;
        let _ = servers.stop(&id);
        return Err(format!(
            "HTTP listener was rolled back because the host service failed: {error}"
        ));
    }

    Ok(snapshot)
}

#[tauri::command]
fn stop_builtin_http_server(
    app: tauri::AppHandle,
    id: ServerId,
    servers: State<'_, SharedHttpServers>,
) -> Result<HttpServerSnapshot, String> {
    let (snapshot, active_count) = {
        let mut servers = servers
            .lock()
            .map_err(|_| "HTTP server manager lock is poisoned".to_string())?;
        let snapshot = servers.stop(&id).map_err(|error| error.to_string())?;
        let active_count = servers.active_count();
        (snapshot, active_count)
    };
    if active_count == 0 {
        app.slopity_host_stop()?;
    }
    Ok(snapshot)
}

fn ensure_server_stopped(
    id: &ServerId,
    servers: &State<'_, SharedHttpServers>,
) -> Result<(), String> {
    let mut servers = servers
        .lock()
        .map_err(|_| "HTTP server manager lock is poisoned".to_string())?;
    if servers.is_active(id) {
        Err(format!(
            "stop {} before editing, disabling, or deleting its profile",
            id.0
        ))
    } else {
        Ok(())
    }
}

fn runtime_catalog() -> Vec<RuntimeAvailability> {
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
    .map(|runtime| {
        if runtime == RuntimeKind::BuiltInHttp {
            RuntimeAvailability {
                available: true,
                runtime,
                reason: "The harmless built-in Rust HTTP probe is compiled into Slopity.".into(),
            }
        } else {
            RuntimeAvailability {
                available: false,
                runtime,
                reason: "No verified external runtime provider is installed.".into(),
            }
        }
    })
    .collect()
}

#[cfg_attr(mobile, tauri::mobile_entry_point)]
pub fn run() {
    tauri::Builder::default()
        .plugin(tauri_plugin_slopity_host::init())
        .invoke_handler(tauri::generate_handler![
            dashboard_snapshot,
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
            app.manage(Mutex::new(profile_store));
            app.manage(Mutex::new(HttpServerManager::default()));
            Ok(())
        })
        .run(tauri::generate_context!())
        .unwrap_or_else(|error| eprintln!("Slopity failed to start: {error}"));
}
