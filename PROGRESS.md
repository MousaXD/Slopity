# Progress ledger

## Step 111: report incomplete Android native stop without rejected host restart

**Status:** PARTIAL (targeted fix and regression tests committed; exact-head GitHub-hosted checks and independent QA pending)  
**Declared:** 2026-11-11  
**Owner:** Agent 00  
**Baseline:** PR #43 exact integrated head `c58dcda4943d656e9dde4b448603680f93b0eb4f`  
**Work branch:** `fix/slopity-native-stop-pending-reconciliation-agent00` (targets owner integration branch; not main)

### Defect and safety invariant
- When the Android foreground-service notification requests Stop, Kotlin sets `stopRequestPending=true`; `registerStartRequest` correctly rejects further start/update requests until shutdown completes.
- If a Rust runtime fails to stop, `stop_servers_after_native_request` currently attempts `slopity_host_start` to refresh the count. That request is guaranteed to be refused by Kotlin's pending-stop latch, adding misleading recovery noise. The existing foreground host is intentionally not stopped while a potentially-live Rust listener remains.
- Required invariant: never attempt a native host restart/update from this pending-stop or notification-unavailable recovery path; retain potentially live runtimes and existing error evidence, return a precise actionable error, and retry shutdown only on later observations/explicit stop.

### Scope, non-goals and risks
- Intended files: `apps/slopity/src-tauri/src/lib.rs` (narrow stop reconciliation and pure regression checks), and this `PROGRESS.md` entry.
- Preserve admission fail-closed checks, store→orchestrator lock ordering, `HOST_OPERATION_GATE`, Kotlin native-stop state machine, host API contracts, runtime failure snapshots, notification safety, all frontend/UI functionality and code provenance.
- Do not change `main`, release workflows, dependencies, Android Kotlin, security suppressions, runtime provider registration or packaging; do not claim physical Android device validation.
- A stopped host must only be requested after no active runtime remains; an unsuccessful Rust stop must not be reported as successful host shutdown.

### Acceptance
- New Rust regression test covers an incomplete native stop with stop pending and the notification-unavailable case; no host restart/update in the incomplete-stop path.
- Run exact-head Rust formatting, locked workspace tests and strict Clippy, frontend checks, Linux Tauri, Windows NSIS, Android ARM64 manifest + **actual** native Kotlin JUnit tests, security audit and PR package validation (publishing skipped) via GitHub Actions.
- Obtain independent Agent 06 review on combined owner integration SHA after this fix; keep PR #43 draft, do not merge without independent GO and branch-protection requirements.


### Implementation and validation status

- `stop_servers_after_native_request` now accepts the observed host status and never calls `slopity_host_start` when one or more Rust runtimes remain active after a pending stop or an unavailable notification. It leaves the existing native host alone (if present), preserves potentially-live runtime state, and returns an explicit incomplete-shutdown error including surviving count and runtime-stop failures.
- Added two Rust tests for pending native stop and notification-unavailable conditions, including explicit text that shutdown was not confirmed. Neither test is a physical Android device test; testing the pure error-decision function does not itself prove native calls.
- No Kotlin, frontend, workflow, security, dependency, runtime-registration or release behavior changed.
- **Hosted CI corrective finding:** [CI #42929811194](https://github.com/MousaXD/Slopity/actions/runs/42929811194) passed rustfmt but failed the new invisible-notification test: `host_reconciliation_action` returned `None` when Android reported active hosting without a visible notification while delivery flags appeared enabled. Added an **Android-only** fail-closed visibility check; desktop has no notification, so it is explicitly exempt. Updated the test message and added a desktop regression.
- **Validation pending:** exact-head hosted Rust fmt/test/strict Clippy and integrated Linux/Windows/Android, native JUnit, package and security gates must pass at the new code SHA before any merge; no local/device testing claim. Keep PR #43 draft pending independent QA.


## Step 110: serialize Tauri host-service operations across runtime lifecycle commands

**Status:** PARTIAL (host-operation gate and two deterministic Rust interleaving regressions implemented; exact-head hosted CI pending)  
**Declared:** 2026-11-08  
**Coordinator:** Agent 00

### Specific verified cross-command hazard
- PR #53 atomically closes the profile-store -> orchestrator startup race, but the returned active server count is a snapshot taken **before** the native host-service call.
- Two concurrent starts can finish host updates in the wrong order (newer count 2 overwritten by stale count 1). A stop can reconcile to count 0 while a prior start is still pending and the old start can subsequently re-enable the host service with no running server.
- A dashboard/status/list reconciliation can also interleave with lifecycle updates. Avoid any false safety claim.

### Scope
- Own only `apps/slopity/src-tauri/src/lib.rs`, this progress ledger, and focused tests.
- Add a single fail-closed mutex gate for host-affecting Tauri command lifecycles (start, stop, status, list, dashboard) so runtime modifications and native host-service reconciliation do not interleave across those commands.
- Preserve existing store -> orchestrator ordering, release both state mutexes before native calls, retain all failure evidence and native pending-stop fail-closed semantics.
- Use deterministic multithreaded gate regressions, including proof a second command cannot overtake a pending host update and that post-stop count cannot be overwritten by stale start.

### Non-goals and safeguards
- Do not change adapter behavior, Kotlin service, dependency versions, provider support, native UI, watchdog policy, audit exemptions, workflows or release settings.
- No host operation mutex acquisition inside helpers already called by gated commands; avoid recursive mutex deadlock. No physical-device success claims.
- Exact-head CI must pass Rust fmt, tests, strict Clippy and all required platform/security/package gates after implementation. Independent QA remains mandatory. Any hosted CI run on the pre-fix integration SHA becomes historical evidence only.

### Implementation and validation status

- Added `HOST_OPERATION_GATE` and `lock_host_operation` to the Tauri adapter, covering dashboard, host status, server-list reconciliation, start and stop commands. Native host calls remain outside both profile and orchestrator mutexes.
- Added deterministic `host_gate_prevents_stale_start_count_after_concurrent_stop` and `host_gate_prevents_older_start_from_overwriting_second_server_count` tests. Both stage a first host operation and assert the second cannot overtake it before validating the final native count.
- No local test claims: Rust formatting/tests/strict Clippy and platform/security/package checks must run on the new combined GitHub SHA; previous isolated/combined SHA runs are historical only.
- Keep PR #43 draft until exact-head CI, independent QA and review protection are satisfied.

## Step 109: owner integration of specialist fixes and guarded main merge

**Status:** PARTIAL (specialist source integration prepared; exact-head CI and independent QA pending)  
**Declared:** 2026-11-08  
**Coordinator:** Agent 00; integration PR #43

### Scope and exact inputs
- Preserve the existing owner integration branch and integrate the reviewed source from PRs #46 (Rust admission/lifecycle), #47 (CI and Android JUnit gate), #50 (GUI), #51 (Android foreground safety), #53 (Tauri atomic profile/start transaction), and #54 (stacked security update that includes #48). Keep all specialist development histories and record exact heads in the integration commit.
- Integrate PR #54 on top of PR #48 as a unit; retain the Tokio 1.53.3 and Tauri 2.11.6 security fixes.
- Preserve the red historical QA PR #49 without duplicating its regression tests, which are already embedded in PR #46.
- Reconcile the shared PROGRESS.md additions without dropping prior entries. For non-overlapping specialist source paths, preserve their exact blob content; review the combined Tauri, Rust-core, host-service, JavaScript and CI contracts, including runtime host counts and error evidence.

### Intended files and risks
- Intended: all changed source/test/docs/workflow paths reviewed in PRs #46/#47/#48/#50/#51/#53/#54, this ledger and engineering status notes if needed.
- Risks: cross-PR runtime semantics despite non-overlapping paths; start/stop interleavings and Android pending stop; native Android generated Gradle/JUnit may pass on isolated heads but fail when combined; one or more dependencies remain audit warnings; CI queue/environment failures must not be misreported as code success.
- Lock order is store then orchestrator. No native platform call while both locks are held. Admission/profile mutation atomicity and host start acknowledgement must stay fail-closed.

### Non-goals
- No direct main edits, force merges, disabled assertions, ignore lists to mask known vulnerabilities, relaxed runtime validation, production release publishing, unverified external runtimes, device-test claims, or signed-release claims.

### Required hosted acceptance before main promotion
- Run exact-head PR #43 Rust fmt/locked metadata/workspace tests/strict Clippy; frontend format/lint/Node tests; Linux Tauri; Windows x64 NSIS; Android ARM64 manifest and debug APK/AAB; real generated Kotlin/JUnit tests with nonempty successful XML; Linux .deb/AppImage; Cargo/npm audit and dependency review; explicit no publication on PR.
- Obtain fresh independent QA GO — DEVELOPMENT MERGE at that same SHA, satisfy branch protection/reviews, merge via PR, verify final main and post-merge CI. Track glib and other debt (#52), physical device durability (#44) and non-production release limitations truthfully.
- If any gate is missing or fails, keep PR #43 draft and record precise blockers.


### Integration assembly record

- Integrated source from PRs #46 (Rust), #47 (CI), #50 (GUI), #51 (Android), #53 (Tauri concurrency) and #54 (stacked on #48). Exact source blobs and all agent commits are to be preserved as merge parents; #54 includes the Tokio 1.53.3 and Tauri 2.11.6 security updates.
- PR #49 is a historical failing test, not to be copied separately. Original QA regression tests are already in PR #46.
- Cross-platform build, Kotlin/JUnit, cargo/npm audit, PR publication skip and fresh independent QA against the combined HEAD are **not yet verified**. Leave PR #43 draft pending verification.
- Audit debt #52 and physical-device verification #44 remain independent limitations; not a signed or production-ready release.

## Source history — specialist PR #46

## Step 105: retain independent QA admission regressions in Rust test suite

**Status:** PARTIAL (QA regressions committed; new exact-head CI pending)
**Declared:** 2026-11-07
**Updated:** 2026-11-07
**Owner:** Agent 02, branch `fix/slopity-rust-safety-agent02`
**Baseline:** Step 104 exact SHA `11356868918b146be98bc5989bee78504544ee8f` and [successful cross-platform CI 42285264906](https://github.com/MousaXD/Slopity/actions/runs/42285264906).

### Scope
- Independent QA's [test-only PR #49](https://github.com/MousaXD/Slopity/pull/49), SHA `48e2e9f4b6215f7bd53a4c315670c89a70ea8ef4`, demonstrated two baseline defects: disabled-after-admission and loopback-to-LAN-after-admission permit reuse.
- Carry both exact, named QA regression tests into the fixed Rust `admission.rs` module, preserving the QA assertions without reducing existing orchestration tests or the full-profile permit match.
- Intended files: `crates/slopity-core/src/admission.rs` and `PROGRESS.md`. No production behavior changes expected.

### Non-goals and acceptance
- No changes to Tauri/Android/GUI/CI workflows, package graph, other agents' branches, `main`, `dev` or integration branch. Do not merge the intentionally red QA branch.
- Mandatory planning commit first; implementation/tests and status ledger in the second commit.
- Verify exact implementation SHA on GitHub-hosted Rust fmt, locked tests, strict Clippy, Linux/Android/Windows and package validation. Record incomplete jobs honestly; no local/device tests claimed.

### Implementation and evidence
- Copied QA's two named test cases `permit_rejects_profile_disabled_after_admission` and `permit_rejects_network_scope_expanded_after_admission` without modifying their assertion logic.
- Retained existing full-profile equality permit implementation, structured admission rejection, and all other tests unchanged.
- Changes: `crates/slopity-core/src/admission.rs` and same-commit `PROGRESS.md` only.
- Parent SHA `11356868918b146be98bc5989bee78504544ee8f` passed [cross-platform run 42285264906](https://github.com/MousaXD/Slopity/actions/runs/42285264906), including 86 Rust tests, formatting, strict Clippy, Linux Tauri, Android ARM64, and Windows NSIS. This does **not** certify this newly tested head.
- This head needs new exact-head Actions. No local tests or physical device checks performed.


## Step 104: apply GitHub-hosted Rust formatting gate for Step 103

**Status:** PARTIAL (exact formatting patch committed; new-head CI pending)
**Declared:** 2026-11-06
**Updated:** 2026-11-06
**Owner:** Agent 02, branch `fix/slopity-rust-safety-agent02`
**Observed failure:** [Cross-platform CI 42284944250](https://github.com/MousaXD/Slopity/actions/runs/42284944250), Rust quality job `128123228627` at `3169e211436d3273cd767baeaeb23272c1ac0c26`.

### Scope and non-goals
- Apply only the three exact `rustfmt` layout diffs reported by Rust 1.89.0 in `crates/slopity-runtime-local/src/lib.rs`: nested std import, multiline start request, multiline chained observation assertion.
- Preserve all Step 102/103 semantics, security checks, tests, process-handling behavior and existing assertions unchanged.
- Intended files: `crates/slopity-runtime-local/src/lib.rs`, `PROGRESS.md`.
- No workflows, GUI, shared Tauri, Android Kotlin, `main`, `dev`, integration edits, or merges.

### Acceptance
- Planning commit precedes code change. Implementation commit carries only formatting and closure of this entry.
- Exact-head GitHub-hosted formatting, locked workspace tests, strict Clippy and Linux/Android/Windows/build-package jobs. The failed Step 103 run did not execute tests or Clippy.
- Maintain draft PR #46 until the last implementation SHA passes; no local tests or device proof claimed.

### Implementation and evidence
- Applied exactly three rustfmt-requested formatting changes to `crates/slopity-runtime-local/src/lib.rs`, without changing process semantics, regression test logic, or lint settings.
- Updated this `PROGRESS.md` entry in the implementation commit.
- Prior run [42284944250](https://github.com/MousaXD/Slopity/actions/runs/42284944250) failed solely at `cargo fmt --all -- --check` and never ran the workspace test/Clippy steps; downstream platform jobs were skipped.
- New-head GitHub-hosted exact-SHA checks remain pending. No local build or physical-device checks performed.


## Step 103: observe and reap desktop child-process exits

**Status:** PARTIAL (source and tests committed; exact-head CI pending)
**Declared:** 2026-11-06
**Updated:** 2026-11-06
**Owner:** Agent 02, branch `fix/slopity-rust-safety-agent02`
**Parent implementation SHA:** `d6714fe21665e10fda6180fdca91f4635870db96`

### Scope
- The currently unregistered `LocalProcessRuntime` stores process handles, but implements no `observe` or `observations`. A natural child exit remains a stale Running state in `ServerOrchestrator` and a stale process handle prevents restart.
- Implement nonblocking, trustworthy child `try_wait` observations and record terminal states. A zero exit is a completed stop, a nonzero exit a failure with diagnostic evidence; an inconclusive wait error must not discard a potentially live child.
- Reap completed processes before new start attempts and retain known terminal observations so the orchestrator can detect unexpected exits; cleanly reset a restarted server's terminal snapshot.
- Add tests for nonzero exit, clean exit, and restart eligibility after a terminated child.
- Intended implementation files: `crates/slopity-runtime-local/src/lib.rs` and this same-commit `PROGRESS.md`.

### Non-goals and safety
- No registration of external runtime providers, actual Java/Node/Python/PHP support claims, shell commands, new OS process-kill permissions, or Android process execution.
- No edits to shared Tauri, Kotlin, GUI, workflows, `main`, `dev` or integration branch.
- Do not erase errors or report a possibly live process terminal merely because `try_wait` fails; keep structured executable arguments, path validation and the Step 102 stop-failure protections.

### Acceptance
- GitHub-hosted exact-head `cargo fmt --all -- --check`, `cargo test --workspace --all-features --locked`, strict Clippy, Linux/Android/Windows and package validation, with exact run links recorded truthfully. No local or physical-device tests claimed.
- Commit A is this plan only. Commit B combines the source changes, regression tests and this closed ledger entry; keep PR #46 draft while exact-head CI is not certified.

### Implementation and evidence
- Implemented `RuntimeAdapter::observe` and `observations` for the dormant local process adapter; `try_wait` polls distinguish live processes from clean exit, nonzero exit, and inconclusive poll errors.
- Natural exits are reaped and recorded as terminal observations. Failed exits retain exit status and last-error diagnostics for orchestrator failure capture; an inconclusive poll retains the live child handle.
- A newly started child replaces stale terminal observations. Local-process `stop` records a user-requested stop when kill/wait confirms termination, or a known terminal observation if the child already exited.
- Added Rust regression tests for clean process exit, nonzero exit diagnostics, and restart after natural exit without shell execution. Android still refuses adapter launches.
- Changed files: `crates/slopity-runtime-local/src/lib.rs`, `PROGRESS.md`.
- No local, physical Android, or manual process/GUI test was run. All exact-head CI checks are pending; no claim of full runtime-provider support. Previous PR #46 Actions runs were superseded by the Step 103 commits and cannot certify the new head.


## Step 102: Agent 02 Rust admission and lifecycle fail-closed hardening

**Status:** PARTIAL (implementation submitted; exact-head GitHub-hosted validation pending)
**Declared:** 2026-11-06
**Updated:** 2026-11-06
**Owner:** Agent 02, branch `fix/slopity-rust-safety-agent02`
**Baseline:** integration PR #43 head `31d524fba0b72a4a79ec7d6bac3b97c749d564f2`

### Scope and acceptance
- Review Rust core, admission/resource reservations, lifecycle observations, profile recovery, built-in HTTP and dormant local-process adapter for actual correctness and security defects.
- Bind start permits to the *entire* validated profile, preventing post-authorization mutations (enabled, networking and process configuration) from reusing admission.
- On adapter start failures, retain authoritative active observations and failure diagnostics instead of incorrectly marking a potentially live workload terminal.
- Preserve local-process child handles when kill or wait fails so stop can be retried and ownership is not silently lost.
- Add focused Rust regression tests for modified authorization and startup failure behavior; preserve existing failed-stop and telemetry tests.
- Intended implementation files: `crates/slopity-core/src/admission.rs`, `crates/slopity-core/src/orchestrator.rs`, `crates/slopity-runtime-local/src/lib.rs`, and this `PROGRESS.md`.

### Non-goals and safety
- No changes to `main`, `dev`, integration branch, Tauri bridge, Android Kotlin, GUI, workflows, package dependencies, or release signing.
- No new runtime-provider registration or support claims, shell command strings, downgraded admission rules, skipped assertions, or weakened Clippy.
- Agent 00 owns Tauri `lib.rs` and will receive any cross-file findings separately.

### Risks and checks
- Changing startup failure handling must avoid false terminal evidence, stale diagnostics and loss of retry/cleanup visibility.
- Local-process kill/wait errors must never orphan Slopity's in-memory process handle.
- Verify via GitHub-hosted exact-head `cargo fmt --all -- --check`, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and relevant Linux/Android/Windows build jobs; report any incomplete gate accurately.
- No local build or device test is claimed. This step requires a second commit combining implementation, regression tests, and the completed ledger entry.

### Implementation and regression coverage
- Admission permit captures the entire validated `ServerProfile`, including enabled status, network exposure and executable arguments rather than only ID/runtime/port/memory.
- Orchestrator checks the adapter's observation after failed startup and conservatively keeps a still-active workload in snapshots (with a persistent startup error), so it remains visible and stoppable after partial launch.
- The status reconciliation preserves live-state failure diagnostics until a fresh adapter error or terminal observation replaces them.
- Desktop local-process termination keeps the child handle until kill/wait is confirmed; an already exited child may be reaped without treating it as a live stop failure.
- Rust regressions cover changed-profile permit reuse, partial startup with live observations and subsequent stop, plus already reaped child cleanup.
- Additional in-scope execution-path hardening: resolve the profile executable and working directory to concrete absolute filesystem paths before calling `Command::new` and `current_dir`. This closes the mismatch between `is_file()` checking a relative file and OS PATH lookup executing a different binary; added a non-file execution rejection test. Local-process registration is still intentionally disabled in the app.
- Changed source: `crates/slopity-core/src/admission.rs`, `crates/slopity-core/src/orchestrator.rs`, `crates/slopity-runtime-local/src/lib.rs`; this progress entry is included in the implementation commit.
- Acceptance is pending **this branch's final implementation SHA** in GitHub Actions; validation on an earlier commit does not certify subsequent source changes. The integration PR #43 baseline at `31d524f` had [Cross-platform CI 42275322969](https://github.com/MousaXD/Slopity/actions/runs/42275322969) and [Security audit 42275322860](https://github.com/MousaXD/Slopity/actions/runs/42275322860) successful; these historical results do not certify Agent 02 changes. No local checks are claimed.

## Source history — specialist PR #47

## Step 036: gate Android native plugin JUnit tests in hosted CI

**Status:** PARTIAL (native test gate implemented; exact-head hosted CI pending and PR #51 tests not present on this branch)  
**Declared:** 2026-11-07  
**Updated:** 2026-11-07

### Verified baseline and review request

- Step 035 implementation SHA `895ca859dc27bad1823264380d871c3101fb1129` passed [cross-platform CI 42284217620](https://github.com/MousaXD/Slopity/actions/runs/42284217620), [security audit 42284217458](https://github.com/MousaXD/Slopity/actions/runs/42284217458), and [release-package CI 42284217616](https://github.com/MousaXD/Slopity/actions/runs/42284217616), including both Linux .deb/AppImage and Android debug packages; PR publishing skipped.
- Independent QA [comment 6694209878](https://github.com/MousaXD/Slopity/pull/47#issuecomment-6694209878) and Android owner [comment 6694244126](https://github.com/MousaXD/Slopity/pull/47#issuecomment-6694244126) identify missing hosted Kotlin/JUnit lifecycle test execution from Android [draft PR #51](https://github.com/MousaXD/Slopity/pull/51). Android ARM64 compile alone is not native JVM unit test certification.
- PR #51 provides `HostForegroundServiceTest.kt` and `junit:junit:4.13.2` in the Android library. Those files are **not yet in this CI branch's base** and must not be copied into another owner's lane.

### Intended scope / files

- `scripts/check-android-host-junit.sh`: following Tauri Android debug build, invoke generated Android Gradle wrapper's `testDebugUnitTest` from the generated multi-module project and verify plugin-specific, nonempty JUnit XML results when the lifecycle test source exists.
- `.github/workflows/ci.yml` and `.github/workflows/release.yml`: add Android native host unit-test step **after Tauri Android build and before manifest/artifact acceptance** in both hosted Android workflows, keeping artifact, manifest and no-publish gates unchanged.
- `PROGRESS.md`: implementation and exact-run evidence.

### Fail-closed acceptance, limitations and risks

- If the lifecycle source test exists, missing Gradle test task/project, missing or empty plugin JUnit XML, skipped tests, failures or errors **must fail CI**. Gradle cannot silently pass on `NO-SOURCE`.
- If lifecycle tests are absent from this branch, explicitly label the test source missing and the resulting execution unverified; the Gradle unit-test task will still be invoked to validate its availability, but **this is not proof that PR #51's new tests ran**. Agent 00 must validate merged PR #51 together with this gate.
- Generated Android Gradle modules/settings are created by Tauri Android build; test step must run after build. Hosted Gradle may need additional dependency downloads and can reveal real test bugs.
- Preserve Rust fmt/locked tests/strict Clippy, frontend gates, all 4 platform build targets, audit/dependency review, action SHA pins, job limits, and PR no-release publication policy.
- No edits to Android/Kotlin application or plugin business logic, dependencies, signatures, production release settings, other agent branches, `main` or `dev`. No local-test claims.
- Require exact new-head GitHub-hosted CI and release results and describe absent native test evidence truthfully.

### Implementation and evidence at implementation commit

- Added `scripts/check-android-host-junit.sh` to run generated Gradle `testDebugUnitTest` after the Tauri Android build, using GitHub-hosted JDK 17/SDK/NDK setup.
- Validates nonempty JUnit XML for the exact `com.slopity.host.HostForegroundServiceTest` class and rejects missing reports, zero tests, skipped tests, errors and failures when the source exists. A successful Android APK compilation or Gradle `NO-SOURCE` alone is never reported as lifecycle test certification.
- Added the native JVM test command to the CI and release-package Android jobs, preserving ARM64 debug builds, manifest checks and artifact assertions.
- **Branch boundary:** PR #51's Kotlin/JUnit source is not yet integrated into this branch's base. If the source is absent, CI prints an explicit GitHub warning that these lifecycle tests have **not** been certified. Agent 00 must validate the combined tree with PR #51 after integration.
- No Android plugin source, dependency versions or Rust/GUI files modified; no local tests run. GitHub-hosted new-head validation is pending at implementation commit.
- Earlier Step 035 implementation was validated: [CI 42284217620](https://github.com/MousaXD/Slopity/actions/runs/42284217620), [security 42284217458](https://github.com/MousaXD/Slopity/actions/runs/42284217458), [release packages 42284217616](https://github.com/MousaXD/Slopity/actions/runs/42284217616) succeeded, publication skipped. These historical results do not certify this new script.

## Step 035: remove premature apt wall-clock cutoff from hosted Linux builds

**Status:** PARTIAL (implementation committed; exact-head GitHub Actions pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06

### Confirmed failure and rationale

- [PR #47 release run 42279623891](https://github.com/MousaXD/Slopity/actions/runs/42279623891) failed its [Linux .deb/AppImage job 128104932881](https://github.com/MousaXD/Slopity/actions/runs/42279623891/job/128104932881) during `Install Linux Tauri system dependencies`, with **exit 124** after the 900-second GNU `timeout` set in Step 034. The logs show apt still successfully fetching necessary packages until the forced cutoff.
- The unmodified parent PR #43's [release run 42275322783](https://github.com/MousaXD/Slopity/actions/runs/42275322783) subsequently **passed** Linux packaging and published no release. Its Linux job spent approximately **32 minutes 37 seconds** fetching 33.6 MB from a severely slow Ubuntu mirror; the explicit 900-second limit introduced a regression relative to that working base.
- [PR #47 cross-platform CI 42279623935](https://github.com/MousaXD/Slopity/actions/runs/42279623935) and [security audit 42279623942](https://github.com/MousaXD/Slopity/actions/runs/42279623942) passed; the Linux package failure is infrastructure download time, not evidence that application build or tests failed.

### Scope and intended files

- Remove both GNU `timeout` wrappers in the apt bootstrap steps from `.github/workflows/ci.yml` and `.github/workflows/release.yml`. Keep `Acquire::Retries=3` and per-request HTTP/HTTPS timeouts (30 seconds) to bound stalled acquisition attempts without prematurely rejecting a long but progressing package set.
- Preserve the original `timeout-minutes` limits on the GitHub jobs as the overarching wall-clock bound, all existing Ubuntu package names and flags, and all downstream Rust/frontend/package gates.
- Update `PROGRESS.md` in the implementation commit with exact-head CI evidence.

### Non-goals

- No changes to application code, platform targets, dependencies, audit policy, release signing, publication behavior, action pinning, user functionality, `main` or `dev`.
- No suppression of security advisories, disabling tests, skipping Linux artifacts, bypassing permission controls, or speculative mirror changes.

### Risks and acceptance

- Hosted Ubuntu mirrors can still be slow. Per-request timeout/retries can fail genuine network outages while GitHub job time limits prevent unbounded hangs; a fast download is not guaranteed.
- Require fresh GitHub Actions runs on exact implementation head: Rust fmt/locked workspace tests/strict Clippy, frontend tests/lint/format, Linux Tauri compile, Android ARM64 manifest and artifacts, Windows NSIS, Cargo/npm audits, dependency review, Linux .deb/AppImage and Android preview packages, with PR publication skipped.
- Keep draft PR until every required check passes; do not claim local verification.

### Implementation and verification at implementation commit

- Removed the fixed 300-second/900-second GNU `timeout` wrappers from apt bootstrap in **both CI Linux jobs and the release Linux job**.
- Retained apt acquisition `Acquire::Retries=3` and HTTP/HTTPS `Timeout=30` request timeouts, all original native packages and target/toolchain choices, GitHub job-level `timeout-minutes`, and every test, build, artifact, audit and no-publish gate.
- No application code, dependency versions or pinned GitHub Actions were modified.
- No local tests were run. Exact new-head GitHub-hosted CI, audits and release packages remain **pending** at implementation commit; report the new job URLs and final results separately and do not promote this PR until they pass.
- Parent PR #43's Linux release job passing is **not** evidence that this new commit passes; the confirmed Step 034 cutoff regression is recorded above.

## Step 034: bound Linux hosted dependency bootstrap without weakening checks

**Status:** PARTIAL (bounded bootstrap implemented; exact-head GitHub Actions pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06

### Evidence and scope

- PR #43 combined candidate at `31d524fba0b72a4a79ec7d6bac3b97c749d564f2`: [exact-head cross-platform CI 42275322969](https://github.com/MousaXD/Slopity/actions/runs/42275322969) and [security audit 42275322860](https://github.com/MousaXD/Slopity/actions/runs/42275322860) passed.
- [Release validation run 42275322783, Linux job 128090109452](https://github.com/MousaXD/Slopity/actions/runs/42275322783/job/128090109452) remained in `Install Linux Tauri system dependencies` from 18:57:40 UTC through repeated inspection; Android release packages passed. Live job logs were unavailable (GitHub returned 404) so the precise apt operation/root cause is unknown.
- Retain all original Linux package names and compiler/toolchain versions. Bound apt metadata refresh and package installation; explicitly configure network acquisition retries and timeouts so transient mirror problems retry and persistent stalls fail with a diagnostic before the overall job timeout.
- Apply the same targeted Linux apt bootstrap handling to Rust quality, Linux Tauri CI, and Linux release bundling, maintaining identical package sets and all downstream gates.

### Intended files

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `PROGRESS.md`

### Non-goals and security boundaries

- No changes to Rust business logic, GUI, Android host, dependencies, signing keys, publication conditions, pinned actions, tests, audit policy, release version or license.
- No assertion that this corrects the particular run's underlying problem; the running job's log is not accessible, and a hang could be transient hosted-runner infrastructure.
- No merge, release publication, direct `main`/`dev` writes, or local-test claims.

### Risks and acceptance checks

- Some mirrors or temporary GitHub runners can remain slow; bounded apt may turn indefinite stalls into explicit failures rather than guarantee a green build.
- After implementation, require exact new branch-head GitHub-hosted Rust formatting, locked tests, strict Clippy, frontend checks, Linux Tauri, Windows NSIS, Android ARM64 manifest/packages, Cargo/npm audit, dependency review, Linux .deb/AppImage, and PR publication skipped.
- Record exact runs and pass/fail/availability in this same entry when available; hold integration for Agent 00 if Linux packaging is not green.

### Implementation and verification at implementation commit

- Applied bounded GNU `timeout` to `apt-get update` (300 seconds) and dependency installation (900 seconds), with a 30-second termination grace period.
- Set `Acquire::Retries=3` and 30-second HTTP/HTTPS acquisition timeouts. Preserved the complete pre-existing packages, Tauri/build commands, pinned Actions, platform target choices, warning strictness, vulnerability checks, and PR publication guard.
- Updated both Rust and Linux CI apt steps and Linux release package apt step; no application or dependency files changed.
- At this implementation commit, exact new-head GitHub Actions checks are pending. Do not count the successful parent PR #43 runs as validation of these new workflow edits. No local tests were run.
- Remaining known review item outside CI ownership: PR #43 Cargo audit succeeded but emitted eight RustSec warnings (six unmaintained dependencies and two unsoundness advisories, including `glib` and `tokio`). They are **not** reported by cargo-audit as a zero-warning result; dependency/security owner must triage, not suppress.
- Root cause of the observed Linux runner stall remains unconfirmed until the original run's job log is available. If the Linux release still fails, retain draft status and investigate exact new-head log before further changes.

## Source history — specialist PR #50

## Step 047: prevent stale server polls from bypassing recovery

**Status:** PARTIAL (implementation committed; exact-head CI/manual verification pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 03 — GUI/UX

### Scope, risk and acceptance
- A backend mutation can commit while its dashboard reload fails; a concurrently executing server-list poll must never clear the stale-state warning or restore profile-mutation controls without an authoritative full dashboard refresh.
- Gate poll start and poll result application against busy/offline state and snapshot identity; discard results from older snapshots rather than overwriting fresher server data.
- Preserve manual Retry connection, emergency Stop of observed active servers, UI refresh settings, and existing backend calls without inventing state.
- Intended files: `apps/slopity/web/app.js`, `apps/slopity/web/domain/mutation-outcome.js`, `apps/slopity/web/tests/preferences-mutations.test.mjs`, and this `PROGRESS.md`.
- Non-goals: Rust, Android, CI/workflow edits, shared manifests, fake server states, platform runtime support, direct integration changes, merges, scheduled tasks.
- Checks: GitHub-hosted exact-head frontend format/lint/Node tests, Rust/Linux/Android/Windows and release validation. Manual app-shell behavior remains unverified.

### Implementation and test evidence
- Added explicit conditions to begin/apply server polling only while the latest snapshot is authoritative and the interface is not busy, hidden, or waiting for a full refresh.
- Captured snapshot identity before asynchronous `listServers` calls and discarded results/errors from superseded snapshots, so delayed polling cannot overwrite a newer dashboard or clear a stale-state lock.
- Polling only refreshes the server-list portion of a snapshot; the stale-state warning can now clear only through a successful full `dashboard_snapshot` request.
- Added Node regression tests for stale refresh lock, hidden/busy/pending suppression, and out-of-order asynchronous result rejection.
- No local tests or physical-device tests run. Exact-head CI and manual keyboard/real backend validation remain pending.


## Step 046: truthful mutation outcomes and functional UI refresh settings

**Status:** PARTIAL (implementation committed; exact-head GitHub Actions and manual UX checks pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 03 — GUI/UX

### Scope and intended files
- Fix a real UX/data-integrity risk: a profile/server operation may succeed, then the dashboard refresh may fail. Do not report the committed operation as failed or encourage accidental retry; retain the existing snapshot, communicate stale state, and require refresh before further non-emergency mutation.
- Continue allowing Stop for an observed active server during stale-state recovery, subject to backend policy; never falsely claim that a failure is success.
- Add a real Settings screen for UI-only auto-refresh on/off and interval (2/5/10/30 seconds), with safe optional local persistence and explicit messaging that these options do not affect native server hosting.
- Improve busy/offline controls on server cards and details so users cannot launch duplicate mutations while the backend state is stale; retain read-only details/export and retry controls.
- Add Node tests proving mutation/action vs refresh outcomes, preference validation/storage failure, and rendering behavior. Preserve all existing functionality.
- Intended files: `apps/slopity/web/app.js`, `index.html`, `styles.css`, `components/server-card.js`, `views/dashboard.js`, new `domain/ui-preferences.js`, new `domain/mutation-outcome.js`, new frontend tests, `PROGRESS.md`.
- No Rust, Android, native settings, package manifests, GitHub workflows, or shared integration changes.

### Risks, non-goals and acceptance
- An action's failure must never be converted to success; distinguish only the separately failing follow-up snapshot request.
- Cached/stale dashboards are never evidence of live backend status. Blocking profile starts during unknown state must not remove the user's ability to attempt Stop on an already-observed active server.
- Local storage can be unavailable in WebViews; settings must still work for the current session. Do not claim local preferences change hosting durability.
- No mocked backend success, unsupported runtimes, executable imports, or scheduled tasks.
- Acceptance: exact-head GitHub-hosted frontend format/lint/Node tests, Linux Tauri, Rust, Android and Windows where applicable, release package validation, manual app-shell keyboard/visual/WebView verification documented truthfully.

### Delivered
- Added an operational Settings screen to desktop/mobile navigation with auto-refresh toggle, validated 2/5/10/30-second intervals, read-only UI polling semantics, safe local persistence and explicit session-only fallback.
- Fixed false failure reporting when a backend action commits but the subsequent dashboard snapshot request fails. User-facing notices now distinguish committed mutation from missing refresh. Stale-state guards block new profile mutations/server starts until a full refresh succeeds, while emergency Stop remains available for servers observed active.
- Added disabled/busy states for server cards/details and backend-unavailable profile actions without suppressing actual backend admission errors.
- Added Node tests covering preference loading, malformed values, storage unavailability, action failure versus post-mutation refresh failure, successful refresh and stale/busy lifecycle button behavior.
- Changed only frontend source/test files and this ledger. No native/platform/workflow manifests were edited.

### Evidence and limitations
- Previous head `acc32598e1f4493261c4b7e36eb5c5444ac221da` passed cross-platform CI [42281605318](https://github.com/MousaXD/Slopity/actions/runs/42281605318): Rust format/locked tests/strict Clippy, Linux Tauri frontend format/lint/Node tests and compile, Windows NSIS, Android ARM64 debug. Release package validation [42281605320](https://github.com/MousaXD/Slopity/actions/runs/42281605320) passed Linux and Android packaging; publish skipped.
- Those green checks validate only the *prior* head. This step requires fresh exact-head Actions evidence. No local tests or physical Android device tests were run.
- Manual keyboard/screen-reader, cross-viewport UI, Tauri desktop/Android actual downloads and real backend failure reproduction remain unverified. Security audit workflow filters dependency changes; it did not run on these frontend-only commits. CI Linux job independently runs `npm audit`.


## Step 045: preserve Rust-compatible profile metadata through JSON transfer

**Status:** PARTIAL (implementation committed; exact-head CI pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 03 — GUI/UX

### Scope, non-goals, risks and acceptance
- Audit versioned profile import/export validation against the actual `slopity-core` model and validation constants, especially UTF-8 path/argument limits and u32 memory budgets.
- Preserve existing built-in HTTP configuration metadata (which the backend warns about but does not execute); a JSON import is never a runtime launch or executable-file import.
- Retain strict JSON format, field allowlist, profile-count/file-size limits, conflict skipping, disabled-on-import behavior, backend validation, and truthful partial-failure states.
- Avoid exporting JSON that fails the declared file limit. Increase the bounded document limit if necessary to support the Rust model with up to 100 profiles.
- Intended files: `apps/slopity/web/domain/profile-transfer.js`, `apps/slopity/web/app.js`, `apps/slopity/web/tests/discovery-transfer.test.mjs`, and this `PROGRESS.md` only.
- No shared manifests, Rust/Android/workflows, filesystem execution, runtime enablement, merges or release operations.
- Acceptance: exact-head GitHub Actions frontend tests, format/lint and platform builds; manual WebView download behavior remains unverified.

### Implementation and evidence
- Aligned JSON profile validation with Rust's 4,096-byte path fields, 256 arguments / 8,192 UTF-8 bytes each / 65,536-byte combined argument payload and u32 memory value. The backend continues to apply its runtime/memory policy independently.
- Preserved inert executable/working-directory/argument metadata even on built-in HTTP profiles, matching the persisted model (the backend warns but never executes these fields).
- Bounded JSON document import **and export** to 8 MiB with at most 100 profiles; oversized/unsupported content fails before backend mutation.
- Expanded regression tests for compatible metadata round trips, malformed payload rejection, UTF-8/argument limits, and memory overflow.
- No local checks or Android WebView download tests claimed; GitHub-hosted exact-head runs are required after committing this implementation.


## Step 044: retain keyboard focus through live card updates

**Status:** PARTIAL (implemented; exact-head Actions and manual keyboard checks pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 03 — GUI/UX

### Scope, risks and checks
- Keep keyboard focus on the same server action when the two-second backend state poll or filter redraws server cards. Never steal focus from a dialog or another view; do not fabricate lifecycle state.
- Give every server action an accessible profile-specific name. Let the Runtime unavailable filter find unavailable adapters even if the profile is disabled; preserve the primary disabled status for badges.
- Limit changes to `apps/slopity/web/components/server-card.js`, `views/dashboard.js`, `domain/profile-query.js`, their dedicated Node tests, and this ledger.
- Risks: focused rows may be removed by filters/deletion, in which case do not redirect focus to unrelated controls. CSS selector values must remain allowlisted to avoid injection.
- Acceptance: exact-head GitHub-hosted frontend format/lint/Node regression tests and platform builds; no local or device-test claims. Manual keyboard checks remain necessary.
- Non-goals: Rust/Android/package/CI changes, new runtimes, fake backend controls, merges, or releases.

### Implementation and verification
- Card lifecycle and Details buttons now have profile-specific accessible labels and stable action identities.
- Re-rendering a card restores keyboard focus to the same action when it remains available, or to that card's Details action if the lifecycle action became disabled; no focus theft from other views or removed profiles.
- Runtime unavailable filter now includes disabled profiles with unsupported backend adapters, while their primary badge still correctly reports disabled state.
- Added dedicated Node regression cases for focus retention/fallback and updated filter behavior.
- No local tests claimed; GitHub Actions exact-head frontend, Rust, Linux/Android/Windows and package checks are pending. Manual keyboard/assistive-technology tests still required.


## Step 043: Agent 03 GUI V2 discoverability, accessibility and safe profile transfer

**Status:** PARTIAL (implemented; exact-head GitHub Actions and manual app-shell UX verification pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 03 — GUI/UX (branch `feat/slopity-gui-agent03`; base `feat/owner-integration-20261106`)

### Scope and acceptance
- Audit GUI V2 dashboard, server cards/details, profile editor, runtime registry, device status and dialog behavior against the real Tauri API.
- Add server search, runtime/status filtering and sorting, understandable result counts and no-match states; do not confuse saved-profile state with observed backend state.
- Improve initial loading, backend-offline/retry behavior, accessible notices/focus and safe modal interactions without inventing backend success.
- Provide versioned JSON-only profile import/export with strict schema, size/count guards, preview, duplicate-ID conflict handling, never auto-start, and per-profile backend validation/persistence; do not overwrite existing profiles or import executable payloads.
- Add tests for selectors, import parsing/preview/conflicts, error handling, and accessibility-facing rendering.
- Intended files: `apps/slopity/web/index.html`, `app.js`, `styles.css`, `views/dashboard.js`, `views/dialogs.js`, `views/profile-editor.js`, new focused `domain/` and `views/` modules, frontend `tests/`, and this `PROGRESS.md`. No shared package manifest, Rust, Android, or workflow edits.
- Acceptance checks: GitHub-hosted exact-head frontend format, lint, Node tests; Linux/Android/Windows CI and security/release workflows as applicable. Record exact results and manual accessibility/device verification gaps.

### Implementation submitted
- Added pure server selectors with name/ID/runtime/port search, actual observed-state filtering, runtime filtering, port/status/name sort and match counts. Empty/no-match screens distinguish zero saved data from filtered results.
- Added versioned, strict JSON-only profile transfer with 256 KiB and 100-profile limits, allowlisted fields and runtimes, type/range validation, disabled-on-import policy, safe DOM preview, existing/duplicate ID skip behavior, per-profile backend validation and partial-failure reporting. Existing profiles are never overwritten or auto-started.
- Added responsive search/filter controls, JSON import/export controls, backend connection/retry state, informative failure notices, protected polling, editor double-submit guard, backend error formatting and native-dialog focus restoration. The UI still uses the real Tauri API, not simulated commands or resources.
- Added `web/tests/discovery-transfer.test.mjs` for selectors, filtering, sorting, import/export and malicious payload rejection, plus existing frontend tests retained.
- Changed only `apps/slopity/web/**` and this ledger. No Rust, Android, workflows or shared manifests changed.

### Verification status and limitations
- No local test execution is claimed; the required frontend format/lint/Node tests and Rust/platform checks must run on the exact implementation SHA in GitHub Actions after the draft PR is opened.
- Manual keyboard/assistive-technology testing, narrow mobile layouts, Tauri Android/desktop WebView JSON download behavior, browser-native Escape focus and physical device telemetry remain unverified.
- Bulk import intentionally reports partial success when backend validation or creation rejects an individual profile; rerunning skips IDs already persisted. External runtime fields remain configuration metadata, never executed by import.
- Follow-up: inspect exact-head jobs, resolve real failures with a newly planned two-commit correction step if needed, then request coordinator review; do not merge this draft PR.

### Non-goals and risks
- No external runtime activation, shell execution, import of runnable files, arbitrary filesystem paths, silent profile replacement, pretend telemetry, backend admission policy edits, or merges.
- All JSON is untrusted; invalid items must not be saved. Bulk import can partially succeed if the backend rejects an item; report the partial result explicitly.
- Browser/WebView file download and keyboard/assistive-technology behavior require manual app-shell validation beyond Node tests.

## Source history — specialist PR #51

## Step 047: preserve an already-foregrounded Android host after update promotion failure

**Status:** PARTIAL (safety correction committed; exact-head native/JVM and hosted CI pending)  
**Declared:** 2026-11-07  
**Updated:** 2026-11-07  
**Owner:** Agent 04 — Android hosting & durability  
**Branch:** `fix/slopity-android-agent04`  
**Base:** `1340ecb5a09dbd0ffaaafbf91fe84c738b5b4805`

### Finding and scope

- `HostForegroundService.onStartCommand` currently calls `markStartFailed` and `stopSelf` unconditionally when `startForeground()` throws, even if this is a rejected **update** while a previous foreground notification protects Rust-owned listeners. This unconditionally removes the previous foreground service and risks orphaning live servers with no notification.
- Reject only the failing native start/update request and preserve previously acknowledged active foreground state and count when a promotion error occurs during an update. An unsuccessful initial foreground promotion still resets state, rejects the request and stops the service. Notification permission/channel gates remain checked independently by the Rust application, which must reconcile/stop servers on delivery loss.
- Scope limited to `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostForegroundService.kt`, `plugins/tauri-plugin-slopity-host/android/src/test/java/com/slopity/host/HostForegroundServiceTest.kt`, Android README, and this progress ledger. Do not modify Agent 01 workflows or Agent 00 application Rust code.

### Verification and limits

- Add JVM lifecycle tests for promotion-failure preservation vs. first-start rejection and ensure callbacks fire exactly once without pending states.
- Check GitHub-hosted exact-head Android ARM64 build, Rust quality, package workflows and **combined** CI Gradle JUnit suite using isolated validation branch based on Agent 01's CI gate.
- No claims about physical notification UI, service-kill durability, battery/thermal behavior or issue #44; no merging, publishing, signing or force pushes.

### Implementation evidence

- The Android service now distinguishes a rejected *initial* `startForeground()` promotion (native state cleared, request rejected, service stopped) from a rejected *update* on an already foregrounded service (only the rejected request is failed, previously acknowledged host/notification state retained).
- Two Kotlin/JUnit regression tests assert the split, exact callback result, and no stale pending state. An Android permission or channel failure still reports unavailable notification delivery on the next Rust observation, triggering fail-closed reconciliation; no claim of ongoing notification visibility after Android revokes delivery.
- Source/test/README and this ledger are committed together. Host-side Android compilation and the combined Agent 01 Gradle test gate must pass before the change is certified. Physical-device validation remains blocked by issue #44.


## Step 044: Agent 04 Android foreground-host start acknowledgement and stop-race hardening

**Status:** PARTIAL (implementation committed; exact-head CI and real-device verification pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 04 — Android Hosting & Durability  
**Branch:** `fix/slopity-android-agent04`  
**Base:** `feat/owner-integration-20261106` at `31d524fba0b72a4a79ec7d6bac3b97c749d564f2`

### Scope and verified baseline

- Audit native Kotlin `HostForegroundService` and Tauri `HostPlugin`, Android manifest, Rust plugin bridge, host lifecycle tests, and the device durability probe.
- Native `startHosting` currently resolves immediately after enqueueing `startForegroundService`, before the service confirms `startForeground` succeeded. In-process snapshots and notification visibility can therefore be mistaken for confirmed service state. Fix the native acknowledgement boundary without hiding failures.
- Preserve native notification Stop safely -> Rust reconciliation, protect pending stop from concurrent start/update, avoid stale native state after start/stop failures, and make channel/permission status conservative.
- Add focused lifecycle regression tests or testable state checks and improve device/probe documentation where the evidence path is unclear.

### Intended files

- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/{HostForegroundService.kt,HostPlugin.kt}` and adjacent Android test sources as appropriate.
- `plugins/tauri-plugin-slopity-host/src/lib.rs` only if required for honest status gating and compatible cross-platform tests.
- `plugins/tauri-plugin-slopity-host/android/README.md`, `docs/android-device-durability.md`, `scripts/android-durability-probe.sh`.
- `PROGRESS.md` (this plan and implementation evidence).

### Guardrails and risks

- Do not modify `apps/slopity/src-tauri/src/lib.rs` or overwrite Agent 00/02 runtime admission work; relay any required Rust application change to Agent 00.
- Preserve built-in HTTP-only runtime support, user-initiated foreground hosting, native stop semantics, notification permission/channel fail-closed behavior, and in-process state/unknown-device limitations.
- Android manifest/cross-compile passing is not physical-device foreground visibility, one-hour durability, thermal/battery safety, or Play policy certification.
- Acknowledgement must be nonblocking for the Android service main thread; concurrent requests, denied permission, delayed lifecycle delivery, and stop-request races need explicit failure behavior.
- No signing keys, releases, direct main/dev changes, bypasses, automated schedules, or manufactured device telemetry.

### Acceptance checks

- GitHub-hosted exact-HEAD Rust format, locked tests, strict Clippy; Android ARM64 debug compile/manifest and any available package/audit gates.
- Review native lifecycle error propagation, start/stop/notification gate transitions, count updates, and no fabricated `notificationVisible` assertions.
- Record precise GitHub Actions runs/job outcomes; no chat-local tests or physical device claim.
- Keep physical ARM64 device release certification `BLOCKED` in [issue #44](https://github.com/MousaXD/Slopity/issues/44) until explicitly authorized device evidence is reviewed.
- Open draft PR targeting `feat/owner-integration-20261106` for Agent 00; do not merge.

### Implementation and evidence at this commit

- Native foreground-service starts now use per-request asynchronous acknowledgement only after Android `startForeground()` returns; an unacknowledged request times out after 15 seconds. Canceled queued starts cannot promote a service, and service destruction rejects pending invokes.
- Preserve stop-pending state across attempted count updates and retain active state while a stop is requested; native `stopHosting` no longer reports unconditional successful shutdown or clears state after a thrown stop exception.
- Create/check the actual notification channel before Rust preflight; a missing channel is not considered enabled. Status visibility remains an inference rather than actual physical observation.
- Rust plugin requires acknowledged, nonpending, visible-gated, exact-count Android status; a failed acknowledgement rolls back the newly started Rust listener through existing application code, while avoiding blind termination of an existing host service relied upon by other listeners.
- Add Rust acknowledgement regressions for pending/denied/stopped/mismatched states; update Android bridge README and device durability cases; probe redacts device serial and timestamps ADB disconnect state.
- `apps/slopity/src-tauri/src/lib.rs` intentionally unchanged to avoid racing Agent 00's runtime admission work. Existing reconciliation is observation-triggered, not an always-on watchdog.
- Validation: changes are committed for GitHub-hosted exact-head Rust and Android ARM64 checks. No chat-local tests, emulator, or physical-device test was performed; no CI result claimed until the run completes. Physical certification remains **BLOCKED** under issue #44.
- Pre-PR static review follow-up: corrected nullable Android `Intent?` access in Kotlin, and latched native shutdown before requesting `stopService` so an overlapping queued start cannot outrun a Rust-owned stop. CI compilation remains pending.
- Android platform-restriction follow-up: replaced the new notification stop action's Android 12+ prohibited service-to-activity trampoline with a direct immutable Activity PendingIntent. Native `HostPlugin` processes the initial/new Activity Intent using a one-time per-process stop token; old service-action notifications keep a fallback. Rust-side reconciliation is still observation-triggered, not guaranteed by the Intent alone. No physical-device behavior claim.
- Added `HostForegroundServiceTest.kt` Android local JVM regression scenarios for delayed acknowledgement, shutdown cancellation, preserving an acknowledged host on an update failure, and failed promotion with multiple pending starts. Added a scoped JUnit test dependency. These tests must be run by a generated Android Gradle unit-test task in CI; merely checking in the tests is not passing evidence. Agent 01 owns workflow wiring if absent.
- Exact-head `076a5fc93dfb21f088a311cbfa2a91a0ffa10c6c` passed [Cross-platform CI 42281473924](https://github.com/MousaXD/Slopity/actions/runs/42281473924): Rust quality, Linux Tauri, Android ARM64 debug/manifest, Windows NSIS. Also passed [PR release-package run 42281473893](https://github.com/MousaXD/Slopity/actions/runs/42281473893): Linux and Android debug packages; release publication skipped. These checks do not execute Kotlin JUnit tests or prove physical-device lifecycle.
- Post-CI review identified overlapping native start requests could produce a successful `startForeground` callback with `startRequestPending=true`, causing Rust's confirmed-host check to reject the first caller and potentially introduce out-of-order server count updates. Native request registration is now serialized (one pending promotion), with regression checks and an explicit retryable rejection message. CI must be repeated on the new exact HEAD.

## Source history — specialist PR #53

## Step 108: remove Clippy test-fixture type complexity without weakening QA

**Status:** PARTIAL (test-fixture cleanup committed; fresh exact-head CI pending)  
**Declared:** 2026-11-08  
**Owner:** Agent 02 — PR #53 strict-lint follow-up.

### Exact-head evidence and cause

- [Cross-platform CI #42492183851](https://github.com/MousaXD/Slopity/actions/runs/42492183851), head `b13ca86ca95303656e767a9ca0feb2c0b6f94acf`: formatting, locked metadata and workspace tests passed, including all four new `concurrent_*` Tauri regressions. Strict Clippy failed at `apps/slopity/src-tauri/src/lib.rs:811` with `clippy::type_complexity` on the nested four-tuple return type of the **test-only** fixture helper. Linux/Android/Windows CI jobs skipped because Rust quality failed; no claims about downstream runs.
- Previous release workflow success was for the earlier head and cannot certify this SHA. [Release workflow #42492183847](https://github.com/MousaXD/Slopity/actions/runs/42492183847) may still be queued.

### Scope, risks, and non-goals

- Convert only the `#[cfg(test)]` `test_fixture()` return value from a complex tuple into a named `TestFixture` struct with identical store/orchestrator/launch evidence/temporary-directory ownership. Destructure it at its two call sites, preserving assertions and thread interleavings.
- No lint suppressions, disabled checks, mutated production lock logic, authorization changes, new runtime functionality, Android plugin changes, dependency/workflow edits, PR #46 edits, merges or direct main/dev edits.
- Intended files: `apps/slopity/src-tauri/src/lib.rs` (test-only type simplification), `PROGRESS.md`.

### Acceptance (GitHub-hosted only)

- Exact-head rustfmt, locked metadata, workspace tests and strict all-targets all-features Clippy; four named race tests must remain green.
- Linux Tauri, Android ARM64 debug, Windows NSIS, release-package/no-publish checks on revised head; independent QA re-review, coordinated integration and final integrated-head validation remain separate.
- No local or physical-device test claims.

### Implementation and evidence

- Replaced only the test fixture's nested four-tuple return with a named `TestFixture` struct and two destructuring statements. The same four references (profile store, orchestrator, recorded adapter requests and RAII temp directory) retain their types, owners, and behavior.
- No lock-order, admission, runtime adapter, assertion, error/recovery, host-service or production behavior changed, and no lint suppression added. Changed files: `apps/slopity/src-tauri/src/lib.rs`, `PROGRESS.md`.
- [CI #42492183851](https://github.com/MousaXD/Slopity/actions/runs/42492183851) proved all four named race tests pass; Clippy flagged the exact tuple. New exact-head GitHub Actions validation is **pending** at implementation commit time, so no current-head pass is claimed. No local or physical-device tests were run.
- Independent QA review and final integration-head test matrix are still required. Signing/real-device release gates remain outside scope.


## Step 107: restore strict rustfmt gate for atomic Tauri start

**Status:** PARTIAL (format-only correction committed; exact-head CI pending)  
**Declared:** 2026-11-08  
**Owner:** Agent 02 — PR #53 follow-up.

### Observed exact-head CI failure

- [Cross-platform CI #42487457518](https://github.com/MousaXD/Slopity/actions/runs/42487457518), exact implementation head `2ce76bd6e2fa945df5acf26df4d11bc9e5491997`, failed only after reaching `cargo fmt --all -- --check`. The log precisely lists formatting diffs in `apps/slopity/src-tauri/src/lib.rs` for import wrapping, profile lookup layout, stopped guard signature, test imports, fixture initialization, mutation error assertion, LAN match arm, and excess blank line. Downstream Rust tests/Clippy/Linux/Windows/Android CI skipped as a consequence; do not claim they passed or failed.
- [Release packages #42487457496](https://github.com/MousaXD/Slopity/actions/runs/42487457496) succeeded on the same patch head, with Android ARM64 and Linux .deb/AppImage jobs successful and prerelease publication skipped.

### Scope and non-goals

- Apply **only** exact formatting changes requested by rustfmt 1.89 to `apps/slopity/src-tauri/src/lib.rs`. Preserve the Step 106 mutex transaction, existing assertions, fake adapter interleavings, host-service rollback, diagnostic behavior and all runtime security logic.
- No CI/workflow/dependency changes; no weakening tests, no Android plugin edits, no PR #46 edits, no main/dev changes or merges.
- Intended files: `apps/slopity/src-tauri/src/lib.rs` and this `PROGRESS.md` entry.

### Acceptance

- GitHub-hosted `cargo fmt --all -- --check`, `cargo metadata --locked`, Rust workspace tests and strict Clippy must pass on the revised exact head, including all four threaded start/mutation regressions.
- Downstream Linux Tauri, Windows NSIS, Android ARM64 and PR-safe release-package jobs must pass (publisher skipped); independent QA #45 must re-review semantics.
- Evidence and limitations to be updated with the implementation commit. No local or device testing claims.

### Implementation and evidence

- Applied only the exact Rust 1.89 rustfmt diff from failed [job #128752351465](https://github.com/MousaXD/Slopity/actions/runs/42487457518/job/128752351465): line wraps, signature layout, and one extra blank line; no assertion, lock-order, authorization, error or adapter behavior changed.
- Changed files: `apps/slopity/src-tauri/src/lib.rs`, `PROGRESS.md` only. No local tests or platform/device checks run.
- Revised exact-head GitHub Actions results: **pending** at commit creation. Previous #42487457518 failed at rustfmt and did not exercise tests or Clippy; release workflow #42487457496 passed but does not certify the new head.
- Independent QA review and final integrated-head validation remain required before merge. No Android real-device or production release proof is claimed.


## Step 106: atomic Tauri profile-to-runtime start transaction

**Status:** PARTIAL (implementation committed; exact-head GitHub Actions and independent QA pending)  
**Declared:** 2026-11-08  
**Owner:** Agent 02 — isolated Tauri race fix, Agent 00 integration coordination.

### Scope and invariant

- Close QA issue #45's profile edit/start TOCTOU window in `apps/slopity/src-tauri/src/lib.rs`: a previously cloned profile cannot launch after an edit, disable, deletion, port change or loopback-to-LAN change has committed.
- Treat profile lookup, currently committed profile collection, authorization, and `ServerOrchestrator::start` as one transaction under a consistent **profile-store lock → orchestrator lock** order. The mutation paths already use this order; centralize their guard logic where practical.
- Take native device telemetry before the transaction; do **not** call host-service/native plugin APIs while either mutex is held. Preserve the existing host-service startup/rollback and runtime failure diagnostics.
- Test both possible linearizations: mutation before start sees the latest profile; mutation attempted during start cannot commit until start completes, then active-runtime safety rejects the mutation. Cover disable, delete, changed port and loopback→LAN scope with deterministic barriers/channels and a fake runtime adapter (no hardware/device assertions).

### Intended files

- `apps/slopity/src-tauri/src/lib.rs` (Tauri commands, private transaction helpers, scoped concurrency regressions).
- `PROGRESS.md` (this entry only).

### Non-goals

- No changes to Rust core admission permits or PR #46, Android plugin/Kotlin, CI workflow configuration, frontend behavior, supported-runtime claims, or `main`/`dev`.
- No merge, release, lockfile churn, weakening of tests/authorization, automation, or physical-device claims.

### Risks

- A long-running adapter startup under both locks may block concurrent profile mutations temporarily; it must never invert lock ordering.
- Host-service native calls occur after the runtime transaction; their own asynchronous foreground visibility/rollback risk remains assigned to Agent 04 and independent QA.
- Final integration must reconcile this Tauri-only PR with PR #46 and all other agent changes, then rerun exact-head gates.

### Acceptance checks — GitHub-hosted only

- Four named deterministic start/mutation regressions (disable, delete, port update, scope update), plus mutation-first behavior and existing host reconciliation/failed-stop assertions.
- `cargo fmt --all -- --check`; `cargo test --workspace --all-features --locked`; `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`; `cargo metadata --locked`.
- Linux Tauri compile, Windows NSIS, Android ARM64 debug packaging, security/release workflow gates; PR publication skipped. Record exact-head result links truthfully.
- Add no claims about local tests, physical Android durability or signed production builds.

### Implementation and evidence

- Implemented Tauri-only atomic start transaction: native telemetry before locks; profile lookup, current profile collection, admission permit and adapter startup within one store → orchestrator lock scope. Native host-service start, rollback and reconciliation remain outside both locks.
- Routed edit, enable/disable and delete commands through the same stopped-profile guard helper; retain durable profile commits and existing stop failure behavior.
- Added four named concurrent start/profile mutation regressions covering disabled, deleted, modified port and loopback→LAN exposure, each checking both start-wins (in-flight profile lock held, mutation rejected) and mutation-wins (latest profile honored). A no-socket fake runtime adapter records the exact authorized profile.
- Files changed: `apps/slopity/src-tauri/src/lib.rs`, `PROGRESS.md`. No Android plugin, runtime core, security checks, dependency, workflow, release or frontend files changed.
- GitHub-hosted `cargo fmt`, locked Rust workspace tests, strict Clippy, Linux Tauri, Android ARM64, Windows NSIS, security and release-package checks: **pending** at implementation commit creation. No local tests or physical-device checks were run; no CI success is claimed here.
- Outstanding: request independent Agent 06 review of this Tauri fix; coordinate Agent 00 integration alongside PR #46 and Agent 04 host-service work; validate on the final integrated SHA. Running adapter starts may temporarily hold two locks; asynchronous foreground host acknowledgement remains separately unverified.

## Source history — specialist PR #54

## Step 052: fix upstream Tauri cross-WebView IPC isolation flaw (GHSA-w28w-mhc8-qvjv)

**Status:** IMPLEMENTED — proposed change in this commit; exact-head GitHub CI/audit/package verification REQUIRED before certification.  
**Declared:** 2026-11-08  
**Owner:** Agent 05, isolated stacked branch `agent-05/tauri-2-11-6-ipc-security`, based on Agent 05 certified PR #48 exact head `78fa630b9dd87d1b717a59f35e3011332b43ca78`.

### Security evidence and risk

- Tauri public advisory [GHSA-w28w-mhc8-qvjv](https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv) (High, published 2026-09-26) affects `tauri >=2.0.0, <=2.11.5`. The local `Cargo.lock` resolves `tauri 2.11.5`. An attacker-controlled WebView with Tauri IPC access could retrieve queued channel responses belonging to another WebView due to the unscoped fetch command. **No exploit or reachable attacker-controlled second WebView has been established in Slopity.**
- The upstream advisory identifies **`tauri 2.11.6`** as patched. The official upstream `tauri-v2.11.6` workspace declares `rust-version = "1.77.2"`, compatible with Slopity's pinned Rust 1.89.0 and workspace MSRV 1.82.
- Existing [Dependabot PR #20](https://github.com/MousaXD/Slopity/pull/20) supplies a narrowly scoped lockfile patch: `tauri 2.11.5 -> 2.11.6` and registry checksum `667b20e2... -> 6fa5bacd...`. It changes no manifest or transitive package entry. Do not merge the stale bot PR itself into the integration branch.
- This upstream GitHub advisory is a **separate finding** from the seven RustSec warnings tracked under issue #52. Cargo audit success on PR #48 must not be misrepresented as proof that this patched GHSA is unnecessary.

### Planned implementation and boundaries

- Change **only the `tauri` record's `version` and `checksum` in `Cargo.lock`**, using the exact independently reviewed Dependabot #20 values; leave every other package, feature graph, Rust toolchain and npm lockfile unchanged.
- Update the supply-chain review to document this additional externally disclosed High finding, its patched disposition, GHSA vs Cargo-audit coverage limits, and the cross-WebView reachability uncertainty.
- Update this ledger in the same implementation commit, referencing the exact new source SHA as soon as known.
- No app code, workflows, Android service, UI, PolyForm license, `main`/`dev`, release publishing, bulk Dependabot merges, or glib/GTK migration. Keep original PR #48 unmodified; submit a **separate draft stacked PR targeting `fix/slopity-deps-agent05`** for Agent 00 sequencing.

### Implementation evidence (exact-head validation pending)

- Applied the single-package `Cargo.lock` change `tauri 2.11.5 -> 2.11.6` using the exact published Dependabot PR #20 registry checksum `6fa5bacdb9bbad5954af3d1bd6cf6ae9192cab1b2e270f4a07f904610b9e85f4` (old `667b20e2726d572dea2de7370da16e188eb06008faf9a92fab7cdc46791190b5`). All other lock packages and dependency relationships unchanged.
- Updated `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md` to include the additional High-severity GitHub advisory and its patched version. No Rust/TypeScript/Kotlin runtime code, workflow, manifest, npm lock, license, `main` or `dev` edits.
- The earlier green checks for Agent 05 PR #48 do **not** certify this source commit. Wait for the new draft PR exact-head GitHub Actions; do not claim success or merge before completed. The seven unrelated RustSec warnings remain open under #52.

### Acceptance gates

- GitHub-hosted exact-head Rust 1.89.0 formatting, `cargo metadata --locked`, all-features workspace tests, strict Clippy; Linux Tauri compilation; Windows NSIS; Android ARM64 APK/AAB and foreground-service manifest.
- Cargo audit (report all **seven** existing allowed warnings honestly), npm audit, GitHub dependency review and pull-request package validation (no publish).
- Inspect exact PR patch to prove no manifest/workflow/other resolved dependency changes; classify risk as patched only after exact-head CI succeeds. Do not extrapolate PR #48 historical green checks to this new source SHA.

## Step 051: remediate Tokio I/O-safety advisory without expanding Rust toolchain

**Status:** DONE (Step 051 implementation at `728fa08` verified by all required exact-head GitHub Actions; seven pre-existing advisory warnings tracked in issue #52)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 05 on `fix/slopity-deps-agent05`, following Step 050's confirmed exact-head CI.

### Finding and declared change

- The [Step 050 Cargo audit job 128106136604](https://github.com/MousaXD/Slopity/actions/runs/42279980285/job/128106136604) passed, yet its detailed output shows **eight allowed warnings** rather than zero findings. Two are `unsound`, not ordinary `vulnerability` advisories. No warning is suppressed.
- RUSTSEC-2026-0386 affects locked `tokio 1.53.1`; the RustSec patched range includes **`>=1.53.3`**. Choose exactly `1.53.3` instead of major or 1.54.0, avoiding unrelated changes and the separately documented Tokio 1.54.0 regression.
- Upstream `tokio-1.53.3/tokio/Cargo.toml` declares `rust-version = "1.71"`, compatible with the repo's actual pinned Rust 1.89.0 and declared workspace MSRV 1.82. Independently corroborated the registry checksum `ce3335fa71841cda333a58d7615b03901380ecf09d59b3296d21f8bbac0dde4e` against a public Rust project's resolved lockfile; CI must still resolve and validate bytes via Cargo.
- Scope: **only** update `Cargo.lock` Tokio 1.53.1 → 1.53.3 version/checksum; update `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md` to correct the earlier false impression of zero advisories and triage all eight original warnings with the Tokio disposition; update this entry **in the same implementation commit** with evidence and limitations.

### Non-goals and ownership

- No edits to Rust source, Tauri runtime, frontend, Android service, workflows, npm lockfile, Dependabot configuration, or PolyForm license. No command execution/download feature, no auto-merge, no signed release, no `main`/`dev` changes.
- Do **not** update `glib 0.18.5` independently: GTK/Tauri ABI and build constraints need Agent 01 + Agent 02 cross-platform validation. No blanket warning ignore or audit weakening.
- The six unmaintained transitive crates require ownership-path review, not unverified mass upgrades; document and delegate appropriately.
- If Cargo `--locked` metadata or Rust 1.89 CI fails on a checksum/dependency change, stop and record the exact failure; do not disable checks.

### Intended files

- `Cargo.lock`
- `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md`
- `PROGRESS.md` (same-implementation-commit completion evidence)

### Acceptance

- Exact-head GitHub-hosted `cargo metadata --locked`, formatting, locked workspace tests, strict Clippy on Rust 1.89, Cargo audit with Tokio RUSTSEC-2026-0386 **absent**, npm audit and dependency review.
- Linux Tauri, Android ARM64 build/manifest, Windows NSIS and release package validation all remain passing, with pull-request publication skipped.
- Accurately report any remaining unmaintained or soundness advisories; no false claim that green Cargo audit means a warning-free graph.

### Implementation and initial evidence

- Per Step 051 plan, changed **only** the Tokio `Cargo.lock` package version `1.53.1 -> 1.53.3` and checksum `202ca... -> ce3335...`; no Tokio dependency names/features, application manifests, or other crate entries were altered manually. The checksum is corroborated by publicly published resolved Cargo locks; hosted Cargo --locked checks are the acceptance authority.
- Documented the eight original allowed warnings and dispositions in `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md`; corrected the incorrect suggestion that a passing cargo audit had no informational advisories. Kept all warnings enabled; no `--ignore`.
- The original Step 050 implementation `bdfebbf` passed audit, Rust/Linux/Android/Windows CI and PR release packaging as linked above. **No local tests were executed.**
- New Step 051 exact-head CI remains **pending** at this commit; completion depends on the selected new SHA's audit (Tokio warning absent) and unchanged cross-platform/release gates.
- The `glib 0.18.5` unsoundness warning and six unmaintained dependency warnings are not fixed in this narrow compatibility-preserving update; their remediations are delegated for separate reviewed changes.

### Final exact implementation-head CI verification (2026-11-06)

- **Verified implementation SHA:** `728fa080d345650da008658820da77e2e905ee16` on draft [PR #48](https://github.com/MousaXD/Slopity/pull/48), targeting `feat/owner-integration-20261106`. The implemented lockfile and source tree were tested on this exact SHA. No local tests ran.
- [Cross-platform CI run **42284699140**](https://github.com/MousaXD/Slopity/actions/runs/42284699140): **success**. Rust 1.89.0 `cargo fmt --all -- --check`, `cargo metadata --locked --format-version 1`, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` all **passed**. Linux Tauri compilation, Android ARM64 debug APK/AAB + manifest validation, and Windows x64 NSIS build/validation all **passed**.
- [Security and dependency audit run **42284699790**](https://github.com/MousaXD/Slopity/actions/runs/42284699790): **success** for Cargo vulnerability audit, npm audit (`found 0 vulnerabilities`), and GitHub dependency review. Cargo-audit log: **7 allowed warnings**, down from 8 at Step 050; **RUSTSEC-2026-0386 (`tokio`) is absent** after pinned Tokio 1.53.3, with 6 `unmaintained` crate warnings and `glib 0.18.5` **RUSTSEC-2024-0429 (`unsound`) still present**. No warning suppression or audit configuration change.
- [Package and publish releases run **42284699112**](https://github.com/MousaXD/Slopity/actions/runs/42284699112): **success**. Linux `.deb` and AppImage bundles and Android ARM64 debug packages succeeded, including artifacts upload; GitHub prerelease publication **skipped** for PR as intended.
- Unresolved GTK/Tauri and unmaintained transitive-dependency risk documented and delegated in [issue #52](https://github.com/MousaXD/Slopity/issues/52). No merge or production-signing/physical-device certification. Any independent `main` integration merge still requires the coordinator's explicit decision and CI on the final integration SHA.
- **Closeout note:** this progress-only documentation update follows the verified implementation SHA, so it creates a **new PR head**; the latest head's required checks must also pass before the coordinator accepts the PR. Do not mislabel the above historical implementation SHA's green results as validation of this new documentation SHA.

## Step 050: Agent 05 narrowly scoped dependency and supply-chain audit

**Status:** DONE (Step 050 implementation validated at bdfebbf; follow-up advisory remediation planned separately)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06  
**Owner:** Agent 05, branch `fix/slopity-deps-agent05`, based on integration PR #43 head `31d524fba0b72a4a79ec7d6bac3b97c749d564f2`.

### Scope and baseline

- Independently review the resolved Cargo/npm lock graphs, pinned Rust compiler, GitHub Actions pins, exact-head audit jobs, 15 outstanding Dependabot PRs, executable/package trust boundaries, and reproducibility limitations.
- Review [integration audit 42275322860](https://github.com/MousaXD/Slopity/actions/runs/42275322860) and [cross-platform CI 42275322969](https://github.com/MousaXD/Slopity/actions/runs/42275322969), both successful on the integration commit; release-package run 42275322783 was pending at initial inspection.
- Apply exactly one reviewable, low-risk Rust lockfile patch: `serde_json 1.0.151 -> 1.0.154` with the registry checksum and unchanged dependency relationships from [Dependabot PR #29](https://github.com/MousaXD/Slopity/pull/29). Keep the existing semver-compatible workspace manifest; no unrelated `cargo update`.
- Align `.github/dependabot.yml` target branches to supported `main`, which is the destination of integration PR #43 and the security-supported trunk defined in `SECURITY.md`; do not retarget existing bot PRs in this step.
- Write `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md` with per-PR disposition, threat-model recommendations for *future* packages, action/provenance review, and unresolved risks. Cross-reference it from `docs/SECURITY_MODEL.md`.

### Non-goals, coordination, risks

- Never modify `.github/workflows/**` (Agent 01); do not change frontend dependencies (Agent 03), runtime execution (Agent 02), or Android code (Agent 04).
- No package download/extraction/execution feature, no runtime support claim, no `main`/`dev` edits, no merge, no auto-merge, no release, and no PolyForm license change.
- The Cargo 1.89.0 pin is an application test contract; do not raise it to satisfy a speculative dependency. Verify the updated lockfile with `--locked`, and revert on confirmed incompatibility.
- GitHub Actions uses moving hosted images and scanner toolchain, so passing audits are time- and commit-scoped rather than permanent supply-chain certification.
- Existing Dependabot action-major PRs were authored against older workflow trees; avoid bulk merging and coordinate any future action pin updates with Agent 01.

### Intended files

- `Cargo.lock`
- `.github/dependabot.yml`
- `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md` (new)
- `docs/SECURITY_MODEL.md`
- `PROGRESS.md` (implementation, checks, results, and honest completion state **in the same implementation commit**)

### Acceptance checks (GitHub-hosted; record exact head)

- Rust formatting; `cargo metadata --locked`; `cargo test --workspace --all-features --locked`; strict all-targets Clippy with `--locked`, using Rust 1.89.0.
- Cargo vulnerability audit, `npm audit`, GitHub dependency review, frontend checks, Linux Tauri, Android ARM64 build/manifest, Windows NSIS, and non-publishing PR release-package validation.
- Verify only the intended lockfile package/checksum and `main` Dependabot targets change. No physical Android device, production signing, or provenance claim from CI alone.


### Implementation and evidence at implementation commit

- Inspected all 15 open Dependabot PR patches, integration manifests/lockfile, pinned workflows, release permission model and untrusted executable boundary. Wrote the per-PR triage and future package verification gates in `docs/DEPENDENCY_SECURITY_REVIEW_2026-11-06.md`.
- Changed exactly the `serde_json` package's version/checksum in `Cargo.lock` from 1.0.151 to 1.0.154, using the independently supplied checksum in #29; no additional package graph entry or manifest range changed.
- Changed exactly three Dependabot `target-branch` fields `dev -> main`; aligned `docs/SECURITY_MODEL.md` with built-in HTTP runtime support and unverified Android hosting.
- Remote baseline (before modification): [Rust / Linux / Android / Windows green](https://github.com/MousaXD/Slopity/actions/runs/42275322969), [Cargo/npm/dependency-review green](https://github.com/MousaXD/Slopity/actions/runs/42275322860). These validate only original integration head `31d524f`.
- Static checks performed in GitHub connector: exact old serde_json version/checksum present exactly once; substitution makes only requested package metadata change; three `dev` target branches found; no workflow/source/licensing files written. **No local tests run.**
- **Exact implementation-head CI: pending until draft PR is opened.** Do not mark this step DONE, close Dependabot #29, or merge before independently checking all new SHA jobs. Release-packaging status to be rechecked and cited separately.
- Remaining action, CLI, Rust Tauri and package-management risks are delegated to their owning agents, with no implied approval or suppression.

### Post-implementation verification (2026-11-06)

- Exact implementation-head `bdfebbf8c14fc0cfb9c70b9d0fb5320d780dc198`: [cross-platform CI 42279980309](https://github.com/MousaXD/Slopity/actions/runs/42279980309) **success** (Rust 1.89.0 fmt, locked metadata/tests, strict Clippy, Linux Tauri, Android ARM64 debug + manifest, Windows NSIS); [audit 42279980285](https://github.com/MousaXD/Slopity/actions/runs/42279980285) **success** (Cargo audit, npm audit, dependency review); [release packaging 42279980270](https://github.com/MousaXD/Slopity/actions/runs/42279980270) **success** (Linux .deb/AppImage and Android debug APK/AAB); GitHub prerelease publishing was **skipped** as expected.
- **Factual correction:** a successful Cargo audit does not mean no advisories. Its detailed log reported **8 allowed warnings**: six **unmaintained** dependencies (`proc-macro-error`, `unic-char-property`, `unic-char-range`, `unic-common`, `unic-ucd-ident`, `unic-ucd-version`) and two **unsound** dependencies (`glib 0.18.5` RUSTSEC-2024-0429; `tokio 1.53.1` RUSTSEC-2026-0386). No vulnerability-class failures were reported, but these warnings require transparent triage. Step 051 handles the patched Tokio advisory; glib and unmaintained transitive crates remain explicit review risks.
- This post-checkpoint documentation does not change the evidence for `bdfebbf`; new commits require their own exact-head CI. No merge authorized.

## Step 033: owner-controlled cross-workstream integration candidate

**Status:** PARTIAL (combined source committed; exact-head GitHub Actions pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06

### Mission and baseline

- Owner-authorized convergence of the existing Slopity Rust/Tauri, GUI, CI, and Android work into one branch for eventual promotion to `main` **only after exact-head CI and independent review**.
- Start from `fix/runtime-admission-rustfmt-ledger-20261106` at `9c061c6d4a3dc9a2a43fb15c43d988ee846cbeb7`: it contains the recovered CI #35 and the runtime safety/Clippy remediation #38/#42. Cross-platform CI [42140594899](https://github.com/MousaXD/Slopity/actions/runs/42140594899) and release-package validation [42140594848](https://github.com/MousaXD/Slopity/actions/runs/42140594848) passed on that exact SHA.
- GUI V2 [PR #37](https://github.com/MousaXD/Slopity/pull/37) on SHA `b6a5b30ad8682a54abf476b33f8f59f0852d1995` passed [cross-platform CI](https://github.com/MousaXD/Slopity/actions/runs/42082159333), [security audit](https://github.com/MousaXD/Slopity/actions/runs/42082159369), and [release package validation](https://github.com/MousaXD/Slopity/actions/runs/42082159284). These are historical separate-branch passes, **not** combined verification.
- Android durability PR #12 is only partially supported by hosted CI; physical Android survival remains an explicit release limitation.
- Earlier draft PRs #9/#10/#11/#12/#35-#42 remain intact until an audited integration decision. Do not bulk merge conflicting PRs or Dependabot updates.

### Workstream ownership (six roles + coordinator, not independently launched agents)

1. CI/release: preserve locked Rust, Linux, Android, Windows, security and no-publish release gates.
2. Rust/lifecycle: preserve fail-closed admission and stop-failure diagnostics, evaluate original #40/#41 diffs against corrected #38/#42.
3. GUI/UX: integrate all modular GUI V2 frontend functionality and tests, check keyboard, responsive behavior, truthful unsupported runtimes.
4. Android: integrate foreground service, durable status reconciliation and documentation from #12, but require real-device proof separately.
5. Security/dependencies: review 15 Dependabot PRs as independent locked batches after a green baseline, preserve license and pinned actions.
6. Independent QA: compare source inventories, exact-head tests, regression gates and release claims; block merges on real failures.

Coordinator controls the only integration branch, workstream conflicts and final promotion.

### Scope

- Incorporate all existing GUI V2 changed frontend files from PR #37, preserving its frontend format/lint/test gates.
- Reconcile Android durability source changes without overwriting modern Rust admission behavior. If conflict resolution cannot be verified, leave it isolated and document it as blocked; do not fake readiness.
- Preserve the original historical Step 021 and other workstream ledger entries, and reconcile CI evidence from PR #39.
- Create a single draft integration PR to `main`, initially no merge, so GitHub Actions can validate the actual combined tree.
- Only after all required exact-head CI passes, security review, and known platform limitations are truthfully documented may the owner consider a merge. No signing secrets, forced pushes, or premature release.

### Non-goals and security boundaries

- No external runtime presented as operational without real platform proof; only built-in HTTP is registered.
- No arbitrary profile shell execution, relaxed resource admission, fake device telemetry, disabled tests, global lint exemptions, license changes, package downloads, or direct `main` edits.
- No assertion that GitHub cross-compilation proves background Android durability or signed production releases.

### Risks

- Android PR #12 and Rust runtime PR #38 both touch Tauri `src/lib.rs`; manual semantic reconciliation and tests are mandatory.
- GUI V2 assumes Tauri command shape and admission rejection JSON; combined build is necessary but insufficient for manual interaction proof.
- CI jobs may need fresh reruns for each implementation SHA. A previously green parent is not approval of this integration.

### Intended files

- `PROGRESS.md`, `docs/ENGINEERING_STATUS_2026-11-05.md` and integration QA references.
- `apps/slopity/package.json`, `apps/slopity/scripts/**` and `apps/slopity/web/**` from PR #37.
- Reviewed Android `plugins/**`, `apps/slopity/src-tauri/src/lib.rs`, `docs/android-device-durability.md`, `scripts/android-durability-probe.sh`.

### Implementation submitted for exact-head CI

- Integrated **all 23 GUI V2 frontend paths** as exact blobs from PR #37 on its verified SHA, including HTML/CSS, modular views, frontend tests, lint and format scripts.
- Integrated **all 6 non-conflicting Android plugin/probe/documentation paths** from PR #12 as exact blobs, and semantically reconciled its Tauri host lifecycle and test functions with Rust admission/error handling and failure-evidence UI snapshots from PR #42. The `StartAdmissionRejection` checks and JSON error kinds are retained.
- Included the original Step 021 GUI and Step 024 Android planning entries, and the CI verification notes from PR #39. No claims of physical-device QA or signed production readiness.
- GitHub-hosted exact-head CI is **pending**. No tests were run in this chat. No merge to `main`, no release, no dependency bulk-update.

### Acceptance checks — GitHub-hosted only

- `cargo fmt --all -- --check`; `cargo metadata --locked --format-version 1`; `cargo test --workspace --all-features --locked`; strict all-targets/all-features Clippy.
- Frontend format, lint and Node tests, Tauri Linux compile, Android ARM64 debug manifest/artifacts, Windows NSIS, Linux and Android release-package validation with PR publication skipped.
- Cargo vulnerability audit, npm audit, GitHub dependency review and any new platform/contract regressions.
- Independent source diff audit of all included slices, documentation accuracy and honest Android real-device caveat.
- Use exact SHA/run URLs in this ledger. Until complete, mark PARTIAL and keep the integration PR in draft.

## Step 032: repair runtime admission formatting gate

**Status:** PARTIAL (minimal formatting fix committed; exact-head GitHub Actions pending)  
**Declared:** 2026-11-06  
**Updated:** 2026-11-06

### Evidence and scope

- [PR #38 CI run 42083372641](https://github.com/MousaXD/Slopity/actions/runs/42083372641) failed the Rust quality job at `cargo fmt --all -- --check` before tests and Clippy.
- The exact diff in `crates/slopity-core/src/admission.rs` is a rustfmt-only line wrap of `serde_json::to_value(&rejection).expect(...)`.
- Correct only that formatting; retain admission rejection diagnostics, assertions, serialization shape, and fail-closed policy.
- Intended files: `crates/slopity-core/src/admission.rs`, `PROGRESS.md`.
- Non-goals: runtime policy changes, warning suppression, skipped tests, Android device claims, workflow changes, merges or releases.
- Acceptance: exact-head GitHub-hosted Rust format, locked tests, strict Clippy, Linux/Android/Windows and release checks; record any failures honestly. No local tests will be claimed.

### Implementation and evidence

- Applied only the rustfmt-required line wrap in `crates/slopity-core/src/admission.rs`; no admission semantics, JSON assertions, or policy changes.
- No local tests or physical-device tests were run. GitHub Actions exact-head results are pending; the original PR #38 Rust gate failure is not evidence of downstream test results.
- Follow-up: inspect exact-head Rust, platform and release results; reconcile this patch with PR #38 only after review.


## Step 030: reduce admission error footprint without weakening checks

**Status:** PARTIAL (layout change and JSON regression assertion committed; exact-head CI pending)  
**Declared:** 2026-11-05  
**Updated:** 2026-11-05

### Confirmed CI finding

- Focused runtime fix PR #36 exact-head [Cross-platform CI 42081473190](https://github.com/MousaXD/Slopity/actions/runs/42081473190) passed Rust format and tests, but strict Clippy failed at `crates/slopity-core/src/admission.rs:59`: `clippy::result_large_err`. The `StartAdmissionRejection` `Err` variant is at least 184 bytes.
- The same admission interface has been copied onto integrated runtime PR #38. Fix the in-memory payload layout rather than silencing the lint or suppressing admission errors.

### Scope

1. Box the large `allocation` field inside `StartAdmissionRejection`; keep the existing `serde` JSON shape, all reasons/issues/warnings, and every safety rejection.
2. Retain `authorize_start`'s return type and all test assertions; no warning suppression or admission-policy weakening.
3. Verify in exact-head CI that strict Clippy, Rust tests, platform builds and audit gates pass.

### Non-goals

- No change to runtime features, executable boundaries, UI/API JSON keys, OS platform targets, or release signing.
- No direct `main`/`dev` modification or premature merge.

### Intended files

- `PROGRESS.md`
- `crates/slopity-core/src/admission.rs`

### Implementation and evidence

- Updated `StartAdmissionRejection.allocation` to `Box<AllocationDecision>`; constructing rejections now boxes the full unchanged allocation report.
- Added assertions on the serialized rejection to protect JSON keys `allocation`, `warnings`, `validationIssues`, and `reasons`; no fields or decision rules removed.
- Did not run any local tests; exact-head CI must confirm the error footprint passes Clippy and app/serialization tests pass. Pending result.

### Acceptance checks

- GitHub-hosted Rust fmt, locked tests, strict Clippy `-D warnings` on exact head.
- Existing admission rejection tests and Tauri serialization/compile continue to pass; no loss of error diagnostics.
- Linux/Android/Windows and package/audit checks remain enabled, with authentic results recorded.

## Step 029: reconcile Rust runtime safety with the current CI foundation

**Status:** PARTIAL (implementation committed; exact-head GitHub Actions pending)  
**Declared:** 2026-11-05  
**Updated:** 2026-11-05

### Scope and sources

1. Integrate the seven actual backend/source/ADR changed paths from [original runtime hardening draft #10](https://github.com/MousaXD/Slopity/pull/10) into the repaired CI foundation, without overwriting new workflow fixes from #35.
2. Import the corrected orchestrator implementation (including stop-failure evidence retention and regression tests) from [draft fix #36](https://github.com/MousaXD/Slopity/pull/36), not PR #10's known-failing version.
3. Preserve original Step 023 runtime admission/lifecycle planning in `PROGRESS.md`, alongside current Step 025/028 CI updates and this new Step 029.
4. Keep launch authority behind opaque `StartAdmissionPermit`, port/memory validation, and fail-closed stop/observed-state behavior. Keep the built-in HTTP runtime the only registered provider.
5. Validate on GitHub-hosted CI using Rust 1.89, Linux Tauri, Android ARM64, Windows NSIS, security review and release package validation. Do not manufacture results.

### Non-goals

- No new external runtime support, code-download capabilities, automatic restart behavior, shell command execution, release publication, or direct updates to `main`/`dev`.
- No replacement or closure of original PR #10 or focused fix PR #36 without proof.
- No physical-device Android durability claim.

### Risks and acceptance criteria

- The new Tauri admission error API may require GUI V2 integration; review cross-branch behavior on the next combined GUI integration.
- The failure evidence regression must remain covered by the **unchanged existing assertion** plus the new repeated-refresh/terminal test.
- Preserve PR #35 audit toolchain and Android SDK setup fixes, and do not replace its CI workflow with stale PR #10 workflow files.
- Do not claim full validation until exact-head GitHub Actions pass and their run/job links appear here.

### Intended files

- `PROGRESS.md`
- `apps/slopity/src-tauri/src/lib.rs`
- `crates/slopity-core/src/{admission.rs,lib.rs,orchestrator.rs}`
- `crates/slopity-core/tests/orchestrator_foundation.rs`
- `crates/slopity-runtime-http/src/lib.rs`
- `docs/adr/0002-runtime-intent-recovery.md`

### Implementation (validation pending)

- Copied the seven precise runtime-source and ADR blobs from the known PR #10 source tree, using the independently corrected `orchestrator.rs` blob from PR #36 head `4dd75293ad4c8669cb7e35bb4e66ff389ed2045f`.
- Preserved the repaired workflow and audit files from PR #35 instead of importing stale workflow copies.
- Inserted original Step 023 historical plan without replacing Steps 025 or 028.
- No application tests were run in this chat. GitHub-hosted exact-head CI is the only acceptance evidence and is still **pending**.

### Checks

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- Linux/Android/Windows CI, security audit, PR release-package builds, unchanged safeguards and honest platform claims.

## Step 028: remove retired Android SDK tools package from hosted builds

**Status:** PARTIAL (CI change committed; remote build evidence pending)  
**Declared:** 2026-11-05  
**Updated:** 2026-11-05

### Finding

- On [PR #35 release validation run 42080026430](https://github.com/MousaXD/Slopity/actions/runs/42080026430), the Android packaging job failed in `android-actions/setup-android` before application build.
- Exact [failing job 127430419954](https://github.com/MousaXD/Slopity/actions/runs/42080026430/job/127430419954) executed `sdkmanager tools` and exited with `Warning: Failed to find package 'tools'`.
- The pinned `android-actions/setup-android@40fd30fb8d7440372e1316f5d1809ec01dcd3699` (v4.0.1) defines default `packages: 'tools platform-tools'`. The legacy `tools` package has been removed from the remote SDK repository.
- The same action is used by cross-platform `ci.yml` and release `release.yml`; both should provide the explicitly supported `packages: 'platform-tools'` input. Existing explicit `sdkmanager` installation of API 36, build-tools 36 and NDK 27.3 must remain unchanged.

### Scope

1. Override the pinned action's obsolete default `packages` in both CI and PR release-package workflows, without changing action pinning, signing policy, Android platform/NDK versions or required artifact assertions.
2. Preserve `PROGRESS.md` evidence and rerun GitHub-hosted jobs on the exact new PR head.

### Non-goals

- Removing Android builds, weakening manifest checks, pretending device durability was tested, adding SDK credentials, or changing runtime behavior.
- Unreviewed Dependabot version upgrades or changes to `main`/`dev`.

### Intended files

- `PROGRESS.md`
- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`

### Implementation and verification at implementation commit

- Explicitly passed `packages: 'platform-tools'` to the pinned Android SDK setup action in both cross-platform CI and release-package Android jobs, overriding the obsolete `tools platform-tools` default without changing toolchain/package targets.
- No local or chat-hosted build was run. **Exact-head Android and platform CI checks are pending**; update this entry with run results and investigate any subsequent errors.

### Acceptance checks

- Exact-head Android CI and package jobs advance past `Set up Android SDK` to SDK/NDK installation and ultimately pass build, manifest and artifact assertions.
- Existing Rust/Linux/Windows/release gates remain intact; actual results linked when available.

## Step 025: restore reproducible security checks and establish November engineering triage

**Status:** PARTIAL (implementation committed; exact-head GitHub Actions results pending)  
**Declared:** 2026-11-05  
**Updated:** 2026-11-05

### Baseline and reason

- Integration base: `fix/ci-release-foundation` at `8825f1e0e997c93730cab11caaa638fe4205a598` (draft PR #11 targeting `dev`); it previously passed cross-platform CI, release-packaging and security-audit workflows on 2026-08-24.
- Default `main` / `dev`: `f57b7cee18dbf73b06eb64aa0c12212c4ca9333d`. Its scheduled security run 41467880911 on 2026-11-02 failed *before an audit* because an un-locked cargo-audit install resolved `kstring 2.0.5` (MSRV Rust 1.96), while the runner used Rust 1.89. Do not report this as a vulnerability finding or a clean security result.
- Open draft GUI PR #9, runtime-safety PR #10, Android durability PR #12 and CI PR #11 must be reconciled without silently losing features. PR #10's run 33215426044 failed in `orchestrator::tests::failed_stop_preserves_potentially_live_runtime`, with `snapshot.last_error.is_some()` false.
- As of 2026-11-05 there are 15 open Dependabot PRs (and four draft human engineering PRs). Do not bulk merge unvalidated updates.

### Scope

1. Preserve PR #11's proven Linux, Android, Windows, Rust, audit and package CI foundation; repair the audit-toolchain MSRV failure without weakening vulnerability detection or changing the application's pinned Rust compiler.
2. Add a durable, source-linked engineering-status and workstream document under `docs/` with known build/feature/physical-device evidence boundaries, integration order, five proposed specialist roles, and acceptance gates.
3. Validate *this exact head* through GitHub-hosted Actions. Investigate failures in job logs and iteratively fix true blockers. Do not equate mocked tests with runtime support.
4. Keep all pre-existing GUI, Android and runtime hardening PRs intact and unmerged pending their own exact-head green validation. Publish this slice as a draft PR targeting `dev`.

### Non-goals

- No direct writes to `main` or `dev`, no releases, no privileged signing or secrets, no force pushes.
- No claim that Java/Minecraft/Node/Python/PHP or arbitrary-executable providers are operational.
- No claim of physical-device Android hosting durability or production-ready Windows runtime from CI compilation alone.
- No global loosening of Rust warnings, vulnerability fail gates, test assertions, or release permissions.
- No implicit license changes; PolyForm Noncommercial remains source-available rather than OSI-approved open source.

### Risks

- Using a newer toolchain for the *audit utility* must not mask application MSRV/build issues: Rust workspace checks remain on pinned 1.89.
- `cargo-audit` may reveal actual vulnerabilities after installing successfully; fail closed and remediate separately.
- GitHub Actions behavior can change since the 2026-08-24 passing runs; exact commit evidence is required.
- Cross-branch feature integration could overwrite shared `PROGRESS.md` or silently discard work. Integration must be diff-reviewed and retested.

### Intended files

- `PROGRESS.md` (plan, then exact CI evidence and status)
- `.github/workflows/audit.yml` (tool installation/runtime isolation only)
- `docs/ENGINEERING_STATUS_2026-11-05.md` (five workstreams, CI evidence and phased integration plan)

### Implementation and verification recorded at implementation commit

- Isolated the `cargo-audit` installer from the application's Rust 1.89 toolchain: the audit job now installs a current stable toolchain solely for audit tooling, retains the pinned `cargo-audit 0.22.2 --locked` install, and keeps the actual vulnerability scan mandatory.
- Preserved the existing Rust workspace CI compiler at 1.89.0 and all build/Clippy/test requirements.
- Added `docs/ENGINEERING_STATUS_2026-11-05.md` with exact baseline evidence, prioritized integration and five non-running proposed ownership lanes.
- Execution: **GitHub-hosted validation is pending for this implementation commit**. No tests were run locally or in this chat. Read exact-head workflow results before changing status to DONE; fix any real findings instead of suppressing gates.
- Follow-up: reconcile PRs #9/#10/#11/#12 after this baseline, address PR #10's actual failed-stop evidence regression and validate GUI V2 through real CI.

### Acceptance checks (GitHub Actions only)

- Audit job actually installs `cargo-audit`, downloads advisory data, analyzes `Cargo.lock`, and fails for genuine vulnerabilities.
- `npm audit` and dependency review remain enabled, with honest outcomes.
- `cargo fmt --all -- --check`, `cargo test --workspace --all-features --locked`, and strict all-targets Clippy run using the project-pinned compiler.
- Linux Tauri compile, Android ARM64 debug/manifest/artifact, Windows x64 NSIS build and release packaging validate this exact head, or are documented as blocked with logs.
- Keep the branch/PR in draft if checks are pending or failing; no unverified merge.

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

## Step 023: harden runtime admission and lifecycle semantics

**Status:** IN PROGRESS  
**Declared:** 2026-08-25  
**Updated:** 2026-08-25

### Scope

1. Add a platform-neutral start-admission decision in `slopity-core` that combines profile validation, enabled state, verified runtime availability, aggregate port reservations, safe memory budget, immediate memory headroom, and existing CPU-headroom warnings before a runtime adapter can start.
2. Require an admission permit at the `ServerOrchestrator` start boundary so callers cannot accidentally bypass hard safety checks, while keeping runtime adapters responsible for their own last-mile bind/process errors.
3. Harden lifecycle semantics for startup, stop failures, runtime observations, unexpected terminal exits, event ordering, terminal evidence, and drop-time cleanup without hiding a potentially live runtime behind a false terminal state.
4. Add bounded structured unexpected-failure evidence suitable for future retry/backoff policy, with no automatic restart behavior in this step.
5. Define a versioned desired-state persistence contract and recovery design without persisting process IDs or treating stale observed state as authoritative; defer on-disk lifecycle restore until restart policy is explicitly designed.
6. Strengthen the built-in HTTP reference-runtime tests for disabled profiles, loopback/LAN binding, request counting, immediate port reuse, unexpected thread exit, event order, and admission rejection before adapter start.
7. Enforce admission at the Tauri start command and return structured rejection data so frontend code renders backend policy instead of recreating it.

### Non-goals

- Implementing or downloading Java/JVM, Minecraft, Jellyfin, Node.js, Python, PHP, native packages, scripts, or any general arbitrary executable runtime.
- Registering or exposing `slopity-runtime-local` through Tauri or the frontend.
- Adding shell command strings, `sh -c`, PowerShell command strings, or `cmd /C`.
- Automatically restarting failed servers or introducing retry/backoff timing policy.
- Persisting observed runtime state or process IDs as proof that a process survived an application restart.
- Owning the concurrent CI/release repair work; this branch will consume Agent 1's fixes later if they become available without overwriting them.

### Risks

- Missing mandatory memory telemetry already causes the conservative resource model to fail closed; start-time enforcement will make that existing policy operational rather than advisory.
- A synchronous adapter that reports an active state after `stop` may still own live resources. The orchestrator must preserve that active observation so later cleanup can retry instead of masking it as terminal failure.
- Existing GitHub Actions currently fail during Rust toolchain setup before repository checks run. Remote validation must remain truthful until the CI foundation branch repairs that failure.
- The frontend currently expects string errors from the start command; structured admission rejection is an intentional API change that GUI work must consume.

### Intended files

- `PROGRESS.md`
- `crates/slopity-core/src/admission.rs`
- `crates/slopity-core/src/lib.rs`
- `crates/slopity-core/src/orchestrator.rs`
- `crates/slopity-runtime-http/src/lib.rs`
- `apps/slopity/src-tauri/src/lib.rs`
- `docs/adr/0002-runtime-intent-recovery.md`
- runtime/core documentation only if factual behavior changes require it

### Acceptance checks

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
cargo metadata --locked --format-version 1
cd apps/slopity && npm ci && npm run tauri:check
cd apps/slopity && npm run android:init -- --ci
cd apps/slopity && npm run android:build -- --debug --target aarch64
```

GitHub Actions remains the authoritative remote proof. Exact run/job conclusions and any infrastructure blockage will be recorded before this step is closed.

## Step 021: overhaul the shared server control center GUI

**Status:** IN PROGRESS  
**Declared:** 2026-08-24  
**Updated:** 2026-08-24

### Scope

1. Replace the current mobile-card-oriented presentation with a production-quality local server control center that has a true desktop shell and first-class mobile layout while preserving the existing Tauri command surface and backend behavior.
2. Surface real host capability, resource accounting, recovery notices, runtime availability, server lifecycle state, network configuration, and failures with clearer hierarchy and direct lifecycle controls.
3. Redesign Add Server, profile editing, server details, destructive confirmation, error/loading states, keyboard/focus behavior, and responsive behavior without adding a frontend framework or remote dependencies.
4. Keep unsupported runtime templates truthful and unavailable; only the currently registered built-in HTTP provider may expose a runnable lifecycle path.
5. Perform rendered browser QA across the required viewport matrix and screenshot-review the 390x844, 768x1024, and 1440x900 layouts before closing the step.

### Non-goals

- No Minecraft, Java/JVM, Jellyfin, Node.js, Python, PHP, generic runtime, package/download, plugin/mod, GitHub deployment, or release-pipeline implementation.
- No changes to `.github/workflows/**`, Cargo/package version metadata, runtime registration, `ServerOrchestrator`, `crates/slopity-core/**`, runtime crates, or plugins.
- No frontend framework migration and no new UI/icon dependency.
- No claims of physical Android durability or runtime support beyond existing backend evidence.

### Risks

- Lifecycle and profile actions must remain wired exactly once to the existing Tauri commands.
- Long names, runtime strings, URLs, IPv6-style values, diagnostics, keyboard opening, and safe areas must not create horizontal overflow or hidden controls.
- Dialog focus containment/restoration and destructive/unsaved-change confirmation must remain keyboard and screen-reader operable.
- `PROGRESS.md` may overlap with concurrent CI/runtime work, so this entry stays isolated at the top and application changes stay inside `apps/slopity/web/**`.

### Intended files

- `PROGRESS.md`
- `apps/slopity/web/index.html`
- `apps/slopity/web/styles.css`
- `apps/slopity/web/app.js`

### Acceptance checks

- `node --check apps/slopity/web/app.js`
- source scan for unsafe DOM injection, inline handlers, remote assets, and disabled zoom
- rendered local browser QA at 320x568, 360x800, 390x844, 768x1024, 1024x768, 1280x800, 1440x900, and 1920x1080
- screenshot review at 390x844, 768x1024, and 1440x900
- keyboard/focus/dialog interaction checks and create/edit/clone/enable/delete/start/stop/refresh mock workflow checks
- `cargo fmt --all -- --check`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cd apps/slopity && npm ci && npm audit && npm run tauri:check`

## Step 024: harden Android host durability and device validation

**Status:** PARTIAL  
**Declared:** 2026-08-25  
**Updated:** 2026-08-25

### Scope

1. Harden the Android foreground-service lifecycle so start/update/stop requests are idempotent, notification delivery constraints are reported conservatively, native stop requests are reconciled by the Rust/Tauri layer, and host-service failures cannot silently leave newly started Rust hosting without required foreground state.
2. Reconcile observed Rust server state with native host-service state on fresh dashboard/list observations, including multiple-server counts, final-server stop, notification stop requests, service loss, runtime exits, and failed foreground-service restarts.
3. Harden Android telemetry collection so missing/unknown values remain unavailable, individual Android API failures do not crash telemetry collection, and Rust-side normalization remains conservative for resource planning.
4. Add Rust-side tests for host reconciliation decisions and telemetry normalization where the Android framework itself is not available to workspace tests.
5. Add a reproducible physical Android ARM64 durability procedure and evidence template, plus a host-side ADB/curl probe helper for multi-hour reachability, battery, temperature, thermal, storage, notification, state-reconciliation, and port-release evidence.
6. Document OEM validation expectations for stock/AOSP-ish Android, Samsung, and Xiaomi/Poco without adding vendor-specific bypasses or background-execution hacks.
7. Use existing GitHub Actions as the remote proof for Rust quality, Linux/Tauri compile, Android ARM64 build, merged manifest verification, and APK/AAB artifact generation. Inspect actual run/job results before closing the step.

### Non-goals

- Claiming production-ready or durable Android hosting without physical-device evidence.
- Adding Minecraft, Jellyfin, Java/JVM downloads, Node.js, Python, PHP, generic native execution, arbitrary commands, runtime downloads, or GUI V2 work.
- Bypassing Android background execution restrictions, hiding foreground hosting, suppressing required user visibility, adding boot receivers, or implementing vendor-specific battery-killer hacks.
- Broad `.github/workflows/**`, `apps/slopity/web/**`, or `slopity-core` redesigns.
- Merging this branch, rebasing shared branches, force-pushing, or modifying `main`/`dev` directly.

### Risks

- Android foreground-service startup is asynchronous; a successful `startForegroundService` request is not itself proof that the service entered foreground state.
- A notification stop action cannot directly mutate Rust orchestrator state without crossing the Tauri/native boundary, so safe stop semantics require reconciliation when the application regains control.
- Runtime crashes while the UI is fully backgrounded may not be observed by the process-local orchestrator until its next observation pass; documentation must not imply proactive crash reconciliation that is not implemented.
- OEM process-management behavior varies and cannot be proven by CI/emulator compilation.
- Android 13+ notification permission and app/channel notification settings can make a foreground service less visible even when Android permits it to run; Slopity must fail conservatively rather than treating hidden hosting as equivalent to visible hosting.

### Delivered

- Extended native/Rust host status with pending-start state plus app-level and channel-level notification-delivery gates, while retaining explicit notification permission state.
- Made Android host start failures reject cleanly: immediate `startForegroundService` exceptions clear pending state, foreground-promotion exceptions clear service state, and Rust refuses to start/continue Android hosting when notification permission, app notifications, or the hosting channel cannot provide the visible notification Slopity requires.
- Kept repeated start/update and stop operations idempotent while reporting asynchronous start-pending state instead of fabricating an active service.
- Changed the persistent notification action to `Stop safely`: it records a native stop request and opens Slopity so Rust listeners can be stopped before the foreground service is removed.
- Added Tauri/Rust reconciliation decisions for stale host-with-no-server, missing host-with-active-server, multi-server count mismatch, native stop requests, and hidden-notification safety failures. Missing-host restart failure now triggers fail-safe attempts to stop active listeners.
- Reconciled fresh dashboard snapshots, explicit host-status requests, and built-in-server list observations before returning state. Runtime exits observed during those passes stop/update stale foreground state; notification count updates retry on subsequent observations if an already-active service update fails.
- Preserved first/additional-server rollback semantics: a failed host-service start/update rolls back the newly started runtime, then reconciles remaining previously active servers.
- Hardened Android telemetry calls with per-source exception isolation. Unknown battery charging state and unknown thermal enums are now omitted rather than converted into false values or synthetic measurements.
- Added Rust-side conservative telemetry normalization for impossible zero total memory, available-memory values above total, invalid battery percentages, non-finite temperatures, and empty thermal labels.
- Added Rust tests for notification-delivery gates, telemetry normalization, and host reconciliation decisions.
- Added `docs/android-device-durability.md` with a real-device ARM64 procedure covering install, loopback/LAN reachability, foreground notification, background/screen-off behavior, multi-server counts, UI reconciliation, UI/notification stop paths, port release, battery/temperature/thermal evidence, structured evidence fields, and OEM expectations for stock/AOSP-ish, Samsung, and Xiaomi/Poco.
- Added `scripts/android-durability-probe.sh` to record device identity, install an exact debug APK, periodically probe reachability, and retain raw ADB battery/thermal/memory/storage evidence without fabricating unavailable values.
- Left the existing Android `specialUse` manifest and CI workflow unchanged; there is no new hidden service, boot receiver, unnecessary permission, vendor bypass, runtime provider, or arbitrary execution path.

### Files changed

- `PROGRESS.md`
- `plugins/tauri-plugin-slopity-host/src/lib.rs`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostForegroundService.kt`
- `plugins/tauri-plugin-slopity-host/android/src/main/java/com/slopity/host/HostPlugin.kt`
- `plugins/tauri-plugin-slopity-host/android/README.md`
- `apps/slopity/src-tauri/src/lib.rs`
- `docs/android-device-durability.md`
- `scripts/android-durability-probe.sh`

### Verification performed

- Read-first and live-state audit completed against `dev`/`main` at `f57b7cee18dbf73b06eb64aa0c12212c4ca9333d`; both refs were identical when the branch was created.
- Current GUI V2 PR #9 changes only `PROGRESS.md`, so it does not overlap the Android native/Rust files in this step.
- No `fix/ci-release-foundation` Agent 1 branch was available when implementation began.
- Existing manifest already declares `POST_NOTIFICATIONS`, `FOREGROUND_SERVICE`, `FOREGROUND_SERVICE_SPECIAL_USE`, a non-exported `HostForegroundService`, `foregroundServiceType="specialUse"`, and the Android 14 special-use subtype property; this step intentionally preserves that standards-compliant declaration.
- Android platform documentation was checked for Android 13 notification permission semantics, Android 12+ foreground-service start restrictions, and Android 14+ foreground-service type/permission requirements before changing lifecycle behavior.
- This execution environment has no local Rust/Android toolchain or network-capable checkout, so no local compile result is being claimed. The repository's declared GitHub Actions path is the validation environment for the implementation commit.

### Verification pending

- Pull-request CI: `cargo fmt --all -- --check`.
- Pull-request CI: `cargo test --workspace --all-features --locked`.
- Pull-request CI: `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`.
- Linux Tauri compile in the existing CI chain.
- Android ARM64 debug build, merged-manifest verification, and APK/AAB artifact upload in the existing CI chain.
- Physical ARM64 device durability execution. The procedure and evidence harness are present, but no physical result is fabricated or implied.

### Known limitations

- Runtime crash detection remains observation-driven. If a Rust listener exits while the UI and command surface are fully suspended, the foreground notification can remain stale until the next orchestrator observation; this step fixes reconciliation when state is next requested but does not add a hidden watchdog.
- Android foreground-service start is asynchronous. `startRequestPending` truthfully represents the gap between requesting `startForegroundService` and `onStartCommand` entering foreground state.
- Process-death/reboot recovery is not implemented. The service remains `START_NOT_STICKY`, no boot receiver exists, and in-process Rust listeners die with the application process.
- OEM durability, battery impact, thermal throttling, notification-stop UX, and multi-hour screen-off reachability remain unproven until the documented procedure is run on real hardware.

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