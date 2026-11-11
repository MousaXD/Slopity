# Changelog

## 0.2.0 — 2026-11-11 (preview)

Slopity's integrated overhaul consolidates the previously isolated Rust, Tauri, Android, GUI and CI improvements in [owner integration PR #43](https://github.com/MousaXD/Slopity/pull/43).

### Added and improved

- **Runtime safety:** fail-closed resource/port admission, mutation-safe start permissions, durable stop-failure evidence, and deterministic concurrency regressions.
- **GUI:** refined profile discovery and filtering, safe JSON profile import/export as data only, clearer device/resource state and keyboard/focus accessibility checks. Existing profile create/edit/clone/enable/disable/delete functionality remains.
- **Android:** foreground-host acknowledgement and notification safety gates, native pending-stop reconciliation that refuses unsafe restarts, and tests for host stop and notification visibility; non-Android platforms retain their own host behavior.
- **Build and security:** Rust workspace checks, frontend tests, Linux Tauri build, Windows NSIS preview, Android ARM64 debug builds and native Kotlin JUnit verification in GitHub Actions; patched Tauri IPC and Tokio versions and tightened dependency workflow integrity.
- **Packaging:** Linux .deb/AppImage and Android ARM64 **debug** APK/AAB with SHA-256 checksums in a GitHub prerelease.

### Known limitations

- **Preview only, not a production-signed release.** Android bundles are debug builds. Physical Android one-hour background/reachability/foreground-notification durability, real installed-GUI accessibility, production signing, provenance and rollback still require verification; see [#44](https://github.com/MousaXD/Slopity/issues/44).
- The dependency audit reports seven allowed RustSec warnings, including unsound `glib 0.18.5`; see [#52](https://github.com/MousaXD/Slopity/issues/52). A passing audit workflow does not mean zero advisories.
- The only registered, verified supported server runtime is the built-in HTTP test provider. Other runtimes remain future work, not shipping capabilities.
- This is an **owner-authorized development merge**; the independent Agent 06 exact-final-SHA GO was not available and is not claimed. The repository owner explicitly accepted the process exception with source and CI risks documented on PR #43 and QA issue #45.

Earlier releases/tags are immutable; version `0.2.0` triggers the existing prerelease workflow after a successful merge to `main`.
