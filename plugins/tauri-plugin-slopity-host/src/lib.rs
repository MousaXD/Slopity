#![forbid(unsafe_code)]

use serde::Serialize;
use tauri::{
    plugin::{Builder, TauriPlugin},
    Manager, Runtime,
};

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServiceCapability {
    pub platform: &'static str,
    pub durable_hosting_available: bool,
    pub reason: &'static str,
}

pub struct HostServiceBridge;

impl HostServiceBridge {
    pub fn capability() -> HostServiceCapability {
        if cfg!(target_os = "android") {
            HostServiceCapability {
                platform: "android",
                durable_hosting_available: false,
                reason: "The Tauri Android shell is present, but the Kotlin foreground-service bridge and ARM64 runtime proof are pending.",
            }
        } else if cfg!(target_os = "windows") {
            HostServiceCapability {
                platform: "windows",
                durable_hosting_available: true,
                reason: "Desktop process hosting is structurally available but no runtime provider is installed by default.",
            }
        } else if cfg!(target_os = "linux") {
            HostServiceCapability {
                platform: "linux",
                durable_hosting_available: true,
                reason: "Desktop process hosting is structurally available but no runtime provider is installed by default.",
            }
        } else {
            HostServiceCapability {
                platform: "unsupported",
                durable_hosting_available: false,
                reason: "This platform has not been declared as a Slopity hosting target.",
            }
        }
    }
}

pub trait SlopityHostExt<R: Runtime> {
    fn slopity_host_capability(&self) -> HostServiceCapability;
}

impl<R: Runtime, T: Manager<R>> SlopityHostExt<R> for T {
    fn slopity_host_capability(&self) -> HostServiceCapability {
        let _ = self.state::<HostServiceBridge>();
        HostServiceBridge::capability()
    }
}

pub fn init<R: Runtime>() -> TauriPlugin<R> {
    Builder::new("slopity-host")
        .setup(|app, _api| {
            app.manage(HostServiceBridge);
            Ok(())
        })
        .build()
}
