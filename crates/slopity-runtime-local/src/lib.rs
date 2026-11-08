#![forbid(unsafe_code)]

use slopity_core::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeExitReason, RuntimeHandle,
    RuntimeKind, RuntimeObservation, RuntimeRequest, ServerId, ServerState,
};
use std::{
    collections::HashMap,
    process::{Child, ExitStatus},
};

pub struct LocalProcessRuntime {
    runtime_kind: RuntimeKind,
    children: HashMap<ServerId, Child>,
    observations: HashMap<ServerId, RuntimeObservation>,
}

impl LocalProcessRuntime {
    pub fn new(runtime_kind: RuntimeKind) -> Self {
        Self {
            runtime_kind,
            children: HashMap::new(),
            observations: HashMap::new(),
        }
    }

    fn refresh_children(&mut self) {
        let mut finished = Vec::new();
        for (server_id, child) in &mut self.children {
            match child.try_wait() {
                Ok(Some(status)) => {
                    finished.push((
                        server_id.clone(),
                        process_exit_observation(server_id.clone(), status),
                    ));
                }
                Ok(None) => {
                    self.observations.insert(
                        server_id.clone(),
                        RuntimeObservation::running(server_id.clone()),
                    );
                }
                Err(error) => {
                    // An inconclusive status check does not prove that a
                    // child exited. Keep it active and preserve its handle.
                    let mut observation = RuntimeObservation::running(server_id.clone());
                    observation.last_error =
                        Some(format!("Unable to inspect local process status: {error}"));
                    self.observations.insert(server_id.clone(), observation);
                }
            }
        }
        for (server_id, observation) in finished {
            self.children.remove(&server_id);
            self.observations.insert(server_id, observation);
        }
    }
}

fn process_exit_observation(server_id: ServerId, status: ExitStatus) -> RuntimeObservation {
    let reason = if status.success() {
        RuntimeExitReason::Completed
    } else {
        RuntimeExitReason::Failed
    };
    let message = format!("Local process exited with {status}.");
    let mut observation = RuntimeObservation::stopped(server_id, reason, message.clone());
    if !status.success() {
        observation.state = ServerState::Failed;
        observation.last_error = Some(message);
    }
    observation
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
            self.refresh_children();
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
            self.observations.insert(
                server_id.clone(),
                RuntimeObservation::running(server_id.clone()),
            );

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
                Ok(Some(status)) => {
                    self.children.remove(server_id);
                    self.observations.insert(
                        server_id.clone(),
                        process_exit_observation(server_id.clone(), status),
                    );
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
        self.observations.insert(
            server_id.clone(),
            RuntimeObservation::stopped(
                server_id.clone(),
                RuntimeExitReason::UserRequested,
                "Local process stopped after a user request.",
            ),
        );
        Ok(())
    }

    fn observe(&mut self, server_id: &ServerId) -> Option<RuntimeObservation> {
        self.refresh_children();
        self.observations.get(server_id).cloned()
    }

    fn observations(&mut self) -> Vec<RuntimeObservation> {
        self.refresh_children();
        let mut observations = self.observations.values().cloned().collect::<Vec<_>>();
        observations.sort_by(|left, right| left.server_id.0.cmp(&right.server_id.0));
        observations
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use std::process::{Command, Stdio};

    fn test_child(args: &[&str]) -> Child {
        Command::new(std::env::current_exe().expect("test executable"))
            .args(args)
            .stdout(Stdio::null())
            .stderr(Stdio::null())
            .spawn()
            .expect("test child should launch")
    }

    #[test]
    fn observed_successful_child_exit_is_reaped() {
        let id = ServerId("completed-child".into());
        let mut child = test_child(&["--exact", "__slopity_no_matching_test__"]);
        child.wait().expect("test child should exit");
        let mut runtime = LocalProcessRuntime::new(RuntimeKind::Native);
        runtime.children.insert(id.clone(), child);

        let observation = runtime.observe(&id).expect("completion must be observable");
        assert_eq!(observation.state, ServerState::Stopped);
        assert_eq!(
            observation.exit.as_ref().map(|exit| exit.reason),
            Some(RuntimeExitReason::Completed)
        );
        assert!(!runtime.children.contains_key(&id));
        assert_eq!(runtime.observations().len(), 1);
    }

    #[test]
    fn observed_failed_child_exit_is_reported_with_diagnostics() {
        let id = ServerId("failed-child".into());
        let mut child = test_child(&["--invalid-slopity-test-runner-argument"]);
        let status = child.wait().expect("test child should finish");
        assert!(!status.success(), "invalid test-runner argument must fail");
        let mut runtime = LocalProcessRuntime::new(RuntimeKind::Native);
        runtime.children.insert(id.clone(), child);

        let observation = runtime.observe(&id).expect("failure must be observable");
        assert_eq!(observation.state, ServerState::Failed);
        assert_eq!(
            observation.exit.as_ref().map(|exit| exit.reason),
            Some(RuntimeExitReason::Failed)
        );
        assert!(observation.last_error.is_some());
        assert!(!runtime.children.contains_key(&id));
    }

    #[test]
    #[cfg(not(target_os = "android"))]
    fn natural_exit_does_not_prevent_restart() {
        let id = ServerId("restart-after-exit".into());
        let profile = slopity_core::ServerProfile {
            id: id.clone(),
            name: "Restart after exit".into(),
            runtime: RuntimeKind::Native,
            executable: Some(std::env::current_exe().expect("test executable")),
            arguments: vec!["--exact".into(), "__slopity_no_matching_test__".into()],
            working_directory: None,
            port: 8_080,
            memory_mib: 128,
            network_scope: slopity_core::NetworkScope::Loopback,
            enabled: true,
        };
        let mut runtime = LocalProcessRuntime::new(RuntimeKind::Native);
        runtime
            .start(RuntimeRequest {
                profile: profile.clone(),
            })
            .expect("initial test process should launch");
        runtime
            .children
            .get_mut(&id)
            .expect("initial process must be owned")
            .wait()
            .expect("initial process must terminate");

        runtime
            .start(RuntimeRequest { profile })
            .expect("natural exit must not block a new process");
        assert!(runtime.children.contains_key(&id));
        assert_eq!(
            runtime
                .observations
                .get(&id)
                .map(|observation| observation.state),
            Some(ServerState::Running)
        );
        runtime.stop(&id).expect("new process must be stoppable");
    }

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
