# Slopity engineering status and next-step coordination

**Observed:** 2026-11-05 UTC  
**Scope:** repository evidence only. This document is a living maintainer aid, not a claim of completed app/runtime verification.

## Product contract

Slopity is a Rust-first local server control plane with Tauri 2 desktop/mobile frontend. Preserve existing profile storage, orchestration, telemetry, structured process execution, Android host bridge, and built-in HTTP runtime. **Only the built-in HTTP provider is currently registered and runnable.** An unverified JVM/Java, Minecraft, Jellyfin, Node.js, Python, PHP, native or custom runtime must not appear supported simply because a profile or UI button exists.

The repository is public but distributed under PolyForm Noncommercial 1.0.0, a source-available license rather than an OSI-approved open-source license. Do not change legal terms as part of engineering maintenance.

The existing `AGENTS.md` two-commit protocol and separation of Rust core from Tauri/UI/platform adapters are mandatory.

## Verified baseline evidence

| Ref / workflow | Observed result | Interpretation |
| --- | --- | --- |
| `main` / `dev` `f57b7cee18dbf73b06eb64aa0c12212c4ca9333d` | Shared default head on 2026-11-05 | Does **not** contain the four draft feature/hardening/CI PRs |
| [Security run 41467880911](https://github.com/MousaXD/Slopity/actions/runs/41467880911), 2026-11-02 | Failed: Cargo audit tool install requires Rust 1.96 via `kstring 2.0.5`, but Rust 1.89 was active | Audit did **not** complete; npm audit passed. Do not call this a vulnerability finding or clean bill of health |
| [CI PR #11 run 32786433246](https://github.com/MousaXD/Slopity/actions/runs/32786433246), 2026-08-24 | Passed Rust quality, Linux Tauri, Android ARM64 build, Windows x64 NSIS | Validated only that branch's then-current exact head |
| [Audit PR #11 run 32786433268](https://github.com/MousaXD/Slopity/actions/runs/32786433268), 2026-08-24 | Passed Cargo audit, npm audit, dependency review | Historical baseline, not current dependency assurance |
| [Release PR #11 run 32786433207](https://github.com/MousaXD/Slopity/actions/runs/32786433207), 2026-08-24 | Passed package validation | Does not prove signed production release quality |
| [Runtime hardening PR #10 run 33215426044](https://github.com/MousaXD/Slopity/actions/runs/33215426044), 2026-08-28 | Failed one Rust unit test: `failed_stop_preserves_potentially_live_runtime`, assertion `snapshot.last_error.is_some()` | Real unresolved correctness regression; 50 tests passed, 1 failed |
| [GUI V2 PR #9 run 33214870336](https://github.com/MousaXD/Slopity/actions/runs/33214870336), 2026-08-28 | Rust setup step failed before test execution | Cannot conclude GUI code passed or failed actual tests |
| [Android host PR #12 run 32786160314](https://github.com/MousaXD/Slopity/actions/runs/32786160314), 2026-08-24 | Rust setup step failed before test execution | Cannot conclude host integration was validated |
| New Step 025 branch | **Pending exact-head Actions** | No local test claims. See draft PR checks |

As of this audit, 19 PRs were open: 15 Dependabot updates and four engineering drafts (#9, #10, #11, #12). Review dependencies as a coherent set only after restoring the baseline, not by blindly merging all PRs.

## Proposed specialist ownership lanes (five roles, not five agents launched)

These are *roles* for follow-up independent review/implementation; no autonomous processes were launched merely by writing this document.

1. **CI, packages, releases:** own recovery branch and PR #11 integration. Keep SHA-pinned Actions, locked Cargo/npm graphs, Rust/Linux/Android/Windows builds, PR no-publish behavior, artifact checks and truthful signing boundaries.
2. **Rust correctness & security:** own PR #10 failure and fail-closed admission/stop recovery, durable profile restoration, authority of observed state, port/memory enforcement and regression tests. Never weaken `last_error` or remove a failing assertion to go green.
3. **GUI/UX and accessibility:** integrate PR #9 without losing controls; validate profile flows, Tauri errors, modal focus/keyboard, small screens, offline state, empty/error states. First additions after integration: safe profile import/export, search/filter, actionable resource warnings, readable logs.
4. **Android platform & durability:** own PR #12 foreground service/notification state reconciliation, telemetry failures, host lifecycle invariants, and reproducible physical ARM64 matrix. CI compilation **cannot** substitute for one-hour background/battery/reachability proof.
5. **Dependency/security review & documentation:** audit Cargo/npm advisories, triage Dependabot updates in tested batches, review third-party package manifests and trust model, keep docs and claims in sync. Do not silence known vulnerabilities.

One integration owner coordinates sequence, branch boundaries, acceptance evidence, and reviews. A role is not a claim of an active separate agent.

## Integration and delivery sequence

1. **Recover CI now:** use PR #11's previously green cross-platform foundation as the base; isolate audit tooling from application Rust version and run all exact-head Actions on a draft replacement/reconciliation PR. Keep `dev`/`main` untouched until credible evidence exists.
2. **Fix backend correctness next:** examine the failed-stop path and Rust test in PR #10; ensure potentially live processes remain visible *and* failure evidence survives. Rerun Rust quality, then Linux/Android/Windows and audit workflows.
3. **Promote GUI V2 safely:** reconcile PR #9's modular frontend, tests and accessibility with the verified backend/CI. Make controls truthful about unsupported runtimes and error states; avoid UI-only "fake" implementations.
4. **Validate Android host boundary:** reconcile PR #12, run artifact/manifest CI, and record actual device evidence separately. Do not claim durability or release readiness until device testing.
5. **Deliver useful new functionality:** prioritize profile import/export as data only with schema validation, previews and conflict handling; search/filter and responsive accessibility; log/event inspection; backup/restore verification; package manifest trust gates. Add runtime providers one at a time behind platform and threat-model proof.
6. **Dependency batches and releases:** rebase/sequence Dependabot updates after stable CI; keep OSI-vs-noncommercial language accurate; publish only when release metadata, artifact provenance, and signing requirements meet the project's chosen release gate.

## Merge and security gates

- Branch PRs remain drafts until their exact heads pass appropriate GitHub Actions.
- No direct `main` edits, force pushes or premature promotion. Keep each slice reviewable and independently revertible.
- Fail closed for runtime and network boundaries. Never use `sh -c`/`cmd /C` for profile data; never claim downloaded runtimes/packages are safe without origin/license/version/hash controls.
- Do not confuse Rust cross-compilation, Windows NSIS packaging, or Android APK generation with a passed real-device runtime test.
- Update `PROGRESS.md` with actual job URLs, commit SHA, outcomes, and remaining limitations after implementation.

## Scope of Step 025

The audit-tool-only Rust update repairs installation incompatibility, not a previously identified application vulnerability. Current security disposition stays **unknown until the new audit job completes**. No runtime providers, GUI components, signing, or product features are added in this CI recovery slice.

## 2026-11-05 exact-head CI recovery addendum

### Exact-head GitHub-hosted verification (2026-11-05)

PR #35 head `0a5309ac49f3bd5e01dbb1fdb0014fa67bb01a7c` was independently queried using GitHub Actions commit-associated runs:
- [Cross-platform CI #42081297534](https://github.com/MousaXD/Slopity/actions/runs/42081297534): completed, success.
- [Security and dependency audit #42081297435](https://github.com/MousaXD/Slopity/actions/runs/42081297435): completed, success.
- [Package and publish releases #42081297671](https://github.com/MousaXD/Slopity/actions/runs/42081297671): completed, success. PR release publication is not authorized; this is package validation, not a signed release.

No local or physical Android device tests were performed. These runs validate only the named commit, not feature PRs #9, #10, #12 or their integration. PR #10's failed-stop evidence assertion remains a release blocker. GUI accessibility and Android one-hour durability proof remain outstanding.

Historical pending language above refers to the earlier observation, before these runs completed. Step 025 and Step 028 now have successful exact-head workflow evidence; their ledger entries retain their historical pending wording pending a separate factual reconciliation, without retroactively altering their implementation commits.
