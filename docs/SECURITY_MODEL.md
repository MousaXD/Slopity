# Security model

## Protected assets

- User worlds, configuration, logs, backups, and credentials.
- Device CPU, memory, storage, battery, and thermal headroom.
- Local network exposure.
- Integrity of downloaded runtimes and server packages.

## Initial trust boundaries

- The web frontend is untrusted presentation code relative to process execution.
- Tauri commands expose only typed profile inspection and validation in this foundation.
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

The current UI cannot download runtimes, launch arbitrary user commands, expose a remote API, or claim Android background hosting support.
