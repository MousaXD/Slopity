# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 002: pause Windows CI and use the Pop!_OS runner

**Status:** IN PROGRESS  
**Declared:** 2026-08-05

### Scope

Pause Windows compilation until the application is functionally mature and the repository is public. Route active Rust, Linux Tauri, and Android CI work to the connected self-hosted Pop!_OS runner.

### Non-goals

- Removing Windows portability from the Rust/Tauri architecture.
- Shipping or testing a Windows application during this step.
- Publishing the repository.
- Implementing a real server runtime.

### Risks

- The self-hosted runner may be missing Rust, Node.js, WebKitGTK, Java, Android SDK, or NDK dependencies.
- Multiple jobs cannot run concurrently on a single runner unless multiple runner services are registered.
- Self-hosted job cleanup and toolchain maintenance become repository-owner responsibilities.

### Acceptance checks

- No active job uses a GitHub-hosted Windows runner.
- Rust, Linux shell, and Android jobs select `[self-hosted, Linux, X64]`.
- Jobs run serially enough to avoid competing for the single runner.
- Documentation states that Windows remains architecturally supported but CI is deferred.

### Intended files

- `.github/workflows/ci.yml`
- `README.md`
- `TASK.md`
- `PROGRESS.md`

## Step 001: portable Rust and Tauri foundation

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Replace the Kotlin-first application foundation with a portable Rust workspace and Tauri 2 application shell targeting Windows, Linux, and Android. Preserve the product goal of hosting multiple server types through explicit runtime adapters rather than hardcoding Minecraft.

### Delivered

- Replaced the Android-only Gradle/Compose source tree with a Rust workspace.
- Added `slopity-core` with serializable profile models, lifecycle states, resource planning, validation, runtime availability, and runtime-adapter contracts.
- Added `slopity-runtime-local`, a desktop adapter using an explicit executable and structured arguments without a shell command string.
- Added a Tauri 2 shell with a static shared dashboard for Windows, Linux, and Android.
- Added a host-service plugin boundary that reports Android foreground-service support as pending rather than pretending it exists.
- Added sample JVM, Node.js, and native profiles that are all disabled and unavailable by default.
- Added Windows, Linux, Rust, and Android CI paths.
- Updated architecture, security, contribution, hook, and roadmap documentation for the Rust-first design.

### Checks performed in the generation environment

- Parsed all JSON files with Python's strict JSON parser.
- Parsed all TOML files with Python `tomllib`.
- Parsed XML files with Python's XML parser.
- Checked JavaScript syntax with `node --check`.
- Inspected the complete generated file tree and searched production Rust code for `unwrap(`, `expect(`, `sh -c`, and `cmd /C`.

### Checks not performed

- `cargo fmt`, `cargo test`, and `cargo clippy` were not executed because Rust was not installed and the environment had no network access to install it.
- Windows and Linux Tauri builds were not executed locally.
- Android initialization/build was not executed because the Android SDK, NDK, and Rust Android targets were unavailable.
- No on-device Android hosting test was performed.

### Known limitations

- The Android app shell is portable, but the native foreground-service implementation is intentionally not claimed as complete.
- No real server runtime is bundled or considered supported.
- The desktop adapter is an internal foundation and is not exposed as an unrestricted UI command.

### Follow-up

Step 002 should implement versioned Rust profile persistence and CRUD before any real runtime download or server execution UI.

## Step 000: repository foundation

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Create a build-oriented Android starter repository for a multi-server phone host. Establish honest runtime boundaries, device capability advice, foreground-service infrastructure, project documentation, agent workflow, and CI.

### Delivered

- Kotlin and Jetpack Compose Android prototype.
- Device capability probe and conservative advisor.
- Multi-server profile model and runtime boundaries.
- Foreground-service prototype and repository workflow.

### Historical note

Step 001 superseded the Kotlin-only implementation with a Rust/Tauri cross-platform foundation while preserving this commit history.
