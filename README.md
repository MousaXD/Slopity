# Slopity

Slopity is a Rust-first, lightweight control plane for hosting and managing multiple local server workloads. Built with a shared Rust core and a Tauri 2 shell, it targets Linux and Android, with Windows retained as a portable target.

## License & Source Availability

Slopity is released as a **source-available** project under the **[PolyForm Noncommercial License 1.0.0](LICENSE.md)** (`PolyForm-Noncommercial-1.0.0`).

- **Noncommercial Use:** You are free to inspect, run, modify, and redistribute Slopity for personal, educational, research, and noncommercial purposes.
- **Commercial Use:** Commercial use, commercial redistribution, commercial forks, paid server hosting, resale, or embedding into commercial services requires a separate commercial license from the copyright holder. See **[COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)** for inquiries.
- Slopity is **not** licensed under an OSI open-source license.

## Current State and Roadmap

Slopity is pre-production software under active development.

### What is implemented today

- **Shared Rust Core (`slopity-core`)**: Platform-neutral domain model, strict identifier grammar `[A-Za-z0-9._-]{1,128}`, port collision detection, and conservative host memory budgeting.
- **Durable Profile Persistence**: Schema v1 JSON storage with atomic writes, backup preservation, and error recovery/rollback.
- **Built-in HTTP Server (`slopity-runtime-http`)**: Native Rust HTTP test runtime supporting loopback and LAN binding, request counters, and bounded circular log buffers.
- **Process Boundary (`slopity-runtime-local`)**: Direct process execution using structured argument vectors—never routed through `sh -c` or `cmd /C`.
- **Host Service Boundary (`tauri-plugin-slopity-host`)**: Native capability probing and Android foreground service lifecycle bridge.
- **Tauri 2 Frontend (`apps/slopity`)**: Mobile-first responsive dashboard, server card grid, drawer navigation, add-server bottom sheet, action menu, details modal, and profile editor.
- **Reproducible CI & Workflows**: Automated GitHub Actions pipelines running on GitHub-hosted `ubuntu-latest` with locked dependencies (`--locked`, `npm ci`) and pinned action commit SHAs.

### What is NOT yet bundled or runnable

Slopity does **not** currently bundle Java, Node.js, Python, PHP, Minecraft (Paper/Bedrock), or any third-party server binaries. A server profile can describe those runtimes, but the interface marks them unavailable until verified adapters and runtime packages are implemented and tested on physical hardware.

## Platform Support Matrix

| Platform | Build Status | Runtime Status | Notes |
| -------- | ------------ | -------------- | ----- |
| **Linux (x86_64)** | Compiles & CI Tested | Operational (Built-in HTTP & Process) | Tauri desktop app, `.deb`, and AppImage |
| **Android (ARM64)** | Compiles & CI Tested | In Progress (Foreground Bridge) | Compiles debug APK/AAB; physical-device durability testing pending |
| **Windows (x64)** | Architectural Target | Deferred | CI and packaging paused until core features mature |

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
