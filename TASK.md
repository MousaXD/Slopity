# Slopity implementation roadmap

## Phase 0: portable foundation

- [x] Rust workspace and shared core crate.
- [x] Multi-runtime profile model and validation.
- [x] Conservative device resource planner.
- [x] Desktop process adapter boundary.
- [x] Tauri 2 shell for Windows, Linux, and Android.
- [x] Host-service capability plugin boundary.
- [x] Cross-platform CI and agent workflow.

## Phase 1: durable profiles and UI

- [ ] Add versioned profile persistence in Rust.
- [ ] Add create, edit, clone, enable, disable, and delete flows.
- [ ] Add aggregate port and memory reservations.
- [ ] Add profile import/export without executable payloads.
- [ ] Add actionable compatibility warnings.

## Phase 2: Android hosting proof

- [ ] Implement the Kotlin foreground-service side of `tauri-plugin-slopity-host`.
- [ ] Keep hosting user-initiated and visibly controlled by a persistent notification.
- [ ] Build a harmless loopback-only Rust test server.
- [ ] Prove start, log capture, command input, graceful stop, forced stop, and crash detection on ARM64 Android.
- [ ] Record device model, Android version, ABI, battery, temperature, memory, and throttling for at least one hour.
- [ ] Write an ADR selecting the Android process/runtime strategy.

## Phase 3: trusted packages

- [ ] Define a versioned package manifest schema.
- [ ] Require checksums, source URLs, licenses, runtime requirements, ports, and entry points.
- [ ] Add path-traversal-safe archive extraction.
- [ ] Add storage quotas and atomic cleanup.
- [ ] Add explicit trust warnings for plugins, mods, and scripts.

## Phase 4: orchestration

- [ ] Persist desired and observed state separately.
- [ ] Implement per-server lifecycle state machines.
- [ ] Add structured logs with bounded retention.
- [ ] Add graceful shutdown deadlines and crash-loop protection.
- [ ] Add notification controls and temperature/battery policies.

## Phase 5: runtime providers

- [ ] JVM provider and Paper preparation flow without redistributing prohibited artifacts.
- [ ] PHP provider, with PocketMine-MP as a candidate workload.
- [ ] Node.js provider.
- [ ] Python provider.
- [ ] Reviewed native-binary provider.

Each provider requires its own threat model, licensing review, integrity rules, and platform validation.

## Phase 6: operations and release

- [ ] Console and command history.
- [ ] Atomic backups and verified restore.
- [ ] Authenticated remote management.
- [ ] Reproducible signed release pipeline.
- [ ] Privacy policy, Play foreground-service declaration, accessibility, localization, and device matrix.
