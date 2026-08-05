#![forbid(unsafe_code)]

use slopity_core::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeHandle, RuntimeKind, RuntimeRequest,
    ServerId,
};
use std::{collections::HashMap, process::Child};

pub struct LocalProcessRuntime {
    runtime_kind: RuntimeKind,
    children: HashMap<ServerId, Child>,
}

impl LocalProcessRuntime {
    pub fn new(runtime_kind: RuntimeKind) -> Self {
        Self {
            runtime_kind,
            children: HashMap::new(),
        }
    }
}

impl RuntimeAdapter for LocalProcessRuntime {
    fn runtime_kind(&self) -> RuntimeKind {
        self.runtime_kind
    }

    fn availability(&self) -> RuntimeAvailability {
        RuntimeAvailability {
            available: cfg!(not(target_os = "android")),
            runtime: self.runtime_kind,
            reason: if cfg!(target_os = "android") {
                "The desktop local-process adapter is intentionally disabled on Android.".into()
            } else {
                "The adapter is available; each profile still requires a verified executable."
                    .into()
            },
        }
    }

    fn start(&mut self, request: RuntimeRequest) -> Result<RuntimeHandle, RuntimeError> {
        #[cfg(target_os = "android")]
        {
            let _ = request;
            return Err(RuntimeError::Unavailable(
                "Android requires a separately proven runtime and foreground-service strategy."
                    .into(),
            ));
        }

        #[cfg(not(target_os = "android"))]
        {
            use std::process::{Command, Stdio};

            let profile = request.profile;
            if self.children.contains_key(&profile.id) {
                return Err(RuntimeError::AlreadyRunning(profile.id.0));
            }
            let executable = profile.executable.ok_or_else(|| {
                RuntimeError::InvalidRequest("profile has no executable path".into())
            })?;
            if !executable.is_file() {
                return Err(RuntimeError::InvalidRequest(format!(
                    "executable does not exist or is not a file: {}",
                    executable.display()
                )));
            }

            let mut command = Command::new(executable);
            command.args(&profile.arguments);
            command.stdin(Stdio::piped());
            command.stdout(Stdio::piped());
            command.stderr(Stdio::piped());
            if let Some(directory) = &profile.working_directory {
                if !directory.is_dir() {
                    return Err(RuntimeError::InvalidRequest(format!(
                        "working directory does not exist: {}",
                        directory.display()
                    )));
                }
                command.current_dir(directory);
            }

            let child = command
                .spawn()
                .map_err(|error| RuntimeError::Process(error.to_string()))?;
            let process_id = child.id();
            let server_id = profile.id.clone();
            self.children.insert(profile.id, child);

            Ok(RuntimeHandle {
                server_id,
                process_id: Some(process_id),
            })
        }
    }

    fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError> {
        let mut child = self
            .children
            .remove(server_id)
            .ok_or_else(|| RuntimeError::NotRunning(server_id.0.clone()))?;
        child
            .kill()
            .map_err(|error| RuntimeError::Process(error.to_string()))?;
        child
            .wait()
            .map_err(|error| RuntimeError::Process(error.to_string()))?;
        Ok(())
    }
}
