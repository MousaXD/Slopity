# Architecture

## Purpose

PocketHost is an Android control plane for multiple server workloads. The application owns profiles, resource policy, lifecycle state, logs, storage, and user controls. Runtime adapters own the mechanics of executing a specific engine.

## Layer boundaries

```text
Compose UI
    |
DashboardViewModel
    |
ServerOrchestrator ---- DeviceCapabilityProbe
    |
RuntimeRegistry
    |
RuntimeAdapter implementations
    |
Android-compatible runtime or native bridge
```

### UI

Displays desired state, observed state, capability guidance, and explicit failures. The UI must not infer that a server is running merely because a start request was accepted.

### Domain

Contains platform-light models such as `ServerProfile`, `ServerRuntime`, `DeviceCapabilities`, `HostingPlan`, and preflight decisions.

### Orchestration

Checks aggregate policy before delegating to an adapter. Future work must add a real lifecycle state machine, port ownership, memory reservations, crash-loop protection, and durable observed state.

### Runtime adapters

A runtime adapter is responsible for one execution family, such as JVM, PHP, Node.js, Python, or reviewed native binaries. An adapter must report availability and the reason for unavailability. It must never silently fall back to an unrelated execution path.

### Foreground service

The foreground service provides the Android-visible lifecycle envelope for active hosting. The current service starts no server process. Future orchestration will bind active sessions to this envelope and update the notification from observed state.

## Storage plan

Use app-private storage by default:

```text
files/
  profiles/
  packages/
  instances/<profile-id>/
  backups/
  logs/
```

External storage export should use the Storage Access Framework and explicit user-selected destinations.

## Runtime package direction

A future package manifest should include at least:

- Schema version.
- Package and runtime IDs.
- Human-readable name and license metadata.
- Supported Android API levels and ABIs.
- Download provenance and SHA-256 hashes.
- Structured entry point and arguments.
- Default and required ports.
- Minimum and recommended RAM.
- Health-check method.
- Graceful shutdown method.
- Data and log paths.

## Why process execution is deferred

Android is not ordinary desktop Linux. Runtime work must account for Bionic, app sandboxing, executable placement restrictions, SELinux, lifecycle limits, architecture, dynamic-code policy, and distribution licensing. The repository keeps these decisions behind adapters so an early shortcut does not poison the whole product.
