# Progress ledger

Append new steps at the top. Do not rewrite completed history except to correct a factual error, and explain any correction in a new entry.

## Step 000: repository foundation

**Status:** DONE  
**Declared:** 2026-08-05  
**Completed:** 2026-08-05

### Scope

Create a build-oriented Android starter repository for a multi-server phone host. Establish honest runtime boundaries, device capability advice, foreground-service infrastructure, project documentation, agent workflow, and CI.

### Non-goals

- Running Minecraft or any other real server engine.
- Downloading or bundling executable runtimes.
- Persisting user-created profiles.
- Remote access or authentication.
- Publishing an APK.

### Delivered

- Single-module Android application using Kotlin and Jetpack Compose.
- Capability probe for total/available memory, CPU cores, ABI, storage, and current thermal status.
- Conservative hosting plan calculator with unit tests.
- Multi-runtime server profile model and sample profiles.
- Runtime adapter registry whose initial adapters report unavailable truthfully.
- Orchestrator preflight decisions for memory, ports, architecture, and runtime availability.
- User-initiated foreground hosting service declared as `specialUse`.
- Persistent notification with an explicit stop action.
- `AGENTS.md`, `TASK.md`, architecture and security documentation.
- Git hook and CI guard requiring `PROGRESS.md` alongside implementation changes.
- Gradle wrapper bootstrap with a pinned checksum.

### Verification performed

- Generated file tree inspected.
- XML resources parsed with a strict XML parser.
- TOML version catalog parsed.
- Gradle wrapper downloader compiled with JDK 21.
- Unit-test source and build configuration inspected.

### Verification not performed

- Android compilation, lint, unit tests, and APK assembly were not executed in the generation environment because an Android SDK was not available.
- No on-device test was performed.

### Next step

Start Phase 1 with a documentation-only planning commit for profile persistence and CRUD.
