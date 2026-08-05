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
mod mobile;

#[derive(Debug, Clone, Serialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServiceCapability {
    pub platform: &'static str,
    pub foreground_service_available: bool,
    pub durable_hosting_available: bool,
    pub reason: &'static str,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct HostServiceStatus {
    pub platform: String,
    pub active: bool,
    pub notification_visible: bool,
    pub reason: String,
}

#[cfg(target_os = "android")]
#[derive(Debug, Serialize)]
#[serde(rename_all = "camelCase")]
pub(crate) struct StartHostRequest {
    pub label: String,
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
                reason: "The Android foreground-service bridge is compiled, but notification visibility and background survival still require a real-device proof.",
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

    fn start(&self, label: String) -> Result<HostServiceStatus, String> {
        #[cfg(target_os = "android")]
        let status = self.mobile.start(label)?;
        #[cfg(not(target_os = "android"))]
        let status = HostServiceStatus {
            platform: std::env::consts::OS.into(),
            active: true,
            notification_visible: false,
            reason: format!("{label}. Desktop hosting remains active while Slopity is running."),
        };
        self.active.store(status.active, Ordering::Release);
        Ok(status)
    }

    fn stop(&self) -> Result<HostServiceStatus, String> {
        #[cfg(target_os = "android")]
        let status = self.mobile.stop()?;
        #[cfg(not(target_os = "android"))]
        let status = HostServiceStatus {
            platform: std::env::consts::OS.into(),
            active: false,
            notification_visible: false,
            reason: "Desktop host activity stopped.".into(),
        };
        self.active.store(false, Ordering::Release);
        Ok(status)
    }
}

pub trait SlopityHostExt<R: Runtime> {
    fn slopity_host_capability(&self) -> HostServiceCapability;
    fn slopity_host_start(&self, label: String) -> Result<HostServiceStatus, String>;
    fn slopity_host_stop(&self) -> Result<HostServiceStatus, String>;
}

impl<R: Runtime, T: Manager<R>> SlopityHostExt<R> for T {
    fn slopity_host_capability(&self) -> HostServiceCapability {
        HostServiceBridge::<R>::capability()
    }

    fn slopity_host_start(&self, label: String) -> Result<HostServiceStatus, String> {
        self.state::<HostServiceBridge<R>>().start(label)
    }

    fn slopity_host_stop(&self) -> Result<HostServiceStatus, String> {
        self.state::<HostServiceBridge<R>>().stop()
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
