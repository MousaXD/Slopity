# Agent 05 — Dependency and supply-chain review (2026-11-06)

**Repository:** `MousaXD/Slopity`  
**Reviewed baseline:** `feat/owner-integration-20261106` at `31d524fba0b72a4a79ec7d6bac3b97c749d564f2` ([integration PR #43](https://github.com/MousaXD/Slopity/pull/43)).  
**Change branch:** `fix/slopity-deps-agent05`; targets that integration branch via a draft PR.  
**Decision:** one compatible `serde_json` lockfile update, Dependabot routing correction, documented safety gates. No workflow/frontend/runtime code changed.

## Verified findings and evidence

- The Rust workspace has a committed `Cargo.lock`, a Rust **1.89.0** pinned toolchain (`rust-toolchain.toml`), a declared package minimum of **1.82**, and exact workspace dependency requirements in `Cargo.toml`. Actual Rust CI validates **1.89.0**; a separate Rust 1.82 MSRV matrix is **not** established.
- The npm graph has committed lockfile v3 and just `@tauri-apps/cli@2.11.4` as a direct **development** dependency, with integrity hashes and optional platform-specific CLI packages. All build jobs use `npm ci`.
- GitHub Actions use SHA-pinned third-party steps; all jobs default to `contents: read`, while only the final release publication job grants `contents: write`. Audit tooling installs `cargo-audit@0.22.2 --locked` under a separately selected `stable` Rust toolchain to avoid requiring an application compiler change.
- The baseline [security run 42275322860](https://github.com/MousaXD/Slopity/actions/runs/42275322860) completed successfully: **Cargo vulnerability audit**, **npm audit**, and **dependency review** all passed. **Important correction after examining the raw cargo-audit log:** although the vulnerability audit passed, it emitted **eight allowed warnings** (six unmaintained crates, two soundness warnings), enumerated below. The absence of vulnerability-class failures is **not** equivalent to a warning-free dependency graph or evidence of safety for proprietary binaries, build-time CLI payloads or future archives.
- The baseline [CI run 42275322969](https://github.com/MousaXD/Slopity/actions/runs/42275322969) passed **Rust formatting, locked metadata/tests, strict Clippy, Linux Tauri compilation, Android ARM64 debug builds/manifest checks and Windows NSIS packaging**. The [release-package run 42275322783](https://github.com/MousaXD/Slopity/actions/runs/42275322783) ultimately **succeeded** for both Android debug APK/AAB and Linux .deb/AppImage; publication remained inapplicable to pull requests. The earlier in-progress observation was transient.
- Step 050 changed `serde_json 1.0.151 -> 1.0.154` using the checksum from [Dependabot #29](https://github.com/MousaXD/Slopity/pull/29), within the existing workspace semver range. Its exact implementation commit `bdfebbf` passed [CI 42279980309](https://github.com/MousaXD/Slopity/actions/runs/42279980309), [audit 42279980285](https://github.com/MousaXD/Slopity/actions/runs/42279980285), and [release packaging 42279980270](https://github.com/MousaXD/Slopity/actions/runs/42279980270). Step 051 separately updates locked `tokio 1.53.1 -> 1.53.3` to address an actual soundness advisory; its implementation SHA requires fresh exact-head validation.

## Step 052 — patched Tauri cross-WebView IPC advisory (new finding 2026-11-08)

**High priority, isolated stacked security fix (exact-head CI pending).** The upstream Tauri [GHSA-w28w-mhc8-qvjv](https://github.com/tauri-apps/tauri/security/advisories/GHSA-w28w-mhc8-qvjv) (published 2026-09-26, severity **High**) reports broken isolation between WebViews in queued IPC response retrieval. A WebView that can invoke Tauri internals could retrieve and drain channel data intended for a different WebView when certain conditions hold. **Affected Tauri:** `>=2.0.0, <=2.11.5`; **patched:** `>=2.11.6`. The original Step 051 lockfile pinned affected `tauri 2.11.5`. The app's use of multiple untrusted WebViews, and exploitation in Slopity, have **not** been demonstrated.

The separate Step 052 branch `agent-05/tauri-2-11-6-ipc-security`, stacked on fully certified Agent 05 PR #48, changes **only** `tauri 2.11.5 -> 2.11.6` and its crates.io checksum `667b20e2726d572dea2de7370da16e188eb06008faf9a92fab7cdc46791190b5 -> 6fa5bacdb9bbad5954af3d1bd6cf6ae9192cab1b2e270f4a07f904610b9e85f4`, matching existing [Dependabot PR #20](https://github.com/MousaXD/Slopity/pull/20). Upstream `tauri-v2.11.6` remains Rust edition 2021 and workspace `rust-version 1.77.2`, compatible with Slopity's **pinned Rust 1.89.0**. No manifest or other lockfile graph changes are required by the Dependabot patch. The `tauri` security change has **not yet** been validated at this new exact commit and must pass Rust tests, strict Clippy, Linux Tauri, Android ARM64, Windows NSIS, Cargo/npm audits, dependency review, and PR packaging before it can be integrated.

**Advisory coverage nuance:** This GitHub GHSA is a distinct issue from the seven RustSec `cargo audit` informational warnings (1 glib unsound, 6 unmaintained). A green `cargo audit` on `tauri 2.11.5` did **not** prove the GHSA absent; the targeted patch is driven by the upstream fixed-version statement. The `glib 0.18.5` unsoundness and unmaintained GTK/urlpattern crates remain open issue [#52](https://github.com/MousaXD/Slopity/issues/52), not suppressed by this change. Tauri 2.12+ raises its MSRV to Rust 1.90 and still depends on GTK 0.18 for Linux, so is not substituted into this narrowly scoped compatible fix.

## Cargo audit advisory triage: eight allowed warnings

The [Cargo audit raw job log on Step 050 commit `bdfebbf`](https://github.com/MousaXD/Slopity/actions/runs/42279980285/job/128106136604) loaded **1,422** advisories, scanned **434** locked crates, reported **8 allowed warnings**, and exited successfully. The default `cargo audit` exit code did **not** indicate a clean informational-warning inventory. `npm audit` on the same head reported **0 vulnerabilities**. No `--ignore` suppression was introduced.

| Advisory | Locked crate | Category | Action |
| --- | --- | --- | --- |
| [RUSTSEC-2024-0370](https://rustsec.org/advisories/RUSTSEC-2024-0370.html) | `proc-macro-error 1.0.4` | Unmaintained | Map transitive dependency owners; upstream replacement may need macro ecosystem migration |
| [RUSTSEC-2025-0081](https://rustsec.org/advisories/RUSTSEC-2025-0081.html) | `unic-char-property 0.9.0` | Unmaintained | Map Unicode/GTK/Tauri dependency chain before selecting compatible alternative |
| [RUSTSEC-2025-0075](https://rustsec.org/advisories/RUSTSEC-2025-0075.html) | `unic-char-range 0.9.0` | Unmaintained | Same Unicode dependency-chain assessment |
| [RUSTSEC-2025-0080](https://rustsec.org/advisories/RUSTSEC-2025-0080.html) | `unic-common 0.9.0` | Unmaintained | Same Unicode dependency-chain assessment |
| [RUSTSEC-2025-0100](https://rustsec.org/advisories/RUSTSEC-2025-0100.html) | `unic-ucd-ident 0.9.0` | Unmaintained | Same Unicode dependency-chain assessment |
| [RUSTSEC-2025-0098](https://rustsec.org/advisories/RUSTSEC-2025-0098.html) | `unic-ucd-version 0.9.0` | Unmaintained | Same Unicode dependency-chain assessment |
| [RUSTSEC-2024-0429](https://rustsec.org/advisories/RUSTSEC-2024-0429.html) | `glib 0.18.5` | **Unsound** | Patched `glib >=0.20` is beyond current 0.18 API; coordinate Tauri/GTK ecosystem migration and Linux regression checks with Agents 01/02 |
| [RUSTSEC-2026-0386](https://rustsec.org/advisories/RUSTSEC-2026-0386.html) | `tokio 1.53.1` | **Unsound; memory-corruption potential under affected `AsyncFd` constructors** | **Step 051 targeted lockfile update to 1.53.3** (patched range `>=1.53.3`). Existing Tokio feature graph and Rust 1.89 toolchain retained; post-update CI/audit must verify advisory disappears |

Tokio 1.53.3 upstream `tokio/Cargo.toml` declares `rust-version = "1.71"`, and upstream's 2026-10-13 release deprecates the affected constructors in favor of new explicitly unsafe registration methods. This patch does **not** claim that all transitive callers or all unsoundness have been proven safe, merely that the locked version moves into the RustSec patched range. Avoid Tokio **1.54.0**, which has a separately reported interval race; moving to 1.53.3 minimizes regression risk.

**Remaining risk:** after successful updated-head CI, seven informational warnings are expected to remain, including the `glib` unsoundness warning. Treat these as visible remediation debt; do not silence the warnings merely to produce a prettier security dashboard. Full exploitation exposure depends on whether affected APIs are actually invoked, which is not established by a lockfile scan.

## Outstanding Dependabot PR decisions (15 reviewed)

All listed PRs were open at audit time and authored against an older `main` tree; do **not** merge their old patches into the integration tree without re-evaluation.

| PR | Dependency / requested version | Disposition |
| --- | --- | --- |
| [#4](https://github.com/MousaXD/Slopity/pull/4) | `actions/dependency-review-action` v5.0.0 | **Superseded:** exact v5 SHA is already pinned in `audit.yml`; do not merge old workflow hunk |
| [#6](https://github.com/MousaXD/Slopity/pull/6) | `actions/checkout` v7.0.1 | **Superseded:** exact v7 SHA already pinned throughout audited workflows |
| [#14](https://github.com/MousaXD/Slopity/pull/14) | `softprops/action-gh-release` v3.0.3 | **Agent 01:** current v3.0.2; evaluate patch and publication/permission behavior |
| [#16](https://github.com/MousaXD/Slopity/pull/16) | `android-actions/setup-android` v4.0.4 | **Agent 01:** current v4.0.1; retain critical explicit `packages: 'platform-tools'` override |
| [#19](https://github.com/MousaXD/Slopity/pull/19) | `thiserror` v2.0.21 | **Next isolated Cargo batch:** current 2.0.20 plus 1.0.69; lockfile/test on 1.89.0 |
| [#20](https://github.com/MousaXD/Slopity/pull/20) | `tauri` v2.11.6 | **Next isolated Cargo batch:** current 2.11.5; test all three desktop/mobile targets |
| [#24](https://github.com/MousaXD/Slopity/pull/24) | `tauri-build` v2.7.1 | **Defer:** transitive lockfile expansion; inspect MSRV and bundle compatibility first |
| [#26](https://github.com/MousaXD/Slopity/pull/26) | `actions/download-artifact` v8.0.2 | **Agent 01:** current v8.0.1, test release artifact round-trip |
| [#27](https://github.com/MousaXD/Slopity/pull/27) | `actions/setup-node` v7.1.0 | **Agent 01:** current v7.0.0; test npm cache/Node 22 across platforms |
| [#28](https://github.com/MousaXD/Slopity/pull/28) | `actions/upload-artifact` v7.0.2 | **Agent 01:** current v7.0.1; test package artifacts and upload/download parity |
| [#29](https://github.com/MousaXD/Slopity/pull/29) | `serde_json` v1.0.154 | **Adopted selectively in this draft PR** via exactly scoped `Cargo.lock` package/checksum |
| [#30](https://github.com/MousaXD/Slopity/pull/30) | `actions/setup-java` v6.1.0 | **Agent 01:** current v5.7.0; Java 17/Gradle/Android runner compatibility validation |
| [#31](https://github.com/MousaXD/Slopity/pull/31) | `dtolnay/rust-toolchain` newer action SHA | **Agent 01:** verify provenance and 1.89.0 selection; do not change pinned compiler |
| [#33](https://github.com/MousaXD/Slopity/pull/33) | `tauri-plugin` v2.7.2 | **Defer:** transitive graph and Android plugin boundary need coordinated tests |
| [#34](https://github.com/MousaXD/Slopity/pull/34) | `@tauri-apps/cli` v2.12.4 | **Priority Agent 03 + Agent 01:** release notes cite fixes to Rust ZIP advisories RUSTSEC-2026-0367/0368/0369 in underlying CLI; assess actual affected build-tool surface and full platform packaging before adoption |

**Important:** the cited ZIP advisories appear in upstream CLI update notes, **not** in the baseline application's passing `Cargo.lock` audit. This is a build-tool review trigger, *not* proof the current Slopity release artifact is vulnerable. Evaluate the CLI binary and installation paths separately from Rust workspace dependencies. No automated closure/merge/retarget of old PRs is performed here.

## Supply-chain, action, release, and reproducibility controls

**Already present:** Cargo/npm lockfiles, npm integrity digests, action commit SHAs, separate minimal write token scope, strict Cargo audit, npm audit, dependency review, locked Rust tests, and `SHA256SUMS.txt` for preview packages. Pull-request builds do not publish releases.

**Open engineering gaps:** runner images (`ubuntu-latest`, etc.), apt/system packages, `stable` scanner compiler, and externally fetched SDK/NDK inputs are not fully content-addressed or hermetic. The checksums created in the *same* publication pipeline protect subsequent transfers but are not independently signed provenance. Production signing, verified provenance/attestation, artifact verification policy, reproducible-build comparison, verified Android physical-device hosting, and a formal release rollback process remain outstanding. Coordinate workflow and release corrections with Agent 01; do not weaken existing audit gates.

## Mandatory future untrusted-package threat-model gates

There is **no authorized downloadable external runtime/package installer today**. Any future implementation must pass an independent security review before providing UI controls that install/execute packages:

1. **Identity and consent:** require an explicit versioned schema, package ID, immutable version, publisher/official origin, SPDX-compatible *declared* license identifier, minimum platform/ABI, required runtime, entry point and per-file or archive SHA-256. Treat all plugin/JAR/script packages as executable code. An unreviewed license declaration must not imply legal authorization.
2. **Source and transport:** explicit trusted origin or per-source user approval, HTTPS verification, normalized redirect restrictions, no untrusted redirects to private/link-local/localhost destinations, safe URL parsing, network timeout/size limits and bounded retries. Verify the final origin; reject insecure schemes and origin-substitution.
3. **Integrity and authenticity:** download to a unique non-executable temporary file; hash the full bytes and verify against an immutable manifest acquired from a *separately authenticated* channel before extraction or launch. Require cryptographic signature/provenance verification where trust policy calls for it; a digest served beside the payload by the same untrusted server is not proof of publisher identity.
4. **Safe extraction:** reject absolute, drive-prefixed, NUL, `..`, empty and normalized-colliding paths; disallow symlink, hardlink, device/FIFO and extraction root escape. Avoid following pre-existing links; use directory-relative safe APIs. Prevent zip bombs with entry counts, decompressed/ratio and disk quotas. Stage in private directory, fsync where appropriate, verify contents, atomically commit or clean up completely.
5. **Execution boundary:** do not automatically execute installation hooks or package-contained code. Never embed user data in shell command strings. Require a reviewed adapter, explicit executable and argument array, least-privilege environment, no shared secrets, resource/port limits, fail-closed runtime availability, and clear opt-in/LAN exposure warning.
6. **Lifecycle:** version pinning/rollback, revocation and hash mismatch handling, checked cleanup and durable state, removal safety, visible logs, user consent, cancellation and non-silent failure. Android requires foreground-service and real-device validation before unsupported package execution claims.
7. **Tests:** adversarial malicious archives (zip-slip, symlink/hardlink escape, duplicate paths, zip bombs), checksum/origin failure, partial-download recovery, competing installs, interruption/atomicity, privileges and OS-specific path normalization; CI cross-platform plus real-device platform proof.

## Coordination and outstanding security risks

- **Agent 01:** independently inspect deferred action bumps, release publication boundaries, absence of attestation and signing; *do not* copy old Dependabot workflow hunks into current workflows.
- **Agent 03:** own `@tauri-apps/cli` update #34 with locked npm integrity, frontend tests, platform package tests and a security-significance assessment.
- **Agent 02:** review dormant `LocalProcessRuntime::stop` in `crates/slopity-runtime-local/src/lib.rs`: it removes the child from the map *before* `kill()/wait()` can fail. This adapter is not the registered built-in HTTP provider today, but would risk losing potentially-live-process visibility if enabled later. The agent should design a failure-retaining stop path before promoting generic executables.
- **Agent 00:** coordinate the PR, downstream approvals, updated evidence and any decision to ship/sign. No external runtime or production-signing claims from this audit.

**No exploit, vulnerability, or package source is asserted beyond the evidence cited above.** Audit success is point-in-time and cannot replace threat modeling, signing, attestation, or physical-device testing.
