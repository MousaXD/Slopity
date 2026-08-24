# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 019: public release hardening and source-available transition

**Status:** IN PROGRESS  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

Prepare Slopity for a high-quality public source-available release:
1. Licensing: Adopt the standard, official, unmodified PolyForm Noncommercial License 1.0.0 (`PolyForm-Noncommercial-1.0.0`), update Cargo package license metadata, add `LICENSE.md`, add `COMMERCIAL-LICENSE.md`, update all documentation to describe Slopity as source-available rather than open-source, and document external contribution policy.
2. Dependency Reproducibility: Commit root `Cargo.lock` and `apps/slopity/package-lock.json`, update CI/build workflows and scripts to use `--locked` and `npm ci`.
3. GitHub Actions Hardening: Migrate CI and release workflows from self-hosted runners to standard GitHub-hosted `ubuntu-latest` runners, pin third-party actions to immutable commit SHAs with version comments, enforce strict top-level read-only permissions with minimal write scopes.
4. Dependency Security: Add `.github/dependabot.yml` covering cargo, npm, and github-actions, add a security/audit workflow with `cargo-audit`, `actions/dependency-review-action`, and `npm audit`.
5. Secret and History Hygiene: Audit Git history and codebase for secrets and credentials.
6. Profile-ID Hardening: Implement strict identifier validation grammar `[A-Za-z0-9._-]{1,128}` in `slopity-core` to prevent control-character injection, interior NUL panics in thread naming, directory traversal, and malformed identifiers. Add regression tests covering edge cases.
7. Security Policy: Expand and harden `SECURITY.md` with supported versions, pre-release boundaries, responsible disclosure instructions, and testing rules.
8. Contributing Guidelines: Update `CONTRIBUTING.md` with clear licensing and external contribution terms, testing commands, and PR guidelines.
9. Documentation & Hygiene: Refresh `README.md`, `TASK.md`, and `docs/releases.md` to accurately represent current functionality, pre-production status, and source-available licensing.

### Non-goals

- Claiming external runtimes (Java/Minecraft, Node.js, Python, PHP, etc.) are implemented or operational.
- Weakening existing security boundaries or introducing shell execution.
- Modifying production signing infrastructure before keys/procedures are formally established.
- Merging PR #12 or any other open PR branches.
- Committing signing keys, credentials, APKs, or binaries.

### Risks and mitigation

- Risk: CI workflow failures when switching from self-hosted to GitHub-hosted runners due to missing system libraries.
  Mitigation: Explicitly install WebKitGTK and Tauri Linux build prerequisites in GitHub Actions workflow steps before running checks.
- Risk: Cargo rejecting non-standard SPDX license identifier in `Cargo.toml`.
  Mitigation: Verify Cargo 1.89+ acceptance of `PolyForm-Noncommercial-1.0.0` or supplement with `license-file`.
- Risk: Breaking valid existing profile identifiers during ID hardening.
  Mitigation: Ensure standard sample IDs (`http-example`, etc.) strictly comply with `[A-Za-z0-9._-]{1,128}` and test boundary conditions thoroughly.

### Intended files

- `PROGRESS.md`
- `LICENSE.md`
- `COMMERCIAL-LICENSE.md`
- `Cargo.toml`
- `Cargo.lock`
- `apps/slopity/package.json`
- `apps/slopity/package-lock.json`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/audit.yml`
- `.github/dependabot.yml`
- `.gitignore`
- `crates/slopity-core/src/validation.rs`
- `crates/slopity-core/src/lib.rs`
- `README.md`
- `SECURITY.md`
- `CONTRIBUTING.md`
- `TASK.md`
- `docs/releases.md`

### Acceptance checks

1. `cargo fmt --all -- --check` passes cleanly.
2. `cargo test --workspace --all-features --locked` passes all existing and new unit/integration tests.
3. `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` reports 0 warnings.
4. `cd apps/slopity && npm ci && npm run tauri:check` passes without error.
5. Profile ID validation rejects empty, oversized (>128), NUL-byte, control-character, space, and path-traversal IDs while accepting valid alphanumeric, dot, underscore, and hyphen IDs.
6. All stale "open-source", "MIT", "Apache" references in repository docs are replaced with source-available terms.
7. Workflows are migrated to `ubuntu-latest`, third-party actions pinned by SHA, and dependabot configuration added.

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
- Runtime state remains process-local and resets to stopped after the application process exits, as documented by Step 008.
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