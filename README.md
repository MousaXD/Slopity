# Slopity

Slopity is a Rust-first, lightweight control plane for hosting and managing multiple local server workloads. Built with a shared Rust core and a Tauri 2 shell, it targets Linux and Android, with Windows retained as a portable target.

## License & Source Availability

Slopity is released as a **source-available** project under the **[PolyForm Noncommercial License 1.0.0](LICENSE.md)** (`PolyForm-Noncommercial-1.0.0`).

- **Noncommercial Use:** You are free to inspect, run, modify, and redistribute Slopity for personal, educational, research, and noncommercial purposes.
- **Commercial Use:** Commercial use, commercial redistribution, commercial forks, paid server hosting, resale, or embedding into commercial services requires a separate commercial license from the copyright holder. See **[COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)** for inquiries.
- Slopity is **not** licensed under an OSI open-source license.

## Current State and Roadmap

Slopity is pre-production software under active development.

### What is implemented in this release candidate

- **Shared Rust Core (`slopity-core`)**: Platform-neutral domain model, authoritative `ServerOrchestrator` lifecycle engine, strict identifier grammar `[A-Za-z0-9._-]{1,128}` (`is_valid_profile_id`), aggregate resource accounting with host reserve policies, deterministic port reservations, and memory budgeting.
- **Profile Persistence & Resilience**: Schema v1 JSON storage with atomic writes, parent-directory synchronization, schema migration infrastructure (v0 to v1), interrupted-write recovery, and backup restoration (`slopity-core`).
- **Host Telemetry**: Linux `/proc/meminfo` telemetry and Android system-service telemetry (memory, battery, temperature, thermal state, storage headroom). Unknown measurements remain conservatively unallocated.
- **Built-in HTTP Test Runtime (`slopity-runtime-http`)**: Safe native Rust HTTP test server supporting loopback and LAN binding, request counters, deterministic lifecycle transitions, and bounded circular log buffers.
- **Process Boundary (`slopity-runtime-local`)**: Direct process execution using structured argument vectors—never routed through `sh -c` or `cmd /C`. (Kept as an unexposed internal desktop adapter foundation).
- **Host Service Boundary (`tauri-plugin-slopity-host`)**: Native capability probing, Android POST_NOTIFICATIONS permission handling, foreground service lifecycle status, and persistent notification control boundary.
- **Tauri 2 Frontend (`apps/slopity`)**: Mobile-first responsive dashboard, server card list, drawer navigation, add-server bottom sheet, action menu, details modal, resource status display, and profile editor.
- **Reproducible Dependencies & Workflows**: Committed and locked `Cargo.lock` (434 crates) and `package-lock.json`, with GitHub Actions workflows configured for hosted Linux runners and SHA-pinned actions.

### What is NOT yet bundled or runnable

Slopity does **not** currently bundle Java, Node.js, Python, PHP, Minecraft (Paper/Bedrock), Jellyfin, or any third-party server binaries. A server profile can describe those runtimes, but the interface marks them unavailable until verified adapters and runtime packages are implemented and tested on physical hardware. Only the built-in HTTP server runtime is registered and runnable.

## Platform Support Matrix

| Platform | Build Status | Runtime Status | Notes |
| -------- | ------------ | -------------- | ----- |
| **Linux (x86_64)** | Validated locally & workflow configured | Operational (Built-in HTTP & Process adapter) | Tauri desktop app, `.deb`, and AppImage |
| **Android (ARM64)** | Compiles locally & workflow configured | In Progress (Foreground bridge) | Rust `.so` compiles cleanly; physical-device durability testing pending |
| **Windows (x64)** | Architectural target | Deferred | CI and packaging paused until core features mature |

## Architecture

```text
apps/slopity/web                   Shared lightweight HTML/CSS/JS frontend
apps/slopity/src-tauri             Tauri 2 desktop and mobile shell
crates/slopity-core                Portable domain contracts, persistence, validation
crates/slopity-runtime-http        Safe built-in Rust HTTP server runtime
crates/slopity-runtime-local       Desktop child-process adapter (explicit args only)
plugins/tauri-plugin-slopity-host  Platform host-service capability boundary
```

The domain core (`slopity-core`) maintains zero dependencies on Tauri, WebViews, Kotlin, or OS-specific UI frameworks.

For in-depth documentation, see:
- [Architecture Overview](docs/ARCHITECTURE.md)
- [Security Model & Boundaries](docs/SECURITY_MODEL.md)
- [Release Packaging Guide](docs/releases.md)

## Getting Started

### Prerequisites

- Rust 1.82+ (or current stable)
- Node.js 22+
- Platform build dependencies (see below)

### Building and Testing

```bash
# Verify formatting, unit tests, and clippy
cargo fmt --all -- --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings

# Run the Tauri development shell
cd apps/slopity
npm ci
npm run tauri:dev
```

### Linux Dependencies (Debian / Ubuntu)

```bash
sudo apt-get update
sudo apt-get install -y --no-install-recommends \
  libwebkit2gtk-4.1-dev \
  build-essential \
  curl \
  wget \
  file \
  libxdo-dev \
  libssl-dev \
  libayatana-appindicator3-dev \
  librsvg2-dev \
  patchelf
```

### Android Development (ARM64)

Install Android Studio, Android SDK Platform 36, Build Tools 36.0.0, and Android NDK r27d (`27.3.13750724`):

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi i686-linux-android x86_64-linux-android
cd apps/slopity
npm ci
npm run android:init
npm run android:dev
```

## Security & Vulnerability Reporting

Please review our [Security Policy](SECURITY.md) for details on responsible vulnerability disclosure. Do not report security vulnerabilities via public GitHub issues.

## Contributing

Please review [CONTRIBUTING.md](CONTRIBUTING.md) and [AGENTS.md](AGENTS.md) before submitting contributions.
