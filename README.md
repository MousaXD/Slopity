# PocketHost

PocketHost is an Android-first control plane for hosting multiple local network server workloads on a phone. The project intentionally separates the Android application from executable runtime adapters so Minecraft, Node.js, PHP, Python, native binaries, and future engines can be added without turning the app into one giant process-launching knot.

## Current state

This repository is a **foundation**, not a finished server runner.

Implemented now:

- Kotlin and Jetpack Compose Android application.
- Device RAM, CPU, ABI, storage, and thermal-status probing.
- Conservative capability-based hosting recommendations.
- Multiple server profile domain model.
- Runtime adapter registry and orchestration boundary.
- Foreground hosting service with persistent controls.
- Sample profiles for Paper, Node.js, and PocketMine-MP.
- Agent workflow, task roadmap, progress ledger, Git hooks, and CI.

Not implemented yet:

- Bundled Android-compatible JVM, PHP, Node.js, Python, or native runtime.
- Arbitrary executable launch.
- Persistent profile storage.
- Server package downloads.
- Real start, stop, console, logs, networking, backup, or update operations.

The UI labels these adapters as unavailable instead of pretending they work.

## Repository bootstrap

The Gradle wrapper JAR is intentionally bootstrapped on first use and verified with Gradle's published SHA-256 checksum. This keeps the starter ZIP small while still pinning the build tool.

Linux/macOS:

```bash
chmod +x gradlew scripts/*.sh .githooks/pre-commit
./gradlew --version
./gradlew testDebugUnitTest lintDebug assembleDebug
```

Windows:

```powershell
.\gradlew.bat --version
.\gradlew.bat testDebugUnitTest lintDebug assembleDebug
```

Requirements:

- JDK 17 or newer available as `java`.
- Android SDK platform 36 and Build Tools 36.0.0.
- Internet access for the first Gradle and dependency download.

## Start working

Read these files in order before changing code:

1. `AGENTS.md`
2. `TASK.md`
3. `PROGRESS.md`
4. `docs/ARCHITECTURE.md`
5. `docs/SECURITY_MODEL.md`

Install the repository Git hook:

```bash
./scripts/install-git-hooks.sh
```

Create a feature branch. Do not implement directly on `main`.

## Product principle

PocketHost is a manager and isolation boundary. Each runtime must prove that it can operate safely and reliably on Android before the UI presents it as supported. No downloaded server package may receive a generic unrestricted shell by default.
# Slopity
