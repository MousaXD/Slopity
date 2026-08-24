# Step 021: production-grade release pipeline and constrained Minecraft E2E

**Status:** IN PROGRESS  
**Declared:** 2026-08-24

## Scope

1. Split pull-request validation from main-branch publication so pull requests never publish releases.
2. Add strict synchronized version validation and deterministic duplicate-tag failure behavior.
3. Restore Windows CI and NSIS packaging alongside Linux and Android release builds.
4. Add optional ephemeral Android release signing using repository secrets while preserving an explicitly named debug prerelease fallback when signing is absent.
5. Add a dedicated constrained Minecraft Java runtime identity/provider used by a real official-Mojang Minecraft server E2E through `ServerOrchestrator`, without registering generic Java or arbitrary process execution.
6. Require explicit `MINECRAFT_EULA_ACCEPTED=true` repository configuration before the real-server E2E may create `eula.txt`.
7. Pin the official Minecraft server version/source/checksum metadata, verify the download, perform a real protocol status ping, verify clean orchestrated shutdown and immediate port release, and retain sanitized failure diagnostics for seven days.
8. Generate release notes from GitHub release metadata, produce deterministic names and SHA-256 checksums, and publish only after all quality/E2E/platform gates pass.
9. Update README, current status, roadmap, architecture, security and release documentation truthfully.

## Non-goals

- Merging this branch or its pull request.
- Publishing releases from pull requests.
- Enabling generic `RuntimeKind::Java`, Node.js, Python, PHP, Native, Custom, shell strings, or arbitrary user-controlled executables.
- Committing Minecraft server JARs/worlds, Android keystores, signing passwords, generated Android projects, credentials, or release binaries.
- Claiming Windows Authenticode signing, Android production signing, or physical Android durability proof until those are actually configured and validated.

## Risks

- Tauri Windows/Android output paths can drift and must be asserted instead of globbed loosely.
- Minecraft startup semantics and protocol/version changes can make an E2E flaky unless version, Java requirement, source and checksum are pinned.
- Release workflows must fail deterministically on reused versions rather than silently skipping.
- Android signing must be all-or-none and ephemeral so partial secret configuration cannot create mislabeled packages.
- Main release concurrency must never cancel a publication already in progress.

## Intended files

- `.github/workflows/ci.yml`
- `.github/workflows/release.yml`
- `.github/release.yml`
- `scripts/check-version.sh`
- `scripts/bump-version.sh`
- `scripts/download-minecraft-server.sh`
- `scripts/configure-android-signing.py`
- `ci/minecraft-server.json`
- `crates/slopity-core/src/model.rs`
- `crates/slopity-core/src/validation.rs`
- `crates/slopity-runtime-local/src/lib.rs`
- `crates/slopity-runtime-local/src/minecraft.rs`
- `crates/slopity-runtime-local/tests/minecraft_host_e2e.rs`
- `README.md`
- `CURRENT_STATUS.md`
- `TASK.md`
- `PROGRESS.md`
- `docs/ARCHITECTURE.md`
- `docs/SECURITY_MODEL.md`
- `docs/releases.md`

## Acceptance checks

- `cargo fmt --all -- --check`
- `cargo test --workspace --all-features --locked`
- `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`
- `cargo metadata --locked --format-version 1`
- `cargo audit`
- `cd apps/slopity && npm ci && npm audit && npm run tauri:check`
- workflow YAML parsing
- `scripts/check-version.sh`
- Windows Tauri compile/package validation in GitHub Actions
- Android ARM64 compile and merged-manifest validation in GitHub Actions
- real Minecraft E2E through `ServerOrchestrator` when explicit EULA acceptance is configured
- no release publication from pull-request workflows
