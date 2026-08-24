# Slopity implementation roadmap

## Platform priority

- Linux and Android are the active development and CI targets.
- Windows remains an architectural target, but Windows CI, packaging, and platform-specific work are deferred until the application is functionally mature and the repository is public.

## Phase 0: portable foundation

- [x] Rust workspace and shared core crate.
- [x] Multi-runtime profile model and validation.
- [x] Conservative device resource planner.
- [x] Desktop process adapter boundary.
- [x] Tauri 2 shell with Linux, Android, and future Windows portability.
- [x] Host-service capability plugin boundary.
- [x] Rust, Linux, and Android CI on GitHub-hosted runners.
- [ ] Restore Windows CI and packaging after functional maturity and public release.

## Phase 1: durable profiles and UI

- [x] Add versioned profile persistence in Rust.
- [x] Add create, edit, clone, enable, disable, and delete flows.
- [ ] Add aggregate port and memory reservations.
- [ ] Add profile import/export without executable payloads.
- [ ] Add actionable compatibility warnings.

## Phase 2: Android hosting proof

- [x] Implement the Kotlin foreground-service side of `tauri-plugin-slopity-host`.
- [x] Keep hosting user-initiated and visibly controlled by a persistent notification.
- [x] Build a harmless loopback-first Rust HTTP test server with opt-in LAN binding.
- [ ] Prove start, log capture, graceful stop, forced stop, crash detection, and notification behavior on ARM64 Android.
- [ ] Prove the server remains reachable while the UI is backgrounded and the foreground notification remains visible.
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
- [ ] Reproducible signed Linux and Android release pipeline.
- [ ] Make the repository public with PolyForm Noncommercial License 1.0.0 and clear contribution policy.
- [ ] Restore and validate Windows CI, packaging, and platform behavior.
