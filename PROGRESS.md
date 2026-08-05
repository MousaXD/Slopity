# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain any correction in a new entry.

## Step 001: portable Rust and Tauri foundation

**Status:** IN PROGRESS  
**Declared:** 2026-08-05

### Scope

Replace the Kotlin-first application foundation with a portable Rust workspace and Tauri 2 application shell targeting Windows, Linux, and Android. Preserve the product goal of hosting multiple server types through explicit runtime adapters rather than hardcoding Minecraft.

### Intended architecture

- A platform-neutral `slopity-core` Rust crate for profiles, device budgets, lifecycle state, validation, and runtime contracts.
- A `slopity-runtime-local` Rust crate for safe desktop child-process execution without shell command strings.
- A Tauri 2 shell with a static lightweight web interface shared by desktop and Android.
- An Android native plugin boundary for the user-visible foreground host service and future Kotlin lifecycle integration.
- CI jobs for Rust quality checks, Windows and Linux Tauri compilation, and Android project initialization/build validation.

### Non-goals

- Bundling Java, Node.js, Python, PHP, Minecraft, or any real server engine.
- Claiming Android runtime execution is complete before an ARM64 on-device test.
- iOS hosting support.
- Remote administration or public-network exposure.
- Merging the branch into `main`.

### Risks

- Android foreground-service behavior requires Kotlin integration even though orchestration is Rust-first.
- Tauri mobile builds require Android SDK/NDK tooling that is unavailable in the current execution container.
- Desktop process management and Android process execution have different security and lifecycle constraints.

### Acceptance checks

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-features`
- `cargo clippy --workspace --all-targets --all-features -- -D warnings`
- Tauri configuration and static frontend files are syntactically valid.
- GitHub Actions define Windows, Linux, and Android verification paths.
- The UI truthfully reports runtime adapters as unavailable until installed or proven.

### Intended files

- Root Rust workspace configuration and toolchain files.
- `crates/slopity-core/`
- `crates/slopity-runtime-local/`
- `apps/slopity/src-tauri/`
- `apps/slopity/web/`
- `plugins/tauri-plugin-slopity-host/`
- Cross-platform CI workflows, scripts, and architecture documentation.
- Removal of the superseded Kotlin-only application skeleton.

## Step 000: repository foundation

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Create a build-oriented Android starter repository for a multi-server phone host. Establish honest runtime boundaries, device capability advice, foreground-service infrastructure, project documentation, agent workflow, and CI.

### Non-goals

- Running Minecraft or any other real server engine.
- Downloading or bundling executable runtimes.
- Persisting user-created profiles.
- Remote access or authentication.
- Publishing an APK.

### Delivered

- Single-module Android application using Kotlin and Jetpack Compose.
- Capability probe for total/available memory, CPU cores, ABI, storage, and current thermal status.
- Conservative hosting plan calculator with unit tests.
- Multi-runtime server profile model and sample profiles.
- Runtime adapter registry whose initial adapters report unavailable truthfully.
- Orchestrator preflight decisions for memory, ports, architecture, and runtime availability.
- User-initiated foreground hosting service declared as `specialUse`.
- Persistent notification with an explicit stop action.
- `AGENTS.md`, `TASK.md`, architecture and security documentation.
- Git hook and CI guard requiring `PROGRESS.md` alongside implementation changes.
- Gradle wrapper bootstrap with a pinned checksum.

### Verification performed

- Generated file tree inspected.
- XML resources parsed with a strict XML parser.
- TOML version catalog parsed.
- Gradle wrapper downloader compiled with JDK 21.
- Unit-test source and build configuration inspected.

### Verification not performed

- Android compilation, lint, unit tests, and APK assembly were not executed in the generation environment because an Android SDK was not available.
- No on-device test was performed.

### Next step

Start Phase 1 with a documentation-only planning commit for profile persistence and CRUD.
