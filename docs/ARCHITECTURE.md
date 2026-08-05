# Architecture

## Layers

### `slopity-core`

Pure Rust domain code shared by every platform. It owns profile semantics, lifecycle states, resource planning, validation, and runtime contracts. It must not depend on Tauri, Kotlin, WebViews, or UI frameworks.

### Runtime adapters

Adapters implement the core `RuntimeAdapter` trait. The first adapter, `slopity-runtime-local`, is desktop-only in behavior and uses `std::process::Command` with explicit executable and argument fields. Android runtime execution remains a separate proof track.

### Tauri shell

`apps/slopity/src-tauri` translates core data into a small command API for the shared static frontend. Mobile builds use the library entry point required by Tauri; desktop builds also provide a normal binary entry point.

### Host-service plugin boundary

`tauri-plugin-slopity-host` gives the shell one stable place to ask whether durable user-visible hosting is available. The foundation intentionally reports Android foreground-service hosting as pending until Kotlin service integration and a real ARM64 test are complete.

## Portability matrix

| Platform | App shell | Shared core | Local process adapter | Durable hosting claim |
| --- | --- | --- | --- | --- |
| Windows | Yes | Yes | Foundation present | Not yet validated |
| Linux | Yes | Yes | Foundation present | Not yet validated |
| Android | Yes | Yes | Not used | Pending foreground-service proof |
| iOS | Future controller | Core-compatible | No | Not planned |

## Runtime model

Profiles describe intent, not proof of availability. Each profile has a runtime kind, executable metadata, structured arguments, working directory, memory budget, port, and network scope. Availability is reported separately by a runtime provider.

The UI must never infer that a profile is runnable merely because its schema is valid.
