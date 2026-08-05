# Slopity

Slopity is a Rust-first, lightweight control plane for hosting and managing multiple local server workloads. The same core is designed to power Windows, Linux, and Android builds through Tauri 2.

## What exists now

- A platform-neutral Rust core for server profiles, resource planning, lifecycle states, validation, and runtime contracts.
- A desktop local-process adapter that launches explicit executables with structured argument arrays and never routes profiles through `sh -c` or `cmd /C`.
- A Tauri 2 shell with one static HTML/CSS/JavaScript frontend for Windows, Linux, and Android.
- A host-service plugin boundary that reports platform capabilities honestly. The Android foreground-service implementation is deliberately marked pending until native integration and an ARM64 device test exist.
- GitHub Actions for Rust checks, Windows and Linux Tauri compilation, and Android initialization/build validation.

Slopity does **not** yet bundle Java, Node.js, Python, PHP, Minecraft, or any other server engine. A profile can describe those runtimes, but the UI labels them unavailable until an adapter and runtime provider have been proven.

## Architecture

```text
apps/slopity/web              shared lightweight UI
apps/slopity/src-tauri        Tauri desktop/mobile shell
crates/slopity-core           portable domain and orchestration contracts
crates/slopity-runtime-local  desktop child-process adapter
plugins/tauri-plugin-slopity-host  platform host-service capability boundary
```

The Rust core has no Tauri, WebView, Kotlin, or operating-system UI dependency. Android-specific lifecycle work belongs behind the plugin boundary rather than leaking into server profile logic.

## Local development

Install Rust, Node.js, and the platform prerequisites listed by Tauri.

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings

cd apps/slopity
npm install
npm run tauri:dev
```

### Windows

Install Microsoft C++ Build Tools and WebView2. Then run `npm run tauri:build` from `apps/slopity`.

### Linux

Install WebKitGTK 4.1 and the other Tauri system packages for your distribution. Then run `npm run tauri:build` from `apps/slopity`.

### Android

Install Android Studio, SDK Platform 36, Build Tools 36.0.0, Android NDK r27d (`27.3.13750724`), and the Rust Android targets.

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
cd apps/slopity
npm install
npm run android:init
npm run android:dev
```

The initial supported physical-device target is ARM64. The Android shell can be built now, but reliable background server hosting is not claimed until the foreground-service bridge and on-device runtime evidence are complete.

## Agent workflow

Read `AGENTS.md`, `TASK.md`, and `PROGRESS.md` before changing code. Every implementation milestone requires a documentation-only planning commit before its code commit.
