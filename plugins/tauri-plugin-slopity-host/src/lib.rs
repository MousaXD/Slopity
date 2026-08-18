#![forbid(unsafe_code)]

use serde::{Deserialize, Serialize};
use std::{
    marker::PhantomData,
    sync::atomic::{AtomicBool, Ordering},
};
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[cfg(target_os = "android")]
use tauri::plugin::PermissionState;

#[cfg(target_os = "android")]
mod mobile;
#[cfg(not(target_os = "android"))]
mod telemetry;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServiceCapability {
    pub platform: &'static str,
    pub foreground_service_available: bool,
    pub durable_hosting_available: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServiceStatus {
    pub platform: String,
    pub active: bool,
    pub notification_visible: bool,
    pub notification_permission_granted: bool,
    pub notification_permission_required: bool,
    pub label: String,
    pub active_server_count: u32,
    pub stop_request_pending: bool,
    pub reason: String,
}

#[derive(Debug, Clone, Default, Serialize, Deserialize, PartialEq)]
#[serde(rename_all = "camelCase")]
pub struct HostDeviceTelemetry {
    pub platform: String,
    pub source: String,
    pub total_memory_mib: Option<u64>,
    pub available_memory_mib: Option<u64>,
    pub battery_percentage: Option<u8>,
    pub charging: Option<bool>,
    pub battery_temperature_celsius: Option<f32>,
    pub thermal_status: Option<String>,
    pub free_storage_mib: Option<u64>,
}

impl HostDeviceTelemetry {
    pub fn unavailable(platform: impl Into<String>, source: impl Into<String>) -> Self {
        Self {
            platform: platform.into(),
            source: source.into(),
            ..Self::default()
        }
    }
}

#[cfg(target_os = "android")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartHostRequest {
    pub label: String,
    pub active_server_count: u32,
}

pub struct HostServiceBridge<R: Runtime> {
    active: AtomicBool,
    marker: PhantomData<fn() -> R>,
    #[cfg(target_os = "android")]
    mobile: mobile::MobileHostService<R>,
}

impl<R: Runtime> HostServiceBridge<R> {
    #[cfg(target_os = "android")]
    fn new(mobile: mobile::MobileHostService<R>) -> Self {
        Self {
            active: AtomicBool::new(false),
            marker: PhantomData,
            mobile,
        }
    }

    #[cfg(not(target_os = "android"))]
    fn new() -> Self {
        Self {
            active: AtomicBool::new(false),
            marker: PhantomData,
        }
    }

    pub fn capability() -> HostServiceCapability {
        if cfg!(target_os = "android") {
            HostServiceCapability {
                platform: "android",
                foreground_service_available: true,
                durable_hosting_available: false,
                reason: "Android has native foreground-service status and notification-permission handling, but background durability remains unproven until physical-device validation.",
            }
        } else if cfg!(target_os = "windows") {
            HostServiceCapability {
                platform: "windows",
                foreground_service_available: false,
                durable_hosting_available: true,
                reason: "Desktop in-process hosting is available, but Windows validation remains paused.",
            }
        } else if cfg!(target_os = "linux") {
            HostServiceCapability {
                platform: "linux",
                foreground_service_available: false,
                durable_hosting_available: true,
                reason: "Linux can host the built-in Rust HTTP probe while the Slopity process is running.",
            }
        } else {
            HostServiceCapability {
                platform: "unsupported",
                foreground_service_available: false,
                durable_hosting_available: false,
                reason: "This platform has not been declared as a Slopity hosting target.",
            }
        }
    }

    #[cfg(target_os = "android")]
    fn telemetry(&self) -> Result<HostDeviceTelemetry, String> {
        self.mobile.telemetry()
    }

    #[cfg(not(target_os = "android"))]
    fn telemetry(&self) -> Result<HostDeviceTelemetry, String> {
        Ok(telemetry::host_device_telemetry())
    }

    fn start(&self, label: String, active_server_count: u32) -> Result<HostServiceStatus, String> {
        #[cfg(target_os = "android")]
        let status = {
            let current = self.mobile.status()?;
            if current.notification_permission_required
                && !current.notification_permission_granted
                && matches!(
                    self.mobile.notification_permission_state()?,
                    PermissionState::Prompt | PermissionState::PromptWithRationale
                )
            {
                self.mobile.request_notification_permission()?;
            }
            self.mobile.start(label, active_server_count)?
        };
        #[cfg(not(target_os = "android"))]
        let status = desktop_started_status(label, active_server_count);
        self.active.store(status.active, Ordering::Release);
        Ok(status)
    }

    fn stop(&self) -> Result<HostServiceStatus, String> {
        #[cfg(target_os = "android")]
        let status = self.mobile.stop()?;
        #[cfg(not(target_os = "android"))]
        let status = desktop_stopped_status();
        self.active.store(status.active, Ordering::Release);
        Ok(status)
    }

    fn status(&self) -> Result<HostServiceStatus, String> {
        #[cfg(target_os = "android")]
        let status = self.mobile.status()?;
        #[cfg(not(target_os = "android"))]
        let status = desktop_observed_status(self.active.load(Ordering::Acquire));
        self.active.store(status.active, Ordering::Release);
        Ok(status)
    }
}

#[cfg(not(target_os = "android"))]
fn desktop_started_status(label: String, active_server_count: u32) -> HostServiceStatus {
    HostServiceStatus {
        platform: std::env::consts::OS.into(),
        active: true,
        notification_visible: false,
        notification_permission_granted: true,
        notification_permission_required: false,
        label,
        active_server_count,
        stop_request_pending: false,
        reason: "Desktop hosting remains active while Slopity is running.".into(),
    }
}

#[cfg(not(target_os = "android"))]
fn desktop_stopped_status() -> HostServiceStatus {
    HostServiceStatus {
        platform: std::env::consts::OS.into(),
        active: false,
        notification_visible: false,
        notification_permission_granted: true,
        notification_permission_required: false,
        label: String::new(),
        active_server_count: 0,
        stop_request_pending: false,
        reason: "Desktop host activity stopped.".into(),
    }
}

#[cfg(not(target_os = "android"))]
fn desktop_observed_status(active: bool) -> HostServiceStatus {
    HostServiceStatus {
        platform: std::env::consts::OS.into(),
        active,
        notification_visible: false,
        notification_permission_granted: true,
        notification_permission_required: false,
        label: String::new(),
        active_server_count: 0,
        stop_request_pending: false,
        reason: "Desktop host activity is process-local and has no foreground notification.".into(),
    }
}

pub trait SlopityHostExt<R: Runtime> {
    fn slopity_host_capability(&self) -> HostServiceCapability;
    fn slopity_host_telemetry(&self) -> Result<HostDeviceTelemetry, String>;
    fn slopity_host_start(
        &self,
        label: String,
        active_server_count: u32,
    ) -> Result<HostServiceStatus, String>;
    fn slopity_host_stop(&self) -> Result<HostServiceStatus, String>;
    fn slopity_host_status(&self) -> Result<HostServiceStatus, String>;
}

impl<R: Runtime, T: Manager<R>> SlopityHostExt<R> for T {
    fn slopity_host_capability(&self) -> HostServiceCapability {
        HostServiceBridge::<R>::capability()
    }

    fn slopity_host_telemetry(&self) -> Result<HostDeviceTelemetry, String> {
        self.state::<HostServiceBridge<R>>().telemetry()
    }

    fn slopity_host_start(
        &self,
        label: String,
        active_server_count: u32,
    ) -> Result<HostServiceStatus, String> {
        self.state::<HostServiceBridge<R>>()
            .start(label, active_server_count)
    }

    fn slopity_host_stop(&self) -> Result<HostServiceStatus, String> {
        self.state::<HostServiceBridge<R>>().stop()
    }

    fn slopity_host_status(&self) -> Result<HostServiceStatus, String> {
        self.state::<HostServiceBridge<R>>().status()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("slopity-host")
        .setup(|app, api| {
            #[cfg(target_os = "android")]
            let bridge: HostServiceBridge<R> = {
                let handle = api.register_android_plugin("com.slopity.host", "HostPlugin")?;
                HostServiceBridge::new(mobile::MobileHostService::new(handle))
            };
            #[cfg(not(target_os = "android"))]
            let bridge: HostServiceBridge<R> = {
                let _ = api;
                HostServiceBridge::new()
            };
            app.manage(bridge);
            Ok(())
        })
        .build()
}

#[cfg(test)]
mod tests {
    use super::{
        desktop_started_status, desktop_stopped_status, HostDeviceTelemetry, HostServiceStatus,
    };

    #[test]
    fn status_serializes_permission_and_reconciliation_fields() {
        let status = HostServiceStatus {
            platform: "android".into(),
            active: true,
            notification_visible: false,
            notification_permission_granted: false,
            notification_permission_required: true,
            label: "Hosting Test Server".into(),
            active_server_count: 2,
            stop_request_pending: true,
            reason: "Permission denied".into(),
        };

        let value = match serde_json::to_value(status) {
            Ok(value) => value,
            Err(error) => panic!("status should serialize: {error}"),
        };
        assert_eq!(value["platform"].as_str(), Some("android"));
        assert_eq!(value["active"].as_bool(), Some(true));
        assert_eq!(value["notificationVisible"].as_bool(), Some(false));
        assert_eq!(
            value["notificationPermissionGranted"].as_bool(),
            Some(false)
        );
        assert_eq!(
            value["notificationPermissionRequired"].as_bool(),
            Some(true)
        );
        assert_eq!(value["label"].as_str(), Some("Hosting Test Server"));
        assert_eq!(value["activeServerCount"].as_u64(), Some(2));
        assert_eq!(value["stopRequestPending"].as_bool(), Some(true));
        assert_eq!(value["reason"].as_str(), Some("Permission denied"));
    }

    #[test]
    fn desktop_start_and_stop_mapping_is_consistent() {
        let started = desktop_started_status("Hosting Test Server".into(), 3);
        assert!(started.active);
        assert_eq!(started.active_server_count, 3);
        assert!(!started.notification_visible);
        assert!(!started.notification_permission_required);
        assert!(started.notification_permission_granted);

        let stopped = desktop_stopped_status();
        assert!(!stopped.active);
        assert_eq!(stopped.active_server_count, 0);
        assert!(!stopped.stop_request_pending);
    }

    #[test]
    fn unavailable_telemetry_keeps_measurements_unknown() {
        let telemetry = HostDeviceTelemetry::unavailable("test", "no provider");
        assert_eq!(telemetry.platform, "test");
        assert_eq!(telemetry.source, "no provider");
        assert_eq!(telemetry.total_memory_mib, None);
        assert_eq!(telemetry.available_memory_mib, None);
    }
}
