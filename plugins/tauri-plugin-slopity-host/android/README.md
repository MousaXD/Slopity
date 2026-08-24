# Android host-service bridge

The Android plugin owns Slopity's user-initiated `specialUse` foreground service. It keeps native foreground-service state separate from the Rust process-local fallback so the application can reconcile status after activity recreation or plugin reconnection while the same process remains alive.

On Android 13 and later the plugin declares and requests `POST_NOTIFICATIONS` when hosting is started and the permission is still promptable. Slopity now treats notification delivery as part of the hosting safety boundary: if `POST_NOTIFICATIONS` is denied, app notifications are disabled, or the Slopity hosting channel is disabled, a new server start is rejected rather than silently accepting hosting without the persistent notification expected by the product. Android may technically allow some foreground-service execution without drawer notification permission, but Slopity deliberately uses the stricter visible-hosting policy.

Repeated start calls update the existing foreground notification with the current hosting label and active-server count. Native status reports both `startRequestPending` and `stopRequestPending`, plus app/channel notification-delivery gates. Immediate `startForegroundService` exceptions are returned to Rust and the pending native state is cleared. The service also clears its snapshot if promotion through `startForeground` throws.

The notification is ongoing and tapping it opens Slopity. Its `Stop safely` action records a native stop request and opens Slopity so the Rust/application layer can stop hosted listeners before the foreground service is removed. The Kotlin service deliberately does not call `stopSelf()` for that action while Rust listeners may still be alive. Fresh dashboard, host-status, and server-list observations consume the pending request and reconcile the native service with the authoritative Rust orchestrator.

Reconciliation is conservative:

- zero active Rust servers cause a stale host service to stop;
- an active-server count mismatch updates the notification;
- active Rust servers with a missing host service cause a restart attempt;
- if a missing host service cannot be restored, Slopity attempts to stop the active Rust servers instead of knowingly leaving them without required foreground state;
- a pending native stop request stops active Rust servers first, then removes the foreground service;
- notification delivery becoming unavailable is treated like a safety failure on the next observation.

This native state remains in-process only. It does not implement recovery after Android kills the application process. `START_NOT_STICKY` remains unchanged, no boot receiver is registered, no vendor-specific background bypass is used, and hosting remains explicitly user initiated. Runtime crashes or notification-setting changes that occur while the UI is fully suspended are reconciled when Slopity next observes state; the current architecture does not claim an always-running native watchdog.

Durable Android hosting remains unproven until the physical-device lifecycle procedure in `docs/android-device-durability.md` is completed on real ARM64 hardware.
