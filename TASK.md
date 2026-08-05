# PocketHost implementation roadmap

The roadmap is ordered. Agents may split a task into smaller numbered steps, but must not skip architecture or security gates merely to make a button appear functional.

## Phase 0: repository foundation

- [x] Android application skeleton.
- [x] Compose dashboard.
- [x] Device capability probe and conservative advisor.
- [x] Multi-server profile model.
- [x] Runtime adapter and orchestration boundaries.
- [x] Foreground host service foundation.
- [x] Agent workflow, progress ledger, hooks, and CI.

## Phase 1: durable profiles and UI

- [ ] Replace the in-memory repository with Room or another reviewed local store.
- [ ] Add create, edit, clone, enable, disable, and delete profile flows.
- [ ] Validate ports, memory budgets, working directories, and runtime-specific fields.
- [ ] Add aggregate resource budgeting across enabled profiles.
- [ ] Show actionable capability warnings rather than hardcoded tiers alone.

## Phase 2: runtime execution spike

This phase must produce evidence before choosing an implementation.

- [ ] Evaluate Android-compatible process strategies for each runtime class.
- [ ] Document Bionic, executable-mount, SELinux, JNI, namespace, and app-store constraints.
- [ ] Build a harmless loopback-only test server adapter.
- [ ] Prove start, health check, log capture, command input, graceful stop, forced stop, and crash detection on a real ARM64 phone.
- [ ] Record battery, temperature, throttling, and memory behavior for at least one hour.
- [ ] Select the first production runtime strategy through an ADR.

## Phase 3: trusted server packages

- [ ] Define a versioned package manifest schema.
- [ ] Require checksums, source URLs, licenses, runtime requirements, ports, and entry points.
- [ ] Add safe archive extraction with path traversal protection.
- [ ] Add storage quotas and cleanup.
- [ ] Add explicit trust warnings for plugins, mods, and scripts.

## Phase 4: orchestration

- [ ] Persist desired and observed state separately.
- [ ] Add per-server lifecycle state machine.
- [ ] Enforce aggregate RAM and port reservations.
- [ ] Add structured logs and bounded retention.
- [ ] Add graceful shutdown deadlines and crash-loop protection.
- [ ] Add notification controls for stop-all and individual server actions.

## Phase 5: Minecraft Java adapter

- [ ] Resolve licensing and distribution approach for the Android-compatible JVM.
- [ ] Add a Java runtime provider with architecture checks.
- [ ] Add Paper server package preparation without redistributing prohibited artifacts.
- [ ] Implement EULA acceptance flow.
- [ ] Add memory, view-distance, simulation-distance, and pregeneration guidance.
- [ ] Test sustained hosting on supported devices.

## Phase 6: additional adapters

Each adapter gets its own threat model, package rules, and on-device validation.

- [ ] PHP adapter, with PocketMine-MP as an initial workload candidate.
- [ ] Node.js adapter.
- [ ] Python adapter.
- [ ] Reviewed native-binary adapter.
- [ ] Custom runtime plug-in API only after the fixed adapters are stable.

## Phase 7: operations

- [ ] Console and command history.
- [ ] World/data import and export.
- [ ] Atomic backups and restore verification.
- [ ] Health checks and local network address display.
- [ ] Optional authenticated remote management.
- [ ] Temperature and battery policies.
- [ ] Update channels with rollback.

## Phase 8: release hardening

- [ ] Instrumented lifecycle tests.
- [ ] Static analysis and dependency review.
- [ ] Reproducible signed release pipeline.
- [ ] Privacy policy and Play foreground-service declaration.
- [ ] Accessibility and localization review.
- [ ] Device compatibility matrix.
