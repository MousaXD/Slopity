# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 009: publish installable Android debug artifacts

**Status:** PARTIAL  
**Declared:** 2026-08-05  
**Updated:** 2026-08-05

### Scope

Preserve the successful Android debug APK and AAB as downloadable GitHub Actions artifacts after every green Android build. Make missing output files fail the job instead of silently producing an empty artifact.

### Non-goals

- Release signing, production keystores, Google Play publication, stable releases, or automatic installation on a device.
- Linux packaging, Windows packaging, or public release automation.
- Treating the debug artifact as proof of Android background durability.

### Risks

- Tauri generates Android output under `src-tauri/gen`, so the upload paths must match the proven build output exactly.
- The self-hosted runner cleans generated files between jobs, making artifact upload part of the Android job mandatory.
- Debug builds are installable test packages, not production releases, and must remain clearly labeled.

### Delivered

- Added an Android post-build upload step using `actions/upload-artifact@v7`.
- Included the generated universal debug APK and universal debug AAB in one artifact.
- Named each artifact with the tested commit SHA so packages cannot be confused across runs.
- Made missing APK or AAB output fail the workflow through `if-no-files-found: error`.
- Set explicit 14-day retention for temporary device-testing packages.
- Disabled extra artifact compression because APK and AAB files are already compressed archives.

### Files changed

- `.github/workflows/ci.yml`
- `PROGRESS.md`

### Verification pending

- Rust formatting, workspace tests, and strict Clippy must remain green.
- Linux Tauri compilation must remain green.
- Android ARM64 compilation must still produce the expected APK and AAB paths.
- The upload step must complete and expose a non-expired downloadable artifact containing both files.

### Follow-up

Inspect the replacement workflow run and download the resulting artifact. Install the APK on an ARM64 Android device and perform the Step 008 notification, reachability, background-survival, stop, and port-release smoke test.

## Step 008: add the first built-in HTTP server

**Status:** PARTIAL  
**Declared:** 2026-08-05  
**Updated:** 2026-08-05

### Scope

Implement Slopity's first real hosted workload as a harmless built-in Rust HTTP server shared by Linux and Android. Add explicit start, stop, observed state, bounded logs, loopback/LAN binding, usable URLs, port-conflict reporting, and an Android foreground-service notification while hosting is active.

### Non-goals

- Downloading or executing Java, Node.js, Python, PHP, game-server, plugin, mod, or arbitrary native payloads.
- Shell command execution, terminal input, static-folder selection, uploads, remote administration, TLS, authentication, or internet exposure.
- Claiming durable Android hosting until a real device test proves notification behavior and survival outside the foreground UI.
- Windows CI, packaging, or platform-specific implementation.

### Delivered

- Added `RuntimeKind::BuiltInHttp` and a disabled loopback-first sample profile.
- Added a Tauri-independent `slopity-runtime-http` crate using a structured lifecycle manager and a fixed harmless HTTP response.
- Added start, stop, observed state, request counts, usable URLs, duplicate-start rejection, clean port release, and port-bind error reporting.
- Added bounded structured logs with a 200-entry retention cap.
- Added unit tests for serving the health endpoint, graceful stop and port release, duplicate starts, occupied ports, and bounded logs.
- Added Tauri commands for listing, starting, and stopping built-in HTTP servers.
- Prevented running profiles from being edited, disabled, or deleted until they stop.
- Added UI controls for built-in HTTP profile creation, start/stop, observed state, URLs, request counts, and recent logs.
- Added a Kotlin Android foreground service, persistent low-priority notification, Tauri mobile bridge, required manifest permissions, and the Android 14 `specialUse` declaration.
- Kept Android durable-hosting capability false until a real-device background test passes.
- Kept external runtime providers unavailable and excluded shell execution, uploads, and arbitrary content serving.

### Files changed

- `Cargo.toml`
- `crates/slopity-core/src/model.rs`
- `crates/slopity-core/src/validation.rs`
- `crates/slopity-runtime-http/Cargo.toml`
- `crates/slopity-runtime-http/src/lib.rs`
- `apps/slopity/src-tauri/Cargo.toml`
- `apps/slopity/src-tauri/src/lib.rs`
- `apps/slopity/web/index.html`
- `apps/slopity/web/app.js`
- `apps/slopity/web/styles.css`
- `plugins/tauri-plugin-slopity-host/build.rs`
- `plugins/tauri-plugin-slopity-host/src/lib.rs`
- `plugins/tauri-plugin-slopity-host/src/mobile.rs`
- `plugins/tauri-plugin-slopity-host/android/build.gradle.kts`
- `plugins/tauri-plugin-slopity-host/android/src/main/AndroidManifest.xml`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostPlugin.kt`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostForegroundService.kt`
- `TASK.md`
- `PROGRESS.md`

### Verification performed

- Workflow run `31026665154` passed `cargo fmt --all -- --check`.
- The same run passed all 14 workspace tests, including the built-in HTTP response, graceful stop and port release, duplicate-start rejection, occupied-port reporting, and bounded-log tests.
- The same run passed `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- Linux Tauri shell compilation passed with the live HTTP manager, managed state, Tauri commands, and shared UI.
- Android initialization passed with the Tauri mobile plugin included.
- The ARM64 debug APK build passed with the Rust listener, Kotlin foreground service, manifest permissions, notification implementation, and `specialUse` declaration compiled into the application.
- Tauri produced `app-universal-debug.apk` and `app-universal-debug.aab` under the generated Android build output.

### Verification pending

- Interactive Linux profile/create/start/request/log/stop and persistence smoke test.
- ARM64 Android installation and notification visibility proof.
- Android loopback and LAN reachability while the UI is foregrounded.
- Android reachability while the UI is backgrounded, followed by clean stop and port-release proof.

### Known limitations

- The HTTP server serves only a fixed development page and `/health`; static folder hosting is intentionally deferred.
- Runtime state is in memory and resets to stopped after application process restart.
- LAN URL discovery depends on an active IPv4 route; loopback URL remains available regardless.
- Android foreground-service compilation does not prove OEM background behavior or Google Play policy acceptance.
- The successful Android build reports a non-blocking duplicate namespace warning between the application and plugin modules.
- The workflow reports non-blocking action and Gradle deprecation warnings that require a separate CI-maintenance step.
- No force-stop, crash-loop protection, TLS, authentication, remote management, file picker, or upload flow exists yet.

### Follow-up

Step 009 preserves the debug APK and AAB as downloadable CI artifacts. After that, perform the Linux smoke test and install the APK on an ARM64 device before claiming Android durable hosting.

## Step 007: remove the blocking GitHub Rust cache

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Remove `Swatinem/rust-cache` from the Rust-quality job on the persistent self-hosted Pop!_OS runner so CI reaches Slopity's formatting, tests, and Clippy gates without waiting on a remote target-cache restore.

### Evidence

- Step 006 workflow run `31011407485` checked out the branch and installed the Rust toolchain successfully.
- The run remained inside `Swatinem/rust-cache@v2` for several minutes and never reached the progress ledger or any Cargo command.
- The self-hosted machine already retains its Cargo registry and toolchains under the runner user's home directory between jobs.
- The cache action was blocking validation before providing value.

### Delivered

- Removed `Swatinem/rust-cache@v2` from the Rust-quality job.
- Preserved the progress-ledger guard, formatting, tests, Clippy, Linux build, and Android build order.
- Left Cargo's normal registry and toolchain reuse on the persistent self-hosted machine untouched.

### Files changed

- `.github/workflows/ci.yml`
- `PROGRESS.md`

### Verification performed

- Replacement workflow run `31012340919` reached the progress-ledger check and `cargo fmt --all -- --check` immediately after Rust toolchain setup.
- No GitHub cache restore step was present in the replacement run.
- The runner exposed Slopity's actual formatting result instead of stalling in cache plumbing.
- Workflow concurrency superseded the obsolete cache-blocked validation path.

### Follow-up

Continue Step 006 validation through formatting, tests, Clippy, Linux, and Android.

## Step 006: add versioned profile persistence and CRUD

**Status:** PARTIAL  
**Declared:** 2026-08-05  
**Updated:** 2026-08-05

### Scope

Implement the first durable product slice: a versioned Rust profile document, filesystem persistence, validated create/edit/clone/enable/disable/delete operations, Tauri commands, and a small profile-management interface shared by Linux and Android.

### Non-goals

- Downloading or executing any server runtime.
- Import/export, aggregate memory reservation, remote management, or runtime lifecycle controls.
- Android foreground-service hosting proof.
- Windows CI, packaging, or platform-specific work.

### Delivered

- Added schema-v1 JSON profile documents with explicit rejection of malformed data and unsupported future versions.
- Added a Tauri-independent Rust `ProfileStore` that loads, seeds, validates, persists, and reloads profile collections.
- Added create, update, clone, enable, disable, and delete operations that write before replacing in-memory state.
- Added collection validation for duplicate IDs, duplicate ports, and existing profile validation errors.
- Added replacement-write handling with a rollback path for platforms that cannot rename over an existing destination.
- Added Rust tests for initial creation, malformed/future documents, durable CRUD, duplicate-port rejection, and write-failure state preservation.
- Added Tauri state and commands backed by the writable application-data directory on Linux and Android.
- Replaced sample-only profile rendering with a shared profile editor and create/edit/clone/toggle/delete flows.
- Kept executable paths optional and made the UI explicit that enabling configuration does not start a server.
- Applied the exact two formatting changes reported by Rust 1.89.0 in workflow run `31012340919`.
- Fixed cloning to select the next free port instead of copying a conflicting source port.
- Added assertions that the automatically selected clone port persists across store reloads.

### Files changed

- `crates/slopity-core/Cargo.toml`
- `crates/slopity-core/src/lib.rs`
- `crates/slopity-core/src/profile_store.rs`
- `apps/slopity/src-tauri/src/lib.rs`
- `apps/slopity/web/index.html`
- `apps/slopity/web/app.js`
- `apps/slopity/web/styles.css`
- `TASK.md`
- `PROGRESS.md`

### Verification performed

- Workflow run `31012340919` reached `cargo fmt --all -- --check` without infrastructure blockage.
- Rustfmt reported only export wrapping and one `matches!` layout in the new profile-store files; those exact mechanical changes were applied.
- Workflow run `31012808047` passed formatting and ran nine Rust tests. Eight passed; the durable CRUD test exposed the cloned-port conflict.
- Workflow run `31013262338` passed formatting, all nine Rust tests, Clippy with warnings denied, and Linux Tauri compilation after the clone-port fix.
- Android setup, SDK/NDK verification, dependency installation, and Tauri Android initialization all passed in the same run.
- The first Android ARM64 APK attempt failed during the final build step. GitHub's decoded job-log blob was unavailable, and the requested job rerun remained queued rather than starting.
- Fresh workflow run `31015265578` passed formatting, all workspace tests, Clippy with warnings denied, Linux Tauri compilation, Android initialization, and the ARM64 debug APK build.

### Verification pending

- UI create, edit, clone, enable, disable, delete, and restart persistence flows still require an interactive smoke test.

### Known limitations

- Profile import/export and aggregate resource reservations remain separate roadmap items.
- The store is process-local and serialized behind one mutex; multi-process writers are not supported.
- Windows persistence behavior remains architectural only while Windows CI is paused.
- No profile operation downloads, launches, or claims support for a runtime.

### Follow-up

Perform the interactive persistence smoke test while Step 008 adds the first harmless built-in hosted workload.

## Step 005: make Linux dependency validation non-interactive

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Replace the self-hosted Linux job's interactive package installation with a deterministic dependency check. Use the Ayatana application-indicator development package required on current Ubuntu and Pop!_OS systems.

### Evidence

- Workflow run `31007274933` passed Rust formatting, all workspace tests, and Clippy with warnings denied.
- The Linux job reached the dependency step and blocked on `[sudo] password for mousa:`.
- Manual installation showed `libappindicator3-dev` conflicts with the already installed `libayatana-appindicator3-1` package family.
- Current Tauri Debian/Ubuntu prerequisites use `libayatana-appindicator3-dev`, not `libappindicator3-dev`.

### Delivered

- Replaced the legacy `libappindicator3-dev` requirement with `libayatana-appindicator3-dev`.
- Added `scripts/check-linux-prerequisites.sh` to verify Tauri's Linux packages without privilege escalation.
- Removed every `sudo` and `apt-get` invocation from GitHub Actions.
- Added `docs/ci-host-prerequisites.md` with the one-time Pop!_OS/Ubuntu administrator command.
- Made missing-package failures print a copyable installation command and exit immediately instead of hanging.

### Files changed

- `.github/workflows/ci.yml`
- `scripts/check-linux-prerequisites.sh`
- `docs/ci-host-prerequisites.md`
- `PROGRESS.md`

### Verification performed

- Workflow run `31008114077` passed the non-interactive Linux prerequisite check.
- The same run passed Linux Tauri compilation without bundling.
- Android SDK and NDK installation completed non-interactively.
- Tauri Android initialization completed successfully.
- The ARM64 debug APK build completed successfully.
- Rust formatting, all workspace tests, and Clippy with warnings denied also passed in the same run.

### Follow-up

Step 006 begins versioned Rust profile persistence and CRUD. Windows remains deferred.

## Step 004: supply Tauri icon and clear compile warning

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Add the application icon required by `tauri::generate_context!()` and remove the unused `tauri::Manager` import reported by the first successful Rust compilation attempt.

### Delivered

- Added a valid 512 by 512 RGBA PNG at the path expected by Tauri.
- Removed the unused `tauri::Manager` import.
- Kept the existing Tauri commands, setup hook, runtime catalog, and application behavior unchanged.

### Verification performed

- Workflow run `31007274933` passed `cargo fmt --all -- --check`.
- The same run passed `cargo test --workspace --all-features`.
- The same run passed `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- This confirms Tauri decoded the icon and the unused import warning was removed.

### Follow-up

Step 005 removes the interactive Linux dependency installation discovered after Rust quality passed.

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

### Verification pending

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
- Added a host-service capability plugin boundary that reports Android foreground-service support as pending rather than pretending it exists.
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
