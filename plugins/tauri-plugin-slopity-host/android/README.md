# Android native bridge placeholder

The plugin crate is registered now so its public boundary remains stable. The Kotlin foreground service is intentionally not generated in this step.

The next Android proof step must add a Tauri mobile plugin class and a user-visible `specialUse` foreground service, then validate it on a real ARM64 phone. Until that evidence exists, `durable_hosting_available` remains false on Android.
