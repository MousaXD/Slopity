# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 018: reconcile workload foundation

**Status:** PARTIAL  
**Declared:** 2026-08-18  
**Updated:** 2026-08-18

### Scope

Reconcile the compatible, still-desired foundation work from draft pull requests #5, #6, #9, and #10 into one coherent branch based on the current `main`. Make the platform-neutral `ServerOrchestrator` the authoritative lifecycle owner, retain structured desired/observed runtime state and bounded observations, integrate aggregate resource accounting and proven host telemetry, harden profile recovery/migrations/backend validation, and combine the Android foreground-host status/permission improvements behind the existing native plugin boundary.

### Non-goals

- Enabling Minecraft, Java/JVM workloads, Jellyfin, GitHub deployment, Node.js, Python, PHP, native-package workloads, custom commands, or arbitrary external runtimes.
- Registering the desktop local-process adapter as a supported runtime or reusing desktop `std::process` execution on Android.
- Adding shell command strings, package downloads, runtime installers, remote management, or trusted-package implementation.
- Claiming Android background durability, OEM survival, notification behavior, process-death recovery, or real-device hosting proof from compilation.
- Merging any source pull request or this integration branch, rebasing shared branches, force-pushing, or rewriting completed progress history.

### Delivered

- Added the platform-neutral `ServerOrchestrator` as the single authoritative lifecycle owner for the Tauri shell, with explicit desired and observed state, structured runtime identity, logs, terminal exits, failure observations, bounded deterministic events, adapter availability, and duplicate-adapter rejection.
- Kept the existing built-in Rust HTTP provider behind the generic runtime adapter contract and registered only that adapter. External runtime kinds remain unavailable configuration values and the desktop local-process adapter is not registered or exposed.
- Removed the parallel direct-`HttpServerManager` lifecycle ownership from the Tauri shell. Existing built-in HTTP commands now delegate through the orchestrator and explicitly reject profiles whose runtime kind is not `BuiltInHttp`.
- Added aggregate resource accounting with host reserve policy, safe memory budgets, active-or-reserved server accounting, deterministic port reservations/conflicts, CPU headroom warnings, and conservative unknown-telemetry behavior.
- Added proven host telemetry surfaces: Linux `/proc/meminfo` memory telemetry and Android system-service memory, battery, charging, battery-temperature, thermal-state, and free-storage telemetry. Missing values remain `None` instead of being guessed.
- Integrated profile-store interrupted-write recovery, primary/temporary/backup candidate inspection, backup recovery, parent-directory synchronization, schema-v0 to schema-v1 migration infrastructure, migration/recovery notices, and hard failure for unsupported future schemas.
- Integrated backend profile validation limits for IDs, names, argument counts and payload sizes, executable paths, working directories, duplicate IDs, and duplicate ports while keeping structural profile validity separate from runtime availability.
- Integrated Android foreground-host lifecycle/status improvements: POST_NOTIFICATIONS permission handling, native service status, active-server counts, start/update behavior, conservative notification visibility, a pending notification stop request, and `START_NOT_STICKY` process-lifecycle honesty.
- Added a dashboard resource-status surface and exposed capability, telemetry, resource accounting, profile recovery notices, generic server snapshots, and host-service status through the existing Tauri boundary.
- Added integration tests for clean terminal runtime-exit observation and the 256-entry deterministic runtime-event retention contract, on top of the retained source-branch tests for adapter behavior, runtime failure, HTTP lifecycle/ports, profile migration/recovery/validation, resource accounting, unknown telemetry, and host-service serialization.

### Reconciliation decisions

- `apps/slopity/src-tauri/src/lib.rs` now owns one `ServerOrchestrator`; there is no parallel Tauri-managed `HttpServerManager` competing for lifecycle state.
- `crates/slopity-core/src/lib.rs` combines the orchestrator, resource-accounting, profile-recovery/migration, runtime-observation, and hardened-validation exports while remaining pure Rust.
- The host plugin combines PR #6 telemetry with PR #9 Android lifecycle/status and notification-permission behavior rather than choosing one branch snapshot over the other.
- Resource reservations derive active IDs from orchestrator observations, including disabled profiles that remain active, so accounting does not rely only on persisted `enabled` flags.
- A native Android `stopRequestPending` is surfaced for safe reconciliation, but it does not directly kill Rust listeners. Automatic process-death recovery or durable background hosting is not claimed.
- PR #6's Rust 1.89 formatting corrections were applied while porting its accounting implementation instead of carrying its known exact-head formatting failure.

### Files changed

- `.github/workflows/ci.yml`
- `apps/slopity/src-tauri/src/lib.rs`
- `apps/slopity/web/index.html`
- `apps/slopity/web/resource-status.js`
- `crates/slopity-core/src/capability.rs`
- `crates/slopity-core/src/lib.rs`
- `crates/slopity-core/src/orchestrator.rs`
- `crates/slopity-core/src/profile_store.rs`
- `crates/slopity-core/src/runtime.rs`
- `crates/slopity-core/src/validation.rs`
- `crates/slopity-core/tests/orchestrator_foundation.rs`
- `crates/slopity-runtime-http/src/lib.rs`
- `plugins/tauri-plugin-slopity-host/Cargo.toml`
- `plugins/tauri-plugin-slopity-host/src/lib.rs`
- `plugins/tauri-plugin-slopity-host/src/mobile.rs`
- `plugins/tauri-plugin-slopity-host/src/telemetry.rs`
- `plugins/tauri-plugin-slopity-host/android/README.md`
- `plugins/tauri-plugin-slopity-host/android/build.gradle.kts`
- `plugins/tauri-plugin-slopity-host/android/src/main/AndroidManifest.xml`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostForegroundService.kt`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostPlugin.kt`
- `PROGRESS.md`

### Verification performed

- Live-state verification on 2026-08-18 confirmed `main` at `356dd3aa9588b885f74ee9d59800f77c74487f34`; draft PRs #5, #6, #9, and #10 were all still open, unmerged, and based on that same main.
- PR #5 retained orchestrator implementation had prior Rust-quality and Linux Tauri success on workflow run `31279333145`; its Android job never executed before cancellation, so that source run is not treated as Android proof.
- PR #6 exact-head workflow run `31279264370` failed only `cargo fmt --all -- --check`; tests, Clippy, Linux, and Android were skipped. The reported Rust 1.89 formatting changes were applied to the reconciled accounting code rather than weakening the gate.
- PR #9 exact-head Linux-and-Android workflow run `31280218290` succeeded, providing the strongest source-branch compile evidence for the Android lifecycle/status changes.
- PR #10 exact-head Linux-and-Android run `31280369242` was cancelled; earlier branch validation had reached its recovery/migration tests and formatting, but the integration does not treat that cancelled final head as sufficient proof.
- A disposable validation branch `integration/workload-foundation-validation` was created from the Step 018 plan commit. Candidate head `8a5baed33259404ff905fae4a4d096b149bd8fd3` is two commits ahead of source main and contains the reconciled implementation tree.
- Draft validation PR #11 was opened explicitly as `Do not merge`. Linux-and-Android CI run `32139520572` and package-validation run `32139520552` were created for that exact candidate head.
- At Step 018 closeout, CI run `32139520572` remained `queued`; its only materialized job, `Rust quality` (`95718614186`), also remained `queued` with zero executed steps. No formatting, test, Clippy, Linux, or Android success is claimed for the reconciled candidate.
- Local execution was not substituted for the required self-hosted workflow because the agent environment has no Rust toolchain; the requested existing self-hosted Linux GitHub Actions runner remains the authoritative validation environment.

### Verification pending / blocked

- The self-hosted Linux runner must actually execute `cargo fmt --all -- --check`, `cargo test --workspace --all-features`, and `cargo clippy --workspace --all-targets --all-features -- -D warnings` for the reconciled integration head.
- The same workflow must execute the repository Linux prerequisite check, dependency install, and Linux `npm run tauri:check` validation.
- The same workflow must execute Android SDK/NDK setup, `npm run android:init -- --ci`, `npm run android:build -- --debug --target aarch64`, merged-manifest foreground-service assertions, and Android artifact upload.
- A final exact-head pull-request CI run on `integration/workload-foundation` is required after the mandated implementation commit exists. That run cannot exist before this ledger is committed and therefore is not pre-claimed here.
- Physical Android proof remains required for notification visibility, foreground/background reachability, notification stop-request reconciliation, app/activity recreation, OEM behavior, clean listener shutdown, and port release. Compilation cannot satisfy this proof.

### Known limitations

- Desired/observed runtime state and runtime events remain process-local; persistence of live runtime state, restart recovery, crash-loop policy, and automatic runtime restoration are future work.
- Only built-in HTTP is registered and runnable. Java/JVM, Minecraft, GitHub deployment, Jellyfin, Node.js, Python, PHP, native packages, custom commands, and arbitrary external runtimes remain unavailable.
- Android foreground-service native state is in-process and `START_NOT_STICKY`; no boot receiver, process-death recovery, or durable-hosting claim is added.
- A notification `Request stop` records a pending native request, but the native layer deliberately does not stop Rust listeners by itself. A future resultful orchestration reconciliation path is needed before that action can safely become a full stop control.
- Until the queued integration workflow executes successfully, this branch is not CI-proven and must not be treated as merge-ready solely from the source-branch evidence.

### Correction note

- The Step 018 planning commit accidentally shortened the existing Step 005 follow-up sentence while replacing the full ledger file. This implementation commit restores the historical line verbatim to `Step 006 begins versioned profile persistence and CRUD. Windows remains deferred.` The final branch therefore does not alter completed Step 005 semantics.

### Follow-up

Keep the integration pull request draft and unmerged. When the existing self-hosted Linux runner accepts the queued work, require the full Rust, Linux Tauri, and Android ARM64 gates to pass without weakening checks. Then perform the remaining physical Android lifecycle proof before making any durability claim.

## Step 011: publish Linux and Android GitHub releases

**Status:** PARTIAL  
**Declared:** 2026-08-08  
**Updated:** 2026-08-08

### Scope

Add a dedicated release workflow that validates package creation on pull requests and publishes versioned GitHub prereleases from the existing Tauri application. Produce Linux `.deb` and AppImage bundles plus the currently proven Android ARM64 debug APK/AAB, collect SHA-256 checksums, and attach the files to a GitHub Release instead of leaving them only as short-lived workflow artifacts.

### Non-goals

- Claiming the Android package is production-signed or Play-ready. The existing proven Android build remains a debug build until signing is designed and configured.
- Marking the roadmap's reproducible signed release pipeline complete.
- Adding Windows or macOS packaging, changing runtime behavior, changing profile persistence, or modifying server security boundaries.
- Publishing releases from pull-request validation runs.
- Committing generated packages, signing keys, APKs, AABs, `.deb` files, AppImages, or Tauri generated mobile projects to Git.

### Delivered

- Added `.github/workflows/release.yml` with pull-request package validation plus `main`, `v*` tag, and manual-dispatch release paths.
- Added a metadata gate that requires `tauri.conf.json`, `package.json`, and the root Cargo workspace version to agree and requires release tags to match `v<version>`.
- Added release-mode Linux bundling with `tauri build --bundles deb,appimage`, deterministic discovery of exactly one `.deb` and one AppImage, and temporary GitHub Actions artifact upload.
- Added Android ARM64 debug APK/AAB packaging using the same SDK, NDK, Java, Tauri initialization, and target path already proven by normal CI.
- Added a publication job that is skipped on pull requests, receives `contents: write` only at job scope, downloads the validated package artifacts, gives them stable release names, generates `SHA256SUMS.txt`, and publishes them through `softprops/action-gh-release@v3` as a prerelease.
- Configured the existing 512 by 512 RGBA Slopity icon as a Tauri bundle icon so AppImage packaging can select a square icon.
- Added `docs/releases.md` documenting version synchronization, PR validation, automatic first release for a new version on `main`, explicit tag/manual paths, asset naming, and the unsigned Android limitation.
- Kept the existing normal CI workflow unchanged.

### Files changed

- `.github/workflows/release.yml`
- `apps/slopity/src-tauri/tauri.conf.json`
- `docs/releases.md`
- `PROGRESS.md`

### Verification performed

- Initial packaging run `31270262665` passed release metadata validation and compiled the Linux release binary. It successfully produced `target/release/bundle/deb/Slopity_0.1.0_amd64.deb`, then exposed a genuine AppImage bundler failure: `couldn't find a square icon to use as AppImage icon`.
- Added `bundle.icon: ["icons/icon.png"]` using the existing Step 004 512 by 512 RGBA application icon rather than suppressing the AppImage error.
- Replacement release-package run `31270803159` validated branch head `0fd05b182d526591bce7283a61c07c43922243cb` through pull-request merge ref `a7962b2bbfd005b9e61e2a1da62c6e88ed03dc76`.
- The replacement run passed release metadata validation, Linux release compilation, `.deb` bundling, AppImage bundling, deterministic Linux asset collection, and Linux artifact upload.
- Linux package artifact `9025722918`, named `release-linux-a7962b2bbfd005b9e61e2a1da62c6e88ed03dc76`, was created with size `79477996` bytes, digest `sha256:41b8b8ebfe7197877fc4f17847e31c5dd284f7f3a4e11f3ea78b050eb0be0067`, and seven-day retention.
- The same replacement run passed Android SDK/NDK setup, `npm run android:init -- --ci`, `npm run android:build -- --debug --target aarch64`, exact APK/AAB path checks, Android asset collection, and artifact upload.
- Android release-package artifact `9025859548`, named `release-android-a7962b2bbfd005b9e61e2a1da62c6e88ed03dc76`, was created with size `148565382` bytes, digest `sha256:5bf2a83f331cf7fd10894f3d2097eb8f79bf6784cc13fdad27c0f40b1235b227`, and seven-day retention.
- The GitHub prerelease publication job was skipped on the pull request exactly as designed, so review-time validation cannot create or overwrite a release.
- Normal CI run `31270803166` on the same branch head passed the progress-ledger guard, `cargo fmt --all -- --check`, all 14 workspace tests, and `cargo clippy --workspace --all-targets --all-features -- -D warnings`.
- The same normal CI run passed Linux prerequisite validation, `npm install --no-audit --no-fund`, Linux `npm run tauri:check`, Android SDK/NDK setup, Android initialization, ARM64 debug build, and Android artifact upload.
- Normal-CI Android artifact `9025966939`, named `slopity-android-debug-a7962b2bbfd005b9e61e2a1da62c6e88ed03dc76`, was created with size `148565466` bytes and digest `sha256:c1775f7163c48aea023632faf6d6da379eb16a35e22e9870b71e84e134b8bb7f`.

### Verification pending

- Actual `contents: write` GitHub Release creation and attachment of the normalized `.deb`, AppImage, APK, AAB, and `SHA256SUMS.txt` assets. This is intentionally not exercised from pull requests and requires the validated workflow to reach `main` or a matching release tag/manual dispatch.
- Installation and launch testing of the generated `.deb` and AppImage on a clean Linux system.
- Installation of the generated APK on a physical ARM64 Android device.
- Production Android signing, signed release-mode Android packaging, and signing-key lifecycle procedures.

### Known limitations

- GitHub Releases produced by this workflow are intentionally prereleases because Android is still a debug package and not production-signed.
- The Phase 6 reproducible signed release-pipeline roadmap item remains incomplete.
- Linux packages are currently x86_64 only and Android packages are currently ARM64 debug only; Windows, macOS, Linux ARM64, and additional Android release targets are not included.
- A GitHub Release is not visible yet because pull-request publication is intentionally disabled and pull request `#3` remains draft and unmerged pending explicit owner authorization.
- Physical-device and clean-install package behavior is not proven by compilation alone.

### Follow-up

After explicit owner authorization, merge pull request `#3`. The first `main` commit carrying version `0.1.0` while tag `v0.1.0` does not exist should build the packages again and publish the preview release. Verify that the release contains the `.deb`, AppImage, APK, AAB, and `SHA256SUMS.txt`, then continue toward signed Android release builds and physical package smoke tests.

### Correction note

- Step 010 recorded pull request `#2` as open and unmerged because that was true when its validation evidence was written. The owner later explicitly requested the merge on 2026-08-08, and PR `#2` was merged into `main` as merge commit `d093e6b5ea9554e22a33f0ee4517347c7c231386`.

## Step 010: redesign the mobile server dashboard

**Status:** PARTIAL  
**Declared:** 2026-08-08  
**Updated:** 2026-08-08

### Scope

Redesign the shared static HTML, CSS, and JavaScript interface around the supplied mobile mockup. Deliver a mobile-first saved-server dashboard, responsive server-card grid, functional hamburger drawer, add-server bottom sheet, compact per-server action menu, focused details surface, and profile editor while preserving the existing Tauri commands, Rust profile persistence, and built-in HTTP lifecycle behavior.

### Non-goals

- Rewriting the Rust core, persistence model, Tauri command surface, Android foreground service, or built-in HTTP runtime unless a small compatibility adjustment proves unavoidable.
- Claiming Minecraft, Node.js, imported VPS, static-site, or arbitrary custom runtimes are operational.
- Adding a frontend framework, icon dependency, shell command execution, downloads, uploads, remote management, TLS, authentication, or runtime-provider installation.
- Merging the branch, changing `main`, or modifying CI workflow behavior.

### Correction note

- Step 010 was resumed on 2026-08-08 from an older draft branch and pull request whose ledger text described a superseded multi-module frontend and earlier CI evidence.
- The superseded frontend modules were removed from the current branch diff. This entry now records the self-contained `index.html`, `app.js`, and `styles.css` implementation and the validation performed against the current code head.
- Steps 009 and earlier remain unchanged.

### Delivered

- Replaced the desktop-oriented overview with a mobile-first dashboard matching the supplied composition: oversized Slopity wordmark, hamburger control, CSS server-stack illustration, Saved Servers heading, dense two-column phone grid, and an easy-to-reach Add Server action.
- Added bundled CSS and inline-SVG illustrations, truthful status pills, three-dot card menus, touch-sized controls, safe-area spacing, narrow-phone fallback, tablet and desktop expansion, and reduced-motion handling.
- Added a functional navigation drawer with Servers, Add Server, Runtime Support, Device Status, Settings, and About Slopity. Settings is visibly marked planned.
- Added an Add Server bottom sheet with drag handle, close control, scrollable templates, focus trapping, backdrop dismissal, Escape dismissal, and body-scroll locking.
- Added truthful template behavior: Built-in HTTP creates a usable built-in profile; Website maps only to the built-in HTTP probe and says so; Minecraft and Node.js create disabled placeholders; Import performs no fake VPS import; Custom Template opens the blank profile editor.
- Added compact per-card actions and a details sheet exposing profile configuration, observed runtime state, bind address, available URLs, request count, recent logs, validation results, and lifecycle or CRUD controls.
- Added a bottom-sheet profile editor that reuses the existing create and update commands, keeps external arguments structured, runs native profile validation, and keeps unsupported external profiles disabled by default.
- Preserved persisted profile loading, create, edit, clone, enable, disable, delete, refresh, built-in HTTP start and stop, runtime polling, and backend-enforced running-profile restrictions.
- Rendered profile names, URLs, log messages, validation messages, runtime errors, and other user-controlled values through DOM text nodes instead of unsafe HTML interpolation.
- Kept the Rust backend, profile model, Tauri invocation names, Android host-service bridge, and runtime security boundaries unchanged.
- Fixed decorative horizontal overflow and verified the document width at 320, 430, 800, and 1280 pixel viewports.

### Files changed

- `apps/slopity/web/index.html`
- `apps/slopity/web/app.js`
- `apps/slopity/web/styles.css`
- `PROGRESS.md`

### Verification performed

- Local JavaScript syntax validation with `node --check` passed on the implementation draft before it was pushed. The repository has no separate frontend validation script.
- A source scan of the current branch `app.js` found no `innerHTML`, `insertAdjacentHTML`, or `eval(` usage.
- Local mocked-browser interaction checks exercised the drawer, Add Server sheet, details sheet, action sheet, backdrop and Escape dismissal, and the responsive server-card grid.
- The responsive browser matrix passed without horizontal document overflow at 320, 430, 800, and 1280 pixels after the overflow fix.
- Pull-request workflow run `31266454190` validated code head `1e0e145a7a933e7102d69ccb721194bc31a70d47` through merge ref `b118419fd9f545f1dd8659d3cb33c5fdbe43c5b4` against main `c5a3e29b75c309c96502c9122bd44522aa7ed4be`.
- `cargo fmt --all -- --check` passed.
- `cargo test --workspace --all-features` passed all 14 workspace tests with 0 failures.
- `cargo clippy --workspace --all-targets --all-features -- -D warnings` passed.
- The Linux job passed the prerequisite check, `npm install --no-audit --no-fund`, and `npm run tauri:check`.
- The Android job installed the configured SDK and NDK, passed `npm run android:init -- --ci`, and passed `npm run android:build -- --debug --target aarch64`.
- Android produced `app-universal-debug.apk` and `app-universal-debug.aab` and uploaded artifact `9024516514`, named `slopity-android-debug-b118419fd9f545f1dd8659d3cb33c5fdbe43c5b4`, size `148565786` bytes, digest `sha256:b3b97cfec46bfc35e6b4d3f4075f51a8c5cc611cb8276f2a5f55f2683bd5627f`, retained through 2026-08-22.
- Draft pull request `#2` remains open, unmerged, and based on `main`.

### Verification pending

- Interactive native Linux WebView smoke testing of profile create, edit, clone, enable, disable, delete, built-in HTTP start, request generation, URL display, log display, stop, application restart, and UI-observed persistence.
- Installation of the generated APK on an ARM64 Android device and validation of touch behavior, system safe areas, keyboard behavior, notification visibility, foreground reachability, background reachability, graceful stop, and port release.
- Native Linux or physical-Android screenshot capture. Current visual screenshots are mocked browser previews of the shared frontend.

### Known limitations

- Website currently creates the fixed built-in HTTP probe foundation; it does not serve a user-selected static folder or deploy a web application.
- Minecraft, Node.js, import, custom, Java, Python, PHP, and native providers remain unavailable or configuration-only and expose no false start path.
- Settings is a planned drawer entry without a settings screen.
- Runtime state remains process-local and resets to stopped after application process exits, as documented by Step 008.
- Android compilation does not prove OEM background behavior, app-restart behavior, or Google Play foreground-service policy acceptance.
- The Android build still reports existing non-blocking generated-code, Gradle deprecation, and duplicate `com.slopity.host` namespace warnings.
- The dashboard uses bundled CSS and generic SVG illustrations rather than licensed game artwork from the mockup.

### Follow-up

Run the native Linux interaction and restart-persistence smoke test, then install the uploaded ARM64 debug APK on a physical Android device for the remaining touch, notification, reachability, and background-hosting proof.

## Step 009: publish installable Android debug artifacts

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

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

### Verification performed

- Workflow run `31028085306` passed Rust formatting, all 14 workspace tests, strict Clippy, Linux Tauri compilation, Android initialization, and the ARM64 debug APK build.
- The `Upload Android debug packages` step completed successfully.
- GitHub created artifact `8939796324`, named `slopity-android-debug-1e864bd8e7b8cd2294f8ef4f64d6e9bce7cc07e3`, with 14-day retention through 2026-08-19.
- GitHub reported artifact size `148602982` bytes and digest `sha256:649ee5d05962102907c3fce8468d64805ee52a7d13a32c8e334f6d9271a6063b`.
- The downloaded ZIP matched that digest and contained exactly `app-universal-debug.apk` and `app-universal-debug.aab`.
- The APK archive passed an integrity test and contains `lib/arm64-v8a/libslopity_lib.so`, `AndroidManifest.xml`, and Android DEX files.

### Follow-up

Install the debug APK on an ARM64 Android device and perform the Step 008 notification, reachability, background-survival, stop, and port-release smoke test. The AAB remains for packaging inspection and is not directly installable like the APK.

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
- The same run passed all 14 workspace tests, including the built-in HTTP response, graceful stop and port release, duplicate-start rejection, occupied ports, and bounded-log tests.
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

- Replacement workflow run `31012340919` reached `cargo fmt --all -- --check` immediately after Rust toolchain setup.
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