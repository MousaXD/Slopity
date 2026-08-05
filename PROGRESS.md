# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 005: make Linux dependency validation non-interactive

**Status:** IN PROGRESS  
**Declared:** 2026-08-05

### Scope

Replace the self-hosted Linux job's interactive package installation with a deterministic dependency check. Use the Ayatana application-indicator development package required on current Ubuntu and Pop!_OS systems.

### Evidence

- Workflow run `31007274933` passed Rust formatting, all workspace tests, and Clippy with warnings denied.
- The Linux job reached the `Ensure Linux Tauri dependencies` step and blocked on `[sudo] password for mousa:`.
- Manual installation showed `libappindicator3-dev` conflicts with the already installed `libayatana-appindicator3-1` package family.
- Current Tauri Debian/Ubuntu prerequisites use `libayatana-appindicator3-dev`, not `libappindicator3-dev`.

### Intended changes

- Replace `libappindicator3-dev` with `libayatana-appindicator3-dev`.
- Remove `sudo apt-get` from GitHub Actions.
- Check required packages with `dpkg -s` and fail with a copyable administrator command when any are missing.
- Keep installation as an explicit host-administration action outside the unprivileged runner.
- Preserve Windows CI as paused.

### Acceptance checks

- CI never prompts for a sudo password.
- The dependency check passes after the documented packages are installed.
- Linux Tauri compilation proceeds to `npm install` and `npm run tauri:check`.

## Step 004: supply Tauri icon and clear compile warning

**Status:** PARTIAL  
**Declared:** 2026-08-05  
**Updated:** 2026-08-05

### Scope

Add the application icon required by `tauri::generate_context!()` and remove the unused `tauri::Manager` import reported by the first successful Rust compilation attempt.

### Non-goals

- Changing application behavior or runtime architecture.
- Designing the final production brand system or full platform icon set.
- Fixing later Clippy, Linux packaging, or Android failures before they are observed.
- Restoring Windows CI.

### Evidence

- Workflow run `31006736597` passed `cargo fmt --all -- --check`.
- `cargo test --workspace --all-features` compiled the shared core, local runtime adapter, Tauri plugin, and most Tauri dependencies.
- Compilation failed in `tauri::generate_context!()` because `apps/slopity/src-tauri/icons/icon.png` did not exist.
- The same run reported `use tauri::Manager;` as unused in the Tauri shell.

### Delivered

- Added a valid 512 by 512 RGBA PNG at the path expected by Tauri.
- Removed the unused `tauri::Manager` import.
- Kept the existing Tauri commands, setup hook, runtime catalog, and application behavior unchanged.

### Files changed

- `apps/slopity/src-tauri/icons/icon.png`
- `apps/slopity/src-tauri/src/lib.rs`
- `PROGRESS.md`

### Verification pending

- A new self-hosted workflow run must confirm Tauri can decode the icon and `cargo test` gets beyond `generate_context!()`.
- Clippy, Linux Tauri compilation, and Android validation remain pending behind the Rust job.

### Follow-up

Inspect the next runner result and declare a separate step for any newly observed compiler, Clippy, Linux, or Android issue.

## Step 003: stabilize Rust CI formatting gate

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Apply the exact `rustfmt` changes reported by the first self-hosted CI run so Rust tests, Clippy, Linux Tauri compilation, and Android validation can proceed.

### Delivered

- Applied the exact import ordering, line wrapping, function-signature layout, and assertion layout requested by Rust 1.89.0 `rustfmt`.
- Changed only formatting in the six files identified by the runner.
- Preserved runtime behavior, APIs, feature flags, and workflow structure.

### Verification performed

- Workflow run `31006736597` passed `cargo fmt --all -- --check` on the `pop-os` self-hosted runner.
- The workflow continued into `cargo test`, proving the formatting gate was cleared.

### Follow-up

Step 004 addresses the missing Tauri icon and unused import reported by the test compile.

## Step 002: pause Windows CI and use the Pop!_OS runner

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Pause Windows compilation until the application is functionally mature and the repository is public. Route active Rust, Linux Tauri, and Android CI work to the connected self-hosted Pop!_OS runner.

### Delivered

- Removed the Windows runner matrix and all active Windows CI work.
- Routed Rust quality checks, Linux Tauri compilation, and Android ARM64 validation to `[self-hosted, Linux, X64]`.
- Ordered the jobs as Rust, Linux, then Android so one self-hosted runner handles them without overlapping workloads.
- Added workflow concurrency cancellation for superseded runs on the same branch.
- Kept Windows in the portable Rust/Tauri architecture while explicitly deferring Windows compilation, packaging, and validation.
- Updated the README and roadmap to make Linux and Android the active platform priority.

### Verification performed

- Inspected the workflow for `windows-latest`, `windows-2025`, and Windows matrix entries; none remain.
- Confirmed all active jobs use the self-hosted Linux X64 label set.
- Confirmed the Linux job depends on Rust and the Android job depends on Linux.
- Parsed the edited workflow as YAML and reviewed shell-array quoting in the dependency-install step.

### Verification pending

- The runner may still require passwordless `sudo` and platform packages before the Linux job passes.
- Android SDK, NDK, Rust targets, disk space, and environment behavior will be proven by the first worker run that reaches those jobs.

### Follow-up

Resume Phase 1 with versioned Rust profile persistence and CRUD after the foundation CI is stable. Restore Windows CI only after the application is functionally mature and the repository is public.

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

Stabilize Linux and Android CI, then begin versioned Rust profile persistence and CRUD before any real runtime download or server execution UI.

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
