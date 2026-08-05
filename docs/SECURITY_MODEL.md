# Security model

## Protected assets

- User worlds and server data.
- Device battery, CPU, memory, thermal health, and storage.
- Local network exposure.
- Downloaded runtime and server-package integrity.
- Future remote-management credentials.
- Android application signing identity.

## Primary threats

- Malicious server packages, plugins, mods, or scripts.
- Archive path traversal and storage exhaustion.
- Command injection through profile arguments.
- Unauthenticated remote administration.
- Servers binding beyond the user's intended interfaces.
- Runtime escape into unrelated app or device data.
- Resource starvation, thermal damage, battery drain, and crash loops.
- Supply-chain compromise of downloaded runtimes.

## Initial controls

- No functional runtime adapter is shipped in the foundation.
- Runtime availability is explicit and defaults to unavailable.
- Profile commands are modeled as structured fields, not shell strings.
- The app uses private storage and does not request broad storage access.
- Hosting begins only after a visible user action.
- A persistent foreground notification exposes a stop control.
- Capability advice reserves memory for Android.

## Required controls before executing packages

- Cryptographic checksums and provenance records.
- Safe extraction that rejects absolute paths, traversal, links, and oversized entries.
- Per-instance storage quotas.
- Fixed executable entry points and structured arguments.
- Loopback-first binding and explicit network exposure.
- Authentication before any remote control.
- Bounded logs and backups.
- Graceful-stop timeout followed by explicit forced termination.
- Crash-loop backoff.
- Runtime and package license review.

## Out of scope for the foundation

The current app does not claim strong isolation between a future server process and the app itself. Android's application sandbox protects the app from other apps, but code executed inside PocketHost's identity must be treated as having access to PocketHost's private files unless a stronger isolation design is proven.
