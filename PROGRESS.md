# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain corrections in a new entry.

## Step 022: repair and harden CI and release foundation

**Status:** IN PROGRESS  
**Declared:** 2026-08-25  
**Updated:** 2026-08-25

### Reconciliation note

- Concurrent GUI work on `overhaul/gui-v2` has already declared Step 021 for the shared control-center GUI. The separate `ci/release-pipeline-v2` branch is only one planning commit ahead of `dev` and labels a release/Minecraft plan as Step 021 without adding it to the live `dev` progress ledger. This Step 022 is the canonical CI/release-infrastructure slice from that abandoned/conflicting plan; it deliberately excludes its Minecraft/runtime scope.

### Scope

1. Repair GitHub-hosted CI so Rust formatting, metadata, tests, Clippy, Linux Tauri compilation, Android ARM64 packaging/manifest verification, and Windows x64 Tauri/NSIS validation can be used as remote build gates by independent feature branches.
2. Make ordinary branch pushes testable without allowing one branch's concurrency group to cancel unrelated branch validation; keep failures strict and actionable and order expensive platform builds behind cheaper Rust validation.
3. Repair and harden release-package validation and publication behavior, preserving PR no-publish guarantees, synchronized version checks, deterministic assets/checksums, clearly labeled Android debug packages, and minimal publication permissions.
4. Refresh SHA-pinned third-party Actions to compatible supported runtimes where live Dependabot evidence identifies maintained replacements, without weakening immutable pinning.
5. Harden dependency/audit automation and Dependabot targeting for the active `dev` workflow, and correct stale CI-host prerequisite documentation.

### Non-goals

- Merging this branch or any pull request.
- GUI V2 implementation.
- Minecraft/Java/JVM, Jellyfin, Node.js, Python, PHP, generic native runtimes, arbitrary process execution, downloads, or signing-secret invention.
- Claiming Android production signing or physical-device durability.
- Claiming Windows support unless a GitHub-hosted Windows job actually compiles and packages it.

### Risks

- Tauri bundle output paths differ by platform and must be asserted rather than silently globbed.
- Newer JavaScript Actions require a GitHub runner runtime that supports Node 24; hosted-runner compatibility must be proven by Actions rather than assumed.
- Release reruns or reused tags can mutate an existing release unless duplicate policy is explicit and enforced.
- CI concurrency that keys only on workflow/ref can behave differently for push and pull-request refs; branch isolation must remain explicit.
- Android debug packages must stay unmistakably labeled as debug and must never imply production signing.

### Intended files

- `PROGRESS.md`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/workflows/audit.yml`
- `.github/dependabot.yml`
- `docs/releases.md`
- `docs/ci-host-prerequisites.md`
- `README.md`
- `CURRENT_STATUS.md`
- `TASK.md`

### Acceptance checks

- `cargo fmt --all -- --check`
- `cargo metadata --locked --format-version 1`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cd apps/slopity && npm ci && npm audit && npm run tauri:check`
- GitHub Actions Rust quality job green on this branch/PR.
- GitHub Actions Linux Tauri job green.
- GitHub Actions Android ARM64 debug build, merged-manifest verification, and artifact upload green.
- GitHub Actions Windows x64 Tauri compile/NSIS package validation green, or exact platform blocker documented without claiming support.
- Release pull-request package validation green for Linux `.deb`/AppImage and Android ARM64 debug APK/AAB, with publication skipped by construction.
- Security/audit jobs green where applicable, with dependency review active for this public repository.
- Every third-party `uses:` reference remains pinned to a full immutable commit SHA.

## Step 020: public history sanitation, workload reconciliation, and release candidate validation

**Status:** DONE  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

1. Workload Foundation & Hardening Integration: Reconcile PR #12 (`integration/workload-foundation`) and PR #13 (`chore/public-release-hardening`) into a unified release candidate branch, integrating authoritative `ServerOrchestrator`, resource accounting, host telemetry, profile store recovery/migrations, strict identifier validation grammar, locked dependencies, PolyForm Noncommercial License 1.0.0, and updated public presentation.
2. Git History & Email Sanitation: Perform deep history audit and rewrite all reachable commits using `git-filter-repo` to replace the maintainer's previously exposed private email with the GitHub noreply address (`101021011+MousaXD@users.noreply.github.com`), while preserving author names, commit messages, and full merge topology.
3. Path & Secret Hygiene: Eliminate all developer-local machine paths across all commit diffs and trees. Perform automated history scanning via `gitleaks` and multi-pattern searches.
4. Offline Local Validation: Thoroughly validate the unified release tree locally without relying on GitHub Actions credits (Rust format, unit/integration tests, Clippy, workspace metadata, Cargo audit, npm audit, Tauri check, Android ARM64 library compilation, Linux `.deb` bundle packaging).
5. Publication Staging: Prepare clean candidate branch `release/public-clean-history` and map old/new SHA lineage for repository owner promotion to `main`. Keep repository private until explicit owner action.

### Delivered

1. Integrated release candidate on branch `release/public-clean-history`, combining all capabilities from PR #12 and PR #13 without loss of any architectural or security features.
2. Complete Git history rewrite using `git-filter-repo` across all reachable refs: 0 private maintainer email occurrences remain, 100% of maintainer commits mapped to `MousaXD <101021011+MousaXD@users.noreply.github.com>`.
3. Sanitized all developer-local paths across all commit trees and history diffs.
4. Gitleaks scan confirmed 0 leaks across repository history.
5. All 60 Rust unit and integration tests passed (`slopity-core`, `slopity-runtime-http`, `tauri-plugin-slopity-host`).
6. Clippy passed with 0 warnings with `-D warnings` on `--all-targets --all-features --locked`.
7. Workspace license metadata verified across all 5 workspace crates (`PolyForm-Noncommercial-1.0.0`).
8. Cargo audit and npm audit verified 0 vulnerabilities.
9. Tauri application debug build completed and verified on Linux.
10. Android ARM64 native shared library `libslopity_lib.so` compiled cleanly against Android SDK Platform 36 and NDK `27.3.13750724`. Merged AndroidManifest verified with `POST_NOTIFICATIONS`, `FOREGROUND_SERVICE_SPECIAL_USE`, `com.slopity.host.HostForegroundService`, and `foregroundServiceType="specialUse"`.
11. Linux `.deb` packaging verified with `Slopity_0.1.0_amd64.deb` bundle generation.
12. Created `CURRENT_STATUS.md` and verified all relative Markdown links across documentation.

### Files changed

- `PROGRESS.md`
- `CURRENT_STATUS.md`
- `README.md`
- `.gitignore`
- `Cargo.lock`
- `apps/slopity/src-tauri/src/lib.rs`
- `crates/slopity-core/src/lib.rs`
- `crates/slopity-core/src/validation.rs`

### Verification performed

```bash
# Toolchains
rustc 1.89.0 / cargo 1.89.0 / node v22.23.2 / npm 11.18.0

# Rust Formatting
cargo fmt --all -- --check
# Result: PASS (0 diffs)

# Test Suite
cargo test --workspace --all-features --locked
# Result: PASS (60 passed, 0 failed, 0 ignored)
#   - slopity-core: 44 unit + 2 integration tests
#   - slopity-runtime-http: 8 tests
#   - tauri-plugin-slopity-host: 6 tests

# Strict Linting
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# Result: PASS (0 warnings)

# License Metadata
cargo metadata --locked --format-version 1
# Result: PASS (All 5 crates inherit PolyForm-Noncommercial-1.0.0)

# Dependency Security
cargo audit
# Result: PASS (0 vulnerabilities across 434 dependencies)

# Frontend & Tauri Checks
cd apps/slopity && npm ci && npm audit && npm run tauri:check
# Result: PASS (0 vulnerabilities, binary compiled at target/debug/slopity)

# Linux Bundle Validation
npx tauri build --bundles deb --debug
# Result: PASS (target/debug/bundle/deb/Slopity_0.1.0_amd64.deb generated)

# Android ARM64 Compilation
npm run android:init -- --ci
npm run android:build -- --debug --target aarch64
# Result: PASS (libslopity_lib.so compiled for aarch64-linux-android with NDK 27.3.13750724)

# Secret & Privacy Scans
gitleaks detect --source . --verbose --log-opts="release/public-clean-history"
# Result: PASS (0 leaks found across 56 commits)

git log release/public-clean-history --format='%H%x09%an%x09%ae%x09%cn%x09%ce' | sort -u -k2
# Result: PASS (0 private emails, only 101021011+MousaXD@users.noreply.github.com)
```

### Verification pending / blocked

- Physical on-device Android runtime durability proof (background reachability, notification stop reconciliation, thermal/battery tracking over a multi-hour real-device session).
- Windows compilation, packaging, and CI validation remain deferred.

### Known limitations

- Repository remains private until the owner reviews the audit report, enables GitHub account email privacy, and executes public promotion.

## Step 019: amendment and public release hardening correction

**Status:** DONE  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

Factual corrections and hardening fixes for Step 019 and PR #13:
1. Reconcile PR #12 relationship: Affirm that draft PR #12 (`Step 018: reconcile workload foundation`) is essential substantive architecture (orchestrator, runtime observations, resource accounting, host telemetry, interrupted-write recovery, schema migrations, and Android host lifecycle) and MUST NOT be closed. Establish a clear post-merge integration sequence.
2. Link and Path Hygiene: Eliminate all developer-local machine paths across `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and `COMMERCIAL-LICENSE.md`, replacing them with repository-relative Markdown links.
3. Feature Claim Accuracy: Re-align `README.md` to truthfully describe functionality on current `main` / PR #13 versus capabilities implemented in pending PR #12, and clarify local verification versus GitHub Actions billing gates.
4. Cargo License Metadata: Retain `license = "PolyForm-Noncommercial-1.0.0"` in `Cargo.toml` and eliminate redundant `license-file`, keeping `LICENSE.md` in repository root. Verified inheritance across all 5 workspace crates via `cargo metadata`.
5. Contribution Policy Clarification: Update `CONTRIBUTING.md` with conservative external contribution guidelines that do not assume or imply copyright assignment.
6. GitHub Actions Audit: Investigate exact runner failure annotations on PR #13 exact head (jobs blocked at account admission due to private repository Actions minute/spending limits), configure `audit.yml` with private/public repository awareness for `actions/dependency-review-action`.

### Delivered

1. Corrected all public documentation links to repository-relative format (`[LICENSE.md](LICENSE.md)`, `[COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)`, `[CONTRIBUTING.md](CONTRIBUTING.md)`, `[AGENTS.md](AGENTS.md)`, `[SECURITY.md](SECURITY.md)`, `[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)`, `[docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)`, `[docs/releases.md](docs/releases.md)`). Repository-wide scan confirms 0 developer-local paths remaining.
2. Overhauled `README.md` with strict distinction between operational capabilities on current `main` / PR #13 versus capabilities implemented in pending PR #12, and clarify local verification versus GitHub Actions billing gates.
3. Updated `Cargo.toml` workspace metadata to use standard `license = "PolyForm-Noncommercial-1.0.0"`. `cargo metadata --locked` confirms all crates inherit `"license": "PolyForm-Noncommercial-1.0.0"`.
4. Refined `CONTRIBUTING.md` with conservative contribution licensing terms without inventing CLAs or assuming copyright transfers.
5. Updated `.github/workflows/audit.yml` to gracefully handle private evaluation vs public execution for `actions/dependency-review-action`.
6. Formulated explicit post-Step-019 integration strategy for PR #12: keep PR #12 open/draft, merge PR #13 into `main` after owner approval, merge resulting `main` into `integration/workload-foundation`, resolve conflicts, and run full CI against PR #12.

### Files changed

- `PROGRESS.md`
- `README.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `COMMERCIAL-LICENSE.md`
- `Cargo.toml`
- `.github/workflows/audit.yml`

### Verification performed

```bash
cargo fmt --all -- --check
# Pass (0 diffs)

cargo test --workspace --all-features --locked
# Pass (20 tests passed: 16 core, 4 http runtime, 0 failed, 0 ignored)

cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# Pass (0 warnings)

cargo metadata --locked --format-version 1
# Pass (all crates cleanly resolve PolyForm-Noncommercial-1.0.0)

cargo audit
# Pass (0 vulnerabilities across 434 dependencies)

cd apps/slopity && npm ci && npm audit && npm run tauri:check
# Pass (0 vulnerabilities, debug binary built cleanly)

grep -RInE 'file://|/home/|/Users/|C:\\Users\\' --exclude-dir={.git,target,node_modules,gen} .
# Pass (0 matches found)
```

## Step 019: public release hardening and source-available transition

**Status:** DONE  
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

### Delivered

1. Official unmodified PolyForm Noncommercial License 1.0.0 in `LICENSE.md` (`PolyForm-Noncommercial-1.0.0`) and dual-licensing commercial terms in `COMMERCIAL-LICENSE.md`.
2. Cargo workspace package metadata (`license = "PolyForm-Noncommercial-1.0.0"`, `license-file = "LICENSE.md"`) and frontend `package.json` updated with matching license identifier.
3. Locked dependency manifests committed (`Cargo.lock` with 434 crates, `apps/slopity/package-lock.json`).
4. CI (`.github/workflows/ci.yml`) and Release (`.github/workflows/release.yml`) migrated from self-hosted runners to GitHub-hosted `ubuntu-latest` runners with automated Tauri Linux prerequisite installation and locked builds (`--locked`, `npm ci`).
5. All third-party GitHub Actions pinned to immutable full commit SHAs with version annotations across all workflows.
6. Dependabot configuration in `.github/dependabot.yml` monitoring Cargo, npm, and GitHub Actions dependencies weekly.
7. Security audit workflow in `.github/workflows/audit.yml` running `cargo-audit`, `dependency-review-action`, and `npm audit`.
8. Git history and commit trees audited for credentials/secrets across all 78 commits with 0 secret leaks found.
9. Server profile identifier validation hardened in `crates/slopity-core/src/validation.rs`: strict `[A-Za-z0-9._-]{1,128}` validation via `is_valid_profile_id` and `MAX_PROFILE_ID_LENGTH`, preventing interior NUL bytes (`\0`) which trigger thread spawn panics in `slopity-runtime-http`, control character escape injections, and path traversal. Re-exported in `crates/slopity-core/src/lib.rs`.
10. Added 6 comprehensive validation tests in `validation.rs` covering valid ID forms, empty/whitespace IDs, oversized IDs (129 chars), NUL byte rejection, control characters (`\n`, `\r`, `\t`, ANSI escapes), and path traversal (`../`, `/`, `\`).
11. Public security policy overhauled in `SECURITY.md` with supported versions table, pre-release disclaimer, responsible disclosure via GitHub Private Vulnerability Reporting and maintainer contact, report requirements, and ethical testing boundaries.
12. Contributing guidelines updated in `CONTRIBUTING.md` with source-available inbound contribution terms, mandatory two-commit protocol, and reproducible setup instructions.
13. Public presentation updated in `README.md`, `TASK.md`, and `docs/releases.md` accurately detailing operational status (built-in HTTP server and child-process boundary operational; Java/Minecraft/Node.js/Python unbundled/unavailable), source-available license, and hosted release pipeline.

### Files changed

- `PROGRESS.md`
- `LICENSE.md`
- `COMMERCIAL-LICENSE.md`
- `Cargo.toml`
- `Cargo.lock`
- `apps/slopity/package.json`
- `apps/slopity/package-lock.json`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/dependabot.yml`
- `.github/workflows/audit.yml`
- `crates/slopity-core/src/validation.rs`
- `crates/slopity-core/src/lib.rs`
- `SECURITY.md`
- `CONTRIBUTING.md`
- `README.md`
- `TASK.md`
- `docs/releases.md`

### Verification performed

```bash
cargo fmt --all -- --check
# Pass (0 diffs)

cargo test --workspace --all-features --locked
# Pass (20 tests passed: 16 core, 4 http runtime, 0 failed, 0 ignored)

cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# Pass (0 warnings)

cargo metadata --locked --format-version 1
# Pass (all crates cleanly resolve PolyForm-Noncommercial-1.0.0)

cargo audit
# Pass (0 vulnerabilities across 434 dependencies)

cd apps/slopity && npm ci && npm audit && npm run tauri:check
# Pass (0 vulnerabilities, debug binary built cleanly)

grep -RInE 'file://|/home/|/Users/|C:\\Users\\' --exclude-dir={.git,target,node_modules,gen} .
# Pass (0 matches found)
```

## Step 018: reconcile workload foundation

**Status:** DONE  
**Declared:** 2026-08-18  
**Updated:** 2026-08-24

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
- Added proven host telemetry surfaces: Linux `/proc/meminfo` memory telemetry and Android system-service memory, battery, temperature, thermal state, storage headroom. Missing values remain `None` instead of being guessed.
- Integrated profile-store interrupted-write recovery, primary/temporary/backup candidate inspection, backup recovery, parent-directory synchronization, schema-v0 to schema-v1 migration infrastructure, migration/recovery notices, and hard failure for unsupported future schemas.
- Integrated backend profile validation limits for IDs, names, argument counts and payload sizes, executable paths, working directories, duplicate IDs, and duplicate ports while keeping structural profile validity separate from runtime availability.
- Integrated Android foreground-host lifecycle/status improvements: POST_NOTIFICATIONS permission handling, native service status, active-server counts, start/update behavior, conservative notification visibility, a pending notification stop request, and `START_NOT_STICKY` process-lifecycle honesty.
- Added a dashboard resource-status surface and exposed capability, telemetry, resource accounting, profile recovery notices, generic server snapshots, and host-service status through the existing Tauri boundary.
- Added integration tests for clean terminal runtime-exit observation and the 256-entry deterministic runtime-event retention contract, on top of the retained source-branch tests for adapter behavior, runtime failure, HTTP lifecycle/ports, profile migration/recovery/validation, resource accounting, unknown telemetry, and host-service serialization.

## Step 019: amendment and public release hardening correction

**Status:** DONE  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

Factual corrections and hardening fixes for Step 019 and PR #13:
1. Reconcile PR #12 relationship: Affirm that draft PR #12 (`Step 018: reconcile workload foundation`) is essential substantive architecture (orchestrator, runtime observations, resource accounting, host telemetry, interrupted-write recovery, schema migrations, and Android host lifecycle) and MUST NOT be closed. Establish a clear post-merge integration sequence.
2. Link and Path Hygiene: Eliminate all developer-local machine paths (`file:///home/developer/...`) across `README.md`, `CONTRIBUTING.md`, `SECURITY.md`, and `COMMERCIAL-LICENSE.md`, replacing them with repository-relative Markdown links.
3. Feature Claim Accuracy: Re-align `README.md` to truthfully describe functionality on current `main` / PR #13 versus capabilities implemented in pending PR #12, and clarify local verification versus GitHub Actions billing gates.
4. Cargo License Metadata: Retain `license = "PolyForm-Noncommercial-1.0.0"` in `Cargo.toml` and eliminate redundant `license-file`, keeping `LICENSE.md` in repository root. Verified inheritance across all 5 workspace crates via `cargo metadata`.
5. Contribution Policy Clarification: Update `CONTRIBUTING.md` with conservative external contribution guidelines that do not assume or imply copyright assignment.
6. GitHub Actions Audit: Investigate exact runner failure annotations on PR #13 exact head (jobs blocked at account admission due to private repository Actions minute/spending limits), configure `audit.yml` with private/public repository awareness for `actions/dependency-review-action`.

### Delivered

1. Corrected all public documentation links to repository-relative format (`[LICENSE.md](LICENSE.md)`, `[COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)`, `[CONTRIBUTING.md](CONTRIBUTING.md)`, `[AGENTS.md](AGENTS.md)`, `[SECURITY.md](SECURITY.md)`, `[docs/ARCHITECTURE.md](docs/ARCHITECTURE.md)`, `[docs/SECURITY_MODEL.md](docs/SECURITY_MODEL.md)`, `[docs/releases.md](docs/releases.md)`). Repository-wide scan confirms 0 developer-local paths remaining.
2. Overhauled `README.md` with strict distinction between operational capabilities on `main`/PR #13, foundation capabilities in pending draft PR #12, and unbundled runtimes (Java/Minecraft/Node.js/Python).
3. Updated `Cargo.toml` workspace metadata to use standard `license = "PolyForm-Noncommercial-1.0.0"`. `cargo metadata --locked` confirms all crates inherit `"license": "PolyForm-Noncommercial-1.0.0"`.
4. Refined `CONTRIBUTING.md` with conservative contribution licensing terms without inventing CLAs or assuming copyright transfers.
5. Updated `.github/workflows/audit.yml` to gracefully handle private evaluation vs public execution for `actions/dependency-review-action`.
6. Formulated explicit post-Step-019 integration strategy for PR #12: keep PR #12 open/draft, merge PR #13 into `main` after owner approval, merge resulting `main` into `integration/workload-foundation`, resolve conflicts, and run full CI against PR #12.

### Files changed

- `PROGRESS.md`
- `README.md`
- `CONTRIBUTING.md`
- `SECURITY.md`
- `COMMERCIAL-LICENSE.md`
- `Cargo.toml`
- `.github/workflows/audit.yml`

### Verification performed

```bash
cargo fmt --all -- --check
# Pass (0 diffs)

cargo test --workspace --all-features --locked
# Pass (20 tests passed: 16 core, 4 http runtime, 0 failed, 0 ignored)

cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
# Pass (0 warnings)

cargo metadata --locked --format-version 1
# Pass (all crates cleanly resolve PolyForm-Noncommercial-1.0.0)

cargo audit
# Pass (0 vulnerabilities across 434 dependencies)

cd apps/slopity && npm ci && npm audit && npm run tauri:check
# Pass (0 vulnerabilities, debug binary built cleanly)

grep -RInE 'file://|/home/|/Users/|C:\\Users\\' --exclude-dir={.git,target,node_modules,gen} .
# Pass (0 matches found)
```

## Step 019: public release hardening and source-available transition

**Status:** DONE  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

Prepare Slopity for a high-quality public source-available release:
1. Licensing: Adopt the standard, official, unmodified PolyForm Noncommercial License 1.0.0 (`PolyForm-Noncommercial-1.0.0`), update Cargo package license metadata, add `LICENSE.md`, add `COMMERCIAL-LICENSE.md`, update all documentation to describe Slopity as source-available rather than open-source, and document external contribution policy.
2. Dependency Reproducibility: Commit root `Cargo.lock` and `apps/slopity/package-lock.json`, update CI/build workflows and scripts to use `--locked` and `npm ci`.
3. GitHub Actions Hardening: Migrate CI and release workflows from self-hosted runners to standard GitHub-hosted `ubuntu-latest` runners, pin third-party actions to immutable full commit SHAs with version comments, enforce strict top-level read-only permissions with minimal write scopes.
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

### Delivered

1. Official unmodified PolyForm Noncommercial License 1.0.0 in `LICENSE.md` (`PolyForm-Noncommercial-1.0.0`) and dual-licensing commercial terms in `COMMERCIAL-LICENSE.md`.
2. Cargo workspace package metadata (`license = "PolyForm-Noncommercial-1.0.0"`, `license-file = "LICENSE.md"`) and frontend `package.json` updated with matching license identifier.
3. Locked dependency manifests committed (`Cargo.lock` with 434 crates, `apps/slopity/package-lock.json`).
4. CI (`.github/workflows/ci.yml`) and Release (`.github/workflows/release.yml`) migrated from self-hosted runners to GitHub-hosted `ubuntu-latest` runners with automated Tauri Linux prerequisite installation and locked builds (`--locked`, `npm ci`).
5. All third-party GitHub Actions pinned to immutable full commit SHAs with version annotations across all workflows.
6. Dependabot configuration in `.github/dependabot.yml` monitoring Cargo, npm, and GitHub Actions dependencies weekly.
7. Security audit workflow in `.github/workflows/audit.yml` running `cargo-audit`, `dependency-review-action`, and `npm audit`.
8. Git history and commit trees audited for credentials/secrets across all 78 commits with 0 secret leaks found.
9. Server profile identifier validation hardened in `crates/slopity-core/src/validation.rs`: strict `[A-Za-z0-9._-]{1,128}` validation via `is_valid_profile_id` and `MAX_PROFILE_ID_LENGTH`, preventing interior NUL bytes (`\0`) which trigger thread spawn panics in `slopity-runtime-http`, control character escape injections, and path traversal. Re-exported in `crates/slopity-core/src/lib.rs`.
10. Added 6 comprehensive validation tests in `validation.rs` covering valid ID forms, empty/whitespace IDs, oversized IDs (129 chars), NUL byte rejection, control characters (`\n`, `\r`, `\t`, ANSI escapes), and path traversal (`../`, `/`, `\`).
11. Public security policy overhauled in `SECURITY.md` with supported versions table, pre-release disclaimer, responsible disclosure via GitHub Private Vulnerability Reporting and maintainer contact, report requirements, and ethical testing boundaries.
12. Contributing guidelines updated in `CONTRIBUTING.md` with source-available inbound contribution terms, mandatory two-commit protocol, and reproducible setup instructions.
13. Public presentation updated in `README.md`, `TASK.md`, and `docs/releases.md` accurately detailing operational status (built-in HTTP server and child-process boundary operational; Java/Minecraft/Node.js/Python unbundled/unavailable), source-available license, and hosted release pipeline.

### Files changed

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

### Verification performed

```bash
cargo fmt --all -- --check
# Pass (0 diffs)

cargo test --workspace --all-features --locked
# Pass (20 tests passed: 16 core, 4 http runtime, 0 failed, 0 ignored)

cargo clippy --workspace --all-targets --all-features -- -D warnings
# Pass (0 warnings)

cd apps/slopity && npm ci && npm run tauri:check
# Pass (Finished dev profile in 1m 36s, debug binary built cleanly)

cargo audit
# Pass (0 vulnerabilities found across 434 crate dependencies)

cd apps/slopity && npm audit
# Pass (0 vulnerabilities found)
```

### Known limitations

- Android hosting background service bridge compiles and builds in debug APK/AAB mode; on-device multi-hour battery/throttling profiling remains to be validated on physical ARM64 hardware.
- Windows builds and packaging are deferred to post-release roadmap phases.
- Android production signing keys are not yet configured (debug builds are generated for prereleases).

### Follow-up work

- Proceed with physical Android ARM64 device testing and foreground-service endurance benchmarks.
- Review and triage Dependabot PRs as automated security dependency updates arrive.

>>>>>>> chore/public-release-hardening

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
- Android produced `app-universal-debug.apk` and `app-universal-debug.aab` and uploaded artifact `9025966939`, named `slopity-android-debug-b118419fd9f545f1dd8659d3cb33c5fdbe43c5b4`, size `148565466` bytes and digest `sha256:c1775f7163c48aea023632faf6d6da379eb16a35e22e9870b71e84e134b8bb7f`.
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
- Android compilation does not prove OEM background behavior or Google Play foreground-service policy acceptance.
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
- Added a Tauri-independent `slopity-core` ProfileStore that loads, seeds, validates, persists, and reloads profile collections.
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