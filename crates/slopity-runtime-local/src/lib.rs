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
            // Resolve the checked file to an absolute path before spawning.
            // Command::new("name") can search PATH even if a different
            // relative file named "name" passed an earlier is_file() check.
            let executable = executable.canonicalize().map_err(|error| {
                RuntimeError::InvalidRequest(format!(
                    "executable path cannot be resolved ({}): {error}",
                    executable.display()
                ))
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
                let directory = directory.canonicalize().map_err(|error| {
                    RuntimeError::InvalidRequest(format!(
                        "working directory cannot be resolved ({}): {error}",
                        directory.display()
                    ))
                })?;
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
        let child = self
            .children
            .get_mut(server_id)
            .ok_or_else(|| RuntimeError::NotRunning(server_id.0.clone()))?;
        if let Err(kill_error) = child.kill() {
            // A child that exited before kill() is already safely stopped.
            // Otherwise retain ownership so the user can retry termination.
            match child.try_wait() {
                Ok(Some(_)) => {
                    self.children.remove(server_id);
                    return Ok(());
                }
                Ok(None) => return Err(RuntimeError::Process(kill_error.to_string())),
                Err(wait_error) => {
                    return Err(RuntimeError::Process(format!(
                        "kill failed: {kill_error}; status check failed: {wait_error}"
                    )));
                }
            }
        }
        // A wait error must not discard a potentially live child handle.
        child
            .wait()
            .map_err(|error| RuntimeError::Process(error.to_string()))?;
        self.children.remove(server_id);
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    #[test]
    fn rejects_non_file_executable_without_using_a_shell_or_path_lookup() {
        let profile = slopity_core::ServerProfile {
            id: ServerId("bad-executable".into()),
            name: "Not an executable file".into(),
            runtime: RuntimeKind::Native,
            executable: Some(std::env::current_dir().expect("test working directory")),
            arguments: vec!["--do-not-run".into()],
            working_directory: None,
            port: 8_080,
            memory_mib: 128,
            network_scope: slopity_core::NetworkScope::Loopback,
            enabled: true,
        };
        let mut adapter = LocalProcessRuntime::new(RuntimeKind::Native);
        // Android intentionally refuses all local process launches.
        if cfg!(target_os = "android") {
            assert!(matches!(
                adapter.start(RuntimeRequest { profile }),
                Err(RuntimeError::Unavailable(_))
            ));
        } else {
            assert!(matches!(
                adapter.start(RuntimeRequest { profile }),
                Err(RuntimeError::InvalidRequest(_))
            ));
        }
    }

    #[test]
    fn already_reaped_child_can_be_stopped_and_removed() {
        // Execute the test binary with an impossible filter, not a shell command.
        let mut child = Command::new(std::env::current_exe().expect("test executable"))
            .args(["--exact", "__slopity_no_matching_test__"])
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("test child should launch");
        child.wait().expect("test child should exit");

        let id = ServerId("finished-child".into());
        let mut runtime = LocalProcessRuntime::new(RuntimeKind::Native);
        runtime.children.insert(id.clone(), child);
        runtime
            .stop(&id)
            .expect("already exited child should count as stopped");
        assert!(!runtime.children.contains_key(&id));
    }
}
