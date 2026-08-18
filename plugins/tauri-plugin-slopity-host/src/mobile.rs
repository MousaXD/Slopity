use crate::{HostDeviceTelemetry, HostServiceStatus, StartHostRequest};
use serde::{Deserialize, Serialize};
use tauri::{
    plugin::{PermissionState, PluginHandle},
    Runtime,
};

#[derive(Debug, Deserialize)]
#[serde(rename_all = "camelCase")]
struct NotificationPermissionResponse {
    post_notification: PermissionState,
}

#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
struct NotificationPermissionRequest {
    post_notification: bool,
}

pub(crate) struct MobileHostService<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MobileHostService<R> {
    pub(crate) fn new(handle: PluginHandle<R>) -> Self {
        Self(handle)
    }

    pub(crate) fn telemetry(&self) -> Result<HostDeviceTelemetry, String> {
        self.0
            .run_mobile_plugin("deviceTelemetry", ())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn start(
        &self,
        label: String,
        active_server_count: u32,
    ) -> Result<HostServiceStatus, String> {
        self.0
            .run_mobile_plugin(
                "startHosting",
                StartHostRequest {
                    label,
                    active_server_count,
                },
            )
            .map_err(|error| error.to_string())
    }

    pub(crate) fn stop(&self) -> Result<HostServiceStatus, String> {
        self.0
            .run_mobile_plugin("stopHosting", ())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn status(&self) -> Result<HostServiceStatus, String> {
        self.0
            .run_mobile_plugin("getStatus", ())
            .map_err(|error| error.to_string())
    }

    pub(crate) fn notification_permission_state(&self) -> Result<PermissionState, String> {
        self.0
            .run_mobile_plugin::<NotificationPermissionResponse>("checkPermissions", ())
            .map(|response| response.post_notification)
            .map_err(|error| error.to_string())
    }

    pub(crate) fn request_notification_permission(&self) -> Result<PermissionState, String> {
        self.0
            .run_mobile_plugin::<NotificationPermissionResponse>(
                "requestPermissions",
                NotificationPermissionRequest {
                    post_notification: true,
                },
            )
            .map(|response| response.post_notification)
            .map_err(|error| error.to_string())
    }
}
