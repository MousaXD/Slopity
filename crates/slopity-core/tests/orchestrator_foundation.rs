use slopity_core::{
    authorize_start, CapabilitySnapshot, DesiredServerState, NetworkScope, RuntimeAdapter,
    RuntimeAvailability, RuntimeError, RuntimeExitReason, RuntimeHandle, RuntimeIdentity,
    RuntimeKind, RuntimeObservation, RuntimeRequest, ServerId, ServerOrchestrator, ServerProfile,
    ServerState, StartAdmissionPermit,
};
use std::collections::HashSet;

#[derive(Default)]
struct StableAdapter {
    active: HashSet<ServerId>,
}

impl RuntimeAdapter for StableAdapter {
    fn runtime_kind(&self) -> RuntimeKind {
        RuntimeKind::BuiltInHttp
    }

    fn runtime_identity(&self) -> RuntimeIdentity {
        RuntimeIdentity {
            runtime: RuntimeKind::BuiltInHttp,
            adapter: "foundation-stable".into(),
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
        if !self.active.insert(request.profile.id.clone()) {
            return Err(RuntimeError::AlreadyRunning(request.profile.id.0));
        }
        Ok(RuntimeHandle {
            server_id: request.profile.id,
            process_id: None,
        })
    }

    fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError> {
        if self.active.remove(server_id) {
            Ok(())
        } else {
            Err(RuntimeError::NotRunning(server_id.0.clone()))
        }
    }

    fn observe(&mut self, server_id: &ServerId) -> Option<RuntimeObservation> {
        self.active
            .contains(server_id)
            .then(|| RuntimeObservation::running(server_id.clone()))
    }

    fn observations(&mut self) -> Vec<RuntimeObservation> {
        self.active
            .iter()
            .cloned()
            .map(RuntimeObservation::running)
            .collect()
    }
}

#[derive(Default)]
struct CompletingAdapter {
    active: HashSet<ServerId>,
}

impl RuntimeAdapter for CompletingAdapter {
    fn runtime_kind(&self) -> RuntimeKind {
        RuntimeKind::BuiltInHttp
    }

    fn runtime_identity(&self) -> RuntimeIdentity {
        RuntimeIdentity {
            runtime: RuntimeKind::BuiltInHttp,
            adapter: "foundation-completing".into(),
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
        self.active.insert(request.profile.id.clone());
        Ok(RuntimeHandle {
            server_id: request.profile.id,
            process_id: None,
        })
    }

    fn stop(&mut self, server_id: &ServerId) -> Result<(), RuntimeError> {
        self.active.remove(server_id);
        Ok(())
    }

    fn observe(&mut self, server_id: &ServerId) -> Option<RuntimeObservation> {
        self.active
            .contains(server_id)
            .then(|| RuntimeObservation::running(server_id.clone()))
    }

    fn observations(&mut self) -> Vec<RuntimeObservation> {
        self.active
            .iter()
            .cloned()
            .map(|server_id| {
                RuntimeObservation::stopped(
                    server_id,
                    RuntimeExitReason::Completed,
                    "test runtime exited cleanly",
                )
            })
            .collect()
    }
}

fn profile(id: impl Into<String>, port: u16) -> ServerProfile {
    let id = id.into();
    ServerProfile {
        id: ServerId(id.clone()),
        name: id,
        runtime: RuntimeKind::BuiltInHttp,
        executable: None,
        arguments: Vec::new(),
        working_directory: None,
        port,
        memory_mib: 128,
        network_scope: NetworkScope::Loopback,
        enabled: true,
    }
}

fn permit(profile: &ServerProfile) -> StartAdmissionPermit {
    authorize_start(
        &CapabilitySnapshot {
            platform: "linux".into(),
            architecture: "x86_64".into(),
            logical_cpus: 8,
            total_memory_mib: Some(8_192),
            available_memory_mib: Some(6_000),
        },
        std::slice::from_ref(profile),
        &[],
        profile,
        &RuntimeAvailability {
            available: true,
            runtime: RuntimeKind::BuiltInHttp,
            reason: "test adapter".into(),
        },
    )
    .expect("fixture profile should pass admission")
}

#[test]
fn terminal_exit_observation_preserves_desired_state_and_exit_reason() {
    let mut orchestrator = ServerOrchestrator::default();
    orchestrator
        .register_adapter(Box::<CompletingAdapter>::default())
        .expect("adapter should register");
    let server = profile("completed", 31_000);

    let started = orchestrator
        .start(&server, permit(&server))
        .expect("runtime should start");
    assert_eq!(started.state, ServerState::Running);

    let observed = orchestrator
        .snapshot(&server.id)
        .expect("terminal observation should remain queryable");
    assert_eq!(observed.desired_state, DesiredServerState::Running);
    assert_eq!(observed.state, ServerState::Stopped);
    let exit = observed.exit.expect("terminal exit should be retained");
    assert_eq!(exit.reason, RuntimeExitReason::Completed);
    assert_eq!(exit.message, "test runtime exited cleanly");
}

#[test]
fn runtime_event_retention_is_bounded_and_sequence_order_is_deterministic() {
    let mut orchestrator = ServerOrchestrator::default();
    orchestrator
        .register_adapter(Box::<StableAdapter>::default())
        .expect("adapter should register");

    for index in 0_u16..70 {
        let server = profile(format!("server-{index:03}"), 32_000 + index);
        orchestrator
            .start(&server, permit(&server))
            .expect("runtime should start");
        orchestrator.stop(&server.id).expect("runtime should stop");
    }

    let events = orchestrator.events();
    assert_eq!(events.len(), 256);
    assert_eq!(events.first().map(|event| event.sequence), Some(25));
    assert_eq!(events.last().map(|event| event.sequence), Some(280));
    assert!(events
        .windows(2)
        .all(|pair| pair[1].sequence == pair[0].sequence + 1));
}
