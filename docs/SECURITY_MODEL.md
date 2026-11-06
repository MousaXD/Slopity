# Security model

## Protected assets

- User worlds, configuration, logs, backups, and credentials.
- Device CPU, memory, storage, battery, and thermal headroom.
- Local network exposure.
- Integrity of downloaded runtimes and server packages.

## Initial trust boundaries

- The web frontend is untrusted presentation code relative to process execution.
- Tauri commands expose typed profile management and a built-in HTTP server lifecycle. Admission checks and stopped-profile guards belong in Rust; frontend data cannot authorize an arbitrary runtime.
- Runtime adapters accept an explicit executable and argument vector, never a generic shell string.
- New profiles bind to loopback by default.
- Runtime availability is separate from profile validity.

## Threats to address before real workloads

- Malicious archives and path traversal.
- Compromised download sources or dependency substitution.
- Server plugins/mods executing with app privileges.
- Port conflicts and accidental public exposure.
- Resource exhaustion, thermal damage, and crash loops.
- Command injection through profile fields.
- Unauthenticated remote administration.

## Android-specific boundary

Long-running hosting must be user-initiated, visible through a foreground-service notification, stoppable, and compliant with Android background execution rules. The Rust core does not bypass those rules. Native Kotlin lifecycle integration belongs behind the host-service plugin.

## Deliberate exclusions

The current UI cannot download runtime packages, execute arbitrary user commands, or expose an authenticated remote administration API. Only the built-in HTTP server is currently registered as runnable. Android host/foreground-service integration compiles in CI, but durable physical-device hosting remains unverified.

No downloadable package, plugin, archive-extraction, or third-party executable integration is approved until the source/version/license/integrity and path-traversal controls in [the dependency and supply-chain review](DEPENDENCY_SECURITY_REVIEW_2026-11-06.md) are implemented and tested.
