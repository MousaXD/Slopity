use crate::{HostServiceStatus, StartHostRequest};
use tauri::{plugin::PluginHandle, Runtime};

pub(crate) struct MobileHostService<R: Runtime>(PluginHandle<R>);

impl<R: Runtime> MobileHostService<R> {
    pub(crate) fn new(handle: PluginHandle<R>) -> Self {
        Self(handle)
    }

    pub(crate) fn start(&self, label: String) -> Result<HostServiceStatus, String> {
        self.0
            .run_mobile_plugin("startHosting", StartHostRequest { label })
            .map_err(|error| error.to_string())
    }

    pub(crate) fn stop(&self) -> Result<HostServiceStatus, String> {
        self.0
            .run_mobile_plugin("stopHosting", ())
            .map_err(|error| error.to_string())
    }
}
