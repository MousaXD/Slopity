# Android host-service bridge

The Android plugin owns Slopity's user-initiated `specialUse` foreground service. It keeps the native foreground-service state separate from the Rust process-local fallback so the application can reconcile status after activity recreation or plugin reconnection while the same process remains alive.

On Android 13 and later the plugin declares and requests `POST_NOTIFICATIONS` when hosting is started and the permission is still promptable. Notification permission is not treated as permission to run the foreground service: if the user denies notifications, hosting state can still be active while `notificationVisible` is reported as false. App-wide or channel-level notification blocking is also reflected conservatively.

Repeated start calls update the existing foreground notification with the current hosting label and active-server count. Tapping the notification opens Slopity. The notification's `Request stop` action records a native stop request and changes the notification text, but it deliberately does not stop the Kotlin service by itself because the hosted Rust listeners must be shut down first. The Rust/application layer can read `stopRequestPending` through the host-service status command and perform safe reconciliation.

This native state is in-process only. It does not prove or implement recovery after Android kills the application process. `START_NOT_STICKY` remains unchanged, no boot receiver is registered, and hosting remains explicitly user initiated. Durable Android hosting remains unproven until the physical-device lifecycle tests are completed.
