use crate::{
    RuntimeAdapter, RuntimeAvailability, RuntimeError, RuntimeExit, RuntimeExitReason,
    RuntimeIdentity, RuntimeKind, RuntimeLogEntry, RuntimeObservation, RuntimeRequest, ServerId,
    ServerProfile, ServerState,
};
use serde::{Deserialize, Serialize};
use std::collections::{HashMap, VecDeque};

const MAX_RUNTIME_EVENTS: usize = 256;

pub type ObservedServerState = ServerState;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum DesiredServerState {
    Stopped,
    Running,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct ServerRuntimeSnapshot {
    pub server_id: ServerId,
    pub desired_state: DesiredServerState,
    pub state: ObservedServerState,
    pub runtime: RuntimeIdentity,
    pub process_id: Option<u32>,
    pub bind_address: String,
    pub urls: Vec<String>,
    pub request_count: u64,
    pub logs: Vec<RuntimeLogEntry>,
    pub last_error: Option<String>,
    pub exit: Option<RuntimeExit>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "kebab-case")]
pub enum RuntimeEventKind {
    StartRequested,
    Started,
    StopRequested,
    Stopped,
    Failed,
    StateChanged,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "camelCase")]
pub struct RuntimeEvent {
    pub sequence: u64,
    pub server_id: ServerId,
    pub runtime: RuntimeIdentity,
    pub kind: RuntimeEventKind,
    pub state: ObservedServerState,
    pub message: String,
}

#[derive(Default)]
pub struct ServerOrchestrator {
    adapters: HashMap<RuntimeKind, Box<dyn RuntimeAdapter>>,
    snapshots: HashMap<ServerId, ServerRuntimeSnapshot>,
    events: VecDeque<RuntimeEvent>,
    next_event_sequence: u64,
}

impl ServerOrchestrator {
    pub fn register_adapter(
        &mut self,
        adapter: Box<dyn RuntimeAdapter>,
    ) -> Result<(), RuntimeError> {
        let runtime = adapter.runtime_kind();
        if self.adapters.contains_key(&runtime) {
            return Err(RuntimeError::AdapterAlreadyRegistered(format!(
                "{runtime:?}"
            )));
        }

        let identity = adapter.runtime_identity();
        if identity.runtime != runtime {
            return Err(RuntimeError::InvalidRequest(format!(
                "adapter {} reports mismatched runtime identity",
                identity.adapter
            )));
        }

        self.adapters.insert(runtime, adapter);
        Ok(())
    }

    pub fn runtime_availability(&self, runtime: RuntimeKind) -> RuntimeAvailability {
        self.adapters
            .get(&runtime)
            .map(|adapter| adapter.availability())
            .unwrap_or_else(|| RuntimeAvailability {
                available: false,
                runtime,
                reason: "No verified runtime adapter is registered.".into(),
            })
    }

    pub fn start(
        &mut self,
        profile: &ServerProfile,
    ) -> Result<ServerRuntimeSnapshot, RuntimeError> {
        self.refresh();
        if self
            .snapshots
            .get(&profile.id)
            .is_some_and(|snapshot| is_active_state(snapshot.state))
        {
            return Err(RuntimeError::AlreadyRunning(profile.id.0.clone()));
        }

        let runtime = profile.runtime;
        let (identity, availability) = self
            .adapters
            .get(&runtime)
            .map(|adapter| (adapter.runtime_identity(), adapter.availability()))
            .ok_or_else(|| {
                RuntimeError::Unavailable(format!(
                    "no verified adapter is registered for {runtime:?}"
                ))
            })?;
        if !availability.available {
            return Err(RuntimeError::Unavailable(availability.reason));
        }

        self.snapshots.insert(
            profile.id.clone(),
            ServerRuntimeSnapshot {
                server_id: profile.id.clone(),
                desired_state: DesiredServerState::Running,
                state: ServerState::Starting,
                runtime: identity.clone(),
                process_id: None,
                bind_address: String::new(),
                urls: Vec::new(),
                request_count: 0,
                logs: Vec::new(),
                last_error: None,
                exit: None,
            },
        );
        self.record_event(
            profile.id.clone(),
            identity.clone(),
            RuntimeEventKind::StartRequested,
            ServerState::Starting,
            "Start requested.",
        );

        let start_result = if let Some(adapter) = self.adapters.get_mut(&runtime) {
            match adapter.start(RuntimeRequest {
                profile: profile.clone(),
            }) {
                Ok(handle) => Ok((handle, adapter.observe(&profile.id))),
                Err(error) => Err(error),
            }
        } else {
            Err(RuntimeError::Unavailable(format!(
                "registered runtime adapter disappeared for {runtime:?}"
            )))
        };

        match start_result {
            Ok((handle, observation)) => {
                let snapshot = observation
                    .map(|observation| {
                        snapshot_from_observation(
                            identity.clone(),
                            DesiredServerState::Running,
                            handle.process_id,
                            observation,
                        )
                    })
                    .unwrap_or_else(|| ServerRuntimeSnapshot {
                        server_id: profile.id.clone(),
                        desired_state: DesiredServerState::Running,
                        state: ServerState::Running,
                        runtime: identity.clone(),
                        process_id: handle.process_id,
                        bind_address: String::new(),
                        urls: Vec::new(),
                        request_count: 0,
                        logs: Vec::new(),
                        last_error: None,
                        exit: None,
                    });
                let event_kind = if snapshot.state == ServerState::Failed {
                    RuntimeEventKind::Failed
                } else {
                    RuntimeEventKind::Started
                };
                self.snapshots.insert(profile.id.clone(), snapshot.clone());
                self.record_event(
                    profile.id.clone(),
                    identity,
                    event_kind,
                    snapshot.state,
                    if event_kind == RuntimeEventKind::Failed {
                        "Runtime reported a failed state during startup."
                    } else {
                        "Runtime started."
                    },
                );
                Ok(snapshot)
            }
            Err(error) => {
                let message = error.to_string();
                self.snapshots.insert(
                    profile.id.clone(),
                    failed_snapshot(
                        profile.id.clone(),
                        identity.clone(),
                        DesiredServerState::Running,
                        message.clone(),
                    ),
                );
                self.record_event(
                    profile.id.clone(),
                    identity,
                    RuntimeEventKind::Failed,
                    ServerState::Failed,
                    message,
                );
                Err(error)
            }
        }
    }

    pub fn stop(&mut self, server_id: &ServerId) -> Result<ServerRuntimeSnapshot, RuntimeError> {
        self.refresh();
        let current = self
            .snapshots
            .get(server_id)
            .cloned()
            .filter(|snapshot| is_active_state(snapshot.state))
            .ok_or_else(|| RuntimeError::NotRunning(server_id.0.clone()))?;
        let runtime = current.runtime.runtime;
        let identity = current.runtime.clone();

        let mut stopping = current;
        stopping.desired_state = DesiredServerState::Stopped;
        stopping.state = ServerState::Stopping;
        stopping.exit = None;
        self.snapshots.insert(server_id.clone(), stopping);
        self.record_event(
            server_id.clone(),
            identity.clone(),
            RuntimeEventKind::StopRequested,
            ServerState::Stopping,
            "Stop requested.",
        );

        let stop_result = if let Some(adapter) = self.adapters.get_mut(&runtime) {
            match adapter.stop(server_id) {
                Ok(()) => Ok(adapter.observe(server_id)),
                Err(error) => Err(error),
            }
        } else {
            Err(RuntimeError::Unavailable(format!(
                "registered runtime adapter disappeared for {runtime:?}"
            )))
        };

        match stop_result {
            Ok(observation) => {
                let snapshot = observation
                    .map(|observation| {
                        snapshot_from_observation(
                            identity.clone(),
                            DesiredServerState::Stopped,
                            None,
                            observation,
                        )
                    })
                    .unwrap_or_else(|| ServerRuntimeSnapshot {
                        server_id: server_id.clone(),
                        desired_state: DesiredServerState::Stopped,
                        state: ServerState::Stopped,
                        runtime: identity.clone(),
                        process_id: None,
                        bind_address: String::new(),
                        urls: Vec::new(),
                        request_count: 0,
                        logs: Vec::new(),
                        last_error: None,
                        exit: Some(RuntimeExit {
                            reason: RuntimeExitReason::UserRequested,
                            message: "Runtime stopped after a user request.".into(),
                        }),
                    });
                let event_kind = if snapshot.state == ServerState::Failed {
                    RuntimeEventKind::Failed
                } else {
                    RuntimeEventKind::Stopped
                };
                self.snapshots.insert(server_id.clone(), snapshot.clone());
                self.record_event(
                    server_id.clone(),
                    identity,
                    event_kind,
                    snapshot.state,
                    if event_kind == RuntimeEventKind::Failed {
                        "Runtime reported a failed state while stopping."
                    } else {
                        "Runtime stopped."
                    },
                );
                Ok(snapshot)
            }
            Err(error) => {
                let message = error.to_string();
                self.snapshots.insert(
                    server_id.clone(),
                    failed_snapshot(
                        server_id.clone(),
                        identity.clone(),
                        DesiredServerState::Stopped,
                        message.clone(),
                    ),
                );
                self.record_event(
                    server_id.clone(),
                    identity,
                    RuntimeEventKind::Failed,
                    ServerState::Failed,
                    message,
                );
                Err(error)
            }
        }
    }

    pub fn snapshot(&mut self, server_id: &ServerId) -> Option<ServerRuntimeSnapshot> {
        self.refresh();
        self.snapshots.get(server_id).cloned()
    }

    pub fn snapshots(&mut self) -> Vec<ServerRuntimeSnapshot> {
        self.refresh();
        let mut snapshots = self.snapshots.values().cloned().collect::<Vec<_>>();
        snapshots.sort_by(|left, right| left.server_id.0.cmp(&right.server_id.0));
        snapshots
    }

    pub fn is_active(&mut self, server_id: &ServerId) -> bool {
        self.refresh();
        self.snapshots
            .get(server_id)
            .is_some_and(|snapshot| is_active_state(snapshot.state))
    }

    pub fn active_count(&mut self) -> usize {
        self.refresh();
        self.snapshots
            .values()
            .filter(|snapshot| is_active_state(snapshot.state))
            .count()
    }

    pub fn events(&mut self) -> Vec<RuntimeEvent> {
        self.refresh();
        self.events.iter().cloned().collect()
    }

    pub fn stop_all(&mut self) {
        let server_ids = self
            .snapshots()
            .into_iter()
            .filter(|snapshot| is_active_state(snapshot.state))
            .map(|snapshot| snapshot.server_id)
            .collect::<Vec<_>>();
        for server_id in server_ids {
            let _ = self.stop(&server_id);
        }
    }

    fn refresh(&mut self) {
        let runtimes = self.adapters.keys().copied().collect::<Vec<_>>();
        for runtime in runtimes {
            let Some(adapter) = self.adapters.get_mut(&runtime) else {
                continue;
            };
            let identity = adapter.runtime_identity();
            let observations = adapter.observations();
            for observation in observations {
                self.apply_observation(identity.clone(), observation);
            }
        }
    }

    fn apply_observation(&mut self, identity: RuntimeIdentity, observation: RuntimeObservation) {
        let previous = self.snapshots.get(&observation.server_id).cloned();
        let desired_state = previous
            .as_ref()
            .map(|snapshot| snapshot.desired_state)
            .unwrap_or_else(|| {
                if is_active_state(observation.state) {
                    DesiredServerState::Running
                } else {
                    DesiredServerState::Stopped
                }
            });
        let process_id = previous.as_ref().and_then(|snapshot| snapshot.process_id);
        let server_id = observation.server_id.clone();
        let state = observation.state;
        let snapshot =
            snapshot_from_observation(identity.clone(), desired_state, process_id, observation);
        self.snapshots.insert(server_id.clone(), snapshot);

        if previous.as_ref().map(|snapshot| snapshot.state) != Some(state) {
            let kind = if state == ServerState::Failed {
                RuntimeEventKind::Failed
            } else {
                RuntimeEventKind::StateChanged
            };
            self.record_event(
                server_id,
                identity,
                kind,
                state,
                format!("Observed runtime state changed to {state:?}."),
            );
        }
    }

    fn record_event(
        &mut self,
        server_id: ServerId,
        runtime: RuntimeIdentity,
        kind: RuntimeEventKind,
        state: ObservedServerState,
        message: impl Into<String>,
    ) {
        self.next_event_sequence = self.next_event_sequence.saturating_add(1);
        self.events.push_back(RuntimeEvent {
            sequence: self.next_event_sequence,
            server_id,
            runtime,
            kind,
            state,
            message: message.into(),
        });
        while self.events.len() > MAX_RUNTIME_EVENTS {
            self.events.pop_front();
        }
    }
}

impl Drop for ServerOrchestrator {
    fn drop(&mut self) {
        let server_ids = self
            .snapshots
            .values()
            .filter(|snapshot| is_active_state(snapshot.state))
            .map(|snapshot| snapshot.server_id.clone())
            .collect::<Vec<_>>();

        for server_id in server_ids {
            let Some(runtime) = self
                .snapshots
                .get(&server_id)
                .map(|snapshot| snapshot.runtime.runtime)
            else {
                continue;
            };
            if let Some(adapter) = self.adapters.get_mut(&runtime) {
                let _ = adapter.stop(&server_id);
            }
        }
    }
}

fn snapshot_from_observation(
    runtime: RuntimeIdentity,
    desired_state: DesiredServerState,
    process_id: Option<u32>,
    observation: RuntimeObservation,
) -> ServerRuntimeSnapshot {
    ServerRuntimeSnapshot {
        server_id: observation.server_id,
        desired_state,
        state: observation.state,
        runtime,
        process_id,
        bind_address: observation.bind_address,
        urls: observation.urls,
        request_count: observation.request_count,
        logs: observation.logs,
        last_error: observation.last_error,
        exit: observation.exit,
    }
}

fn failed_snapshot(
    server_id: ServerId,
    runtime: RuntimeIdentity,
    desired_state: DesiredServerState,
    message: String,
) -> ServerRuntimeSnapshot {
    ServerRuntimeSnapshot {
        server_id,
        desired_state,
        state: ServerState::Failed,
        runtime,
        process_id: None,
        bind_address: String::new(),
        urls: Vec::new(),
        request_count: 0,
        logs: Vec::new(),
        last_error: Some(message.clone()),
        exit: Some(RuntimeExit {
            reason: RuntimeExitReason::Failed,
            message,
        }),
    }
}

fn is_active_state(state: ObservedServerState) -> bool {
    matches!(
        state,
        ServerState::Starting | ServerState::Running | ServerState::Stopping
    )
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{NetworkScope, RuntimeHandle, RuntimeLogLevel};

    #[derive(Default)]
    struct MockRuntime {
        observations: HashMap<ServerId, RuntimeObservation>,
        fail_on_poll: bool,
    }

    impl RuntimeAdapter for MockRuntime {
        fn runtime_kind(&self) -> RuntimeKind {
            RuntimeKind::BuiltInHttp
        }

        fn runtime_identity(&self) -> RuntimeIdentity {
            RuntimeIdentity {
                runtime: RuntimeKind::BuiltInHttp,
                adapter: "mock-http".into(),
            }
        }

        fn availability(&self) -> RuntimeAvailability {
            RuntimeAvailability {
                available: true,
                runtime: RuntimeKind::BuiltInHttp,
                reason: "test adapter".into(),
            }
        }

        fn start(&mut self, request: RuntimeRequest) -> Result<RuntimeHandle, RuntimeError> {
            if self
                .observations
                .get(&request.profile.id)
                .is_some_and(|observation| is_active_state(observation.state))
            {
                return Err(RuntimeError::AlreadyRunning(request.profile.id.0));
            }

            let server_id = request.profile.id;
            self.observations.insert(
                server_id.clone(),
                RuntimeObservation::running(server_id.clone()),
            );
            Ok(RuntimeHandle {
                server_id,
                process_id: Some(42),
            })
        }

        fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError> {
            let observation = self
                .observations
                .get_mut(server_id)
                .filter(|observation| is_active_state(observation.state))
                .ok_or_else(|| RuntimeError::NotRunning(server_id.0.clone()))?;
            observation.state = ServerState::Stopped;
            observation.exit = Some(RuntimeExit {
                reason: RuntimeExitReason::UserRequested,
                message: "stopped by test".into(),
            });
            Ok(())
        }

        fn observe(&mut self, server_id: &ServerId) -> Option<RuntimeObservation> {
            self.observations.get(server_id).cloned()
        }

        fn observations(&mut self) -> Vec<RuntimeObservation> {
            if self.fail_on_poll && !self.observations.is_empty() {
                self.fail_on_poll = false;
                for observation in self.observations.values_mut() {
                    if observation.state == ServerState::Running {
                        observation.state = ServerState::Failed;
                        observation.last_error = Some("synthetic runtime failure".into());
                        observation.exit = Some(RuntimeExit {
                            reason: RuntimeExitReason::Failed,
                            message: "synthetic runtime failure".into(),
                        });
                        observation.logs.push(RuntimeLogEntry {
                            sequence: 1,
                            level: RuntimeLogLevel::Error,
                            message: "synthetic runtime failure".into(),
                        });
                    }
                }
            }
            self.observations.values().cloned().collect()
        }
    }

    fn profile(id: &str) -> ServerProfile {
        ServerProfile {
            id: ServerId(id.into()),
            name: id.into(),
            runtime: RuntimeKind::BuiltInHttp,
            executable: None,
            arguments: Vec::new(),
            working_directory: None,
            port: 0,
            memory_mib: 128,
            network_scope: NetworkScope::Loopback,
            enabled: true,
        }
    }

    fn orchestrator() -> ServerOrchestrator {
        let mut orchestrator = ServerOrchestrator::default();
        orchestrator
            .register_adapter(Box::<MockRuntime>::default())
            .expect("mock adapter should register");
        orchestrator
    }

    #[test]
    fn owns_start_stop_state_and_active_count() {
        let mut orchestrator = orchestrator();
        let started = orchestrator
            .start(&profile("alpha"))
            .expect("server should start");
        assert_eq!(started.desired_state, DesiredServerState::Running);
        assert_eq!(started.state, ServerState::Running);
        assert_eq!(orchestrator.active_count(), 1);
        assert_eq!(
            orchestrator
                .snapshot(&ServerId("alpha".into()))
                .map(|snapshot| snapshot.state),
            Some(ServerState::Running)
        );

        let stopped = orchestrator
            .stop(&ServerId("alpha".into()))
            .expect("server should stop");
        assert_eq!(stopped.desired_state, DesiredServerState::Stopped);
        assert_eq!(stopped.state, ServerState::Stopped);
        assert_eq!(orchestrator.active_count(), 0);
        assert_eq!(orchestrator.snapshots().len(), 1);
    }

    #[test]
    fn rejects_duplicate_start_and_non_running_stop() {
        let mut orchestrator = orchestrator();
        orchestrator
            .start(&profile("alpha"))
            .expect("server should start");
        assert!(matches!(
            orchestrator.start(&profile("alpha")),
            Err(RuntimeError::AlreadyRunning(_))
        ));
        orchestrator
            .stop(&ServerId("alpha".into()))
            .expect("server should stop");
        assert!(matches!(
            orchestrator.stop(&ServerId("alpha".into())),
            Err(RuntimeError::NotRunning(_))
        ));
    }

    #[test]
    fn supports_two_servers_and_independent_stop() {
        let mut orchestrator = orchestrator();
        orchestrator
            .start(&profile("alpha"))
            .expect("first server should start");
        orchestrator
            .start(&profile("beta"))
            .expect("second server should start");
        assert_eq!(orchestrator.active_count(), 2);
        orchestrator
            .stop(&ServerId("alpha".into()))
            .expect("first server should stop");
        assert_eq!(orchestrator.active_count(), 1);
        assert!(orchestrator.is_active(&ServerId("beta".into())));
    }

    #[test]
    fn polling_represents_runtime_failure_and_terminal_state() {
        let mut orchestrator = ServerOrchestrator::default();
        orchestrator
            .register_adapter(Box::new(MockRuntime {
                observations: HashMap::new(),
                fail_on_poll: true,
            }))
            .expect("mock adapter should register");
        orchestrator
            .start(&profile("alpha"))
            .expect("server should initially start");

        let failed = orchestrator
            .snapshot(&ServerId("alpha".into()))
            .expect("failed snapshot should remain queryable");
        assert_eq!(failed.state, ServerState::Failed);
        assert_eq!(failed.desired_state, DesiredServerState::Running);
        assert_eq!(
            failed.last_error.as_deref(),
            Some("synthetic runtime failure")
        );
        assert_eq!(orchestrator.active_count(), 0);
        assert!(orchestrator
            .events()
            .iter()
            .any(|event| event.kind == RuntimeEventKind::Failed));
    }

    #[test]
    fn rejects_duplicate_adapter_registration() {
        let mut orchestrator = orchestrator();
        assert!(matches!(
            orchestrator.register_adapter(Box::<MockRuntime>::default()),
            Err(RuntimeError::AdapterAlreadyRegistered(_))
        ));
    }
}
