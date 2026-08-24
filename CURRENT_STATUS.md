# Current Status

**Date:** 2026-08-24  
**Licensing:** PolyForm Noncommercial License 1.0.0 (`PolyForm-Noncommercial-1.0.0`, source-available)

---

## 1. Operational Overview

Slopity is a pre-production local server control plane with a shared Rust domain core and a Tauri 2 frontend shell.

- **Source Availability:** Released under the PolyForm Noncommercial License 1.0.0. Commercial usage requires a separate agreement (see [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)).
- **Security Stance:** Strict structured process boundaries (never `sh -c` / `cmd /C`), loopback-first binding, typed validation, and explicit capability models.

---

## 2. Integrated Feature Matrix

| Component | Status | Description |
| --- | --- | --- |
| **Shared Rust Core (`slopity-core`)** | Operational | Strict identifier grammar (`[A-Za-z0-9._-]{1,128}`), memory budgeting, port conflict checks. |
| **Authoritative Orchestrator** | Operational | Single lifecycle manager with desired vs observed states, terminal exit observations, bounded deterministic event logs (256 entries). |
| **Profile Store Resilience** | Operational | Schema v1 storage, atomic writes, directory syncing, schema v0→v1 migrations, interrupted-write recovery, automatic backup recovery. |
| **Resource Accounting** | Operational | Host reserve policy, memory planning, deterministic port reservations, CPU headroom warnings. |
| **Host Telemetry** | Operational | Linux `/proc/meminfo` parsing and Android system telemetry (memory, battery, temperature, thermal state, storage). |
| **Built-in HTTP Runtime (`slopity-runtime-http`)** | Operational | Single runnable runtime provider in v0.1.0; loopback & LAN support, bounded logs, request counters. |
| **Desktop Process Boundary (`slopity-runtime-local`)** | Unregistered | Internal foundation adapter for desktop child processes using structured arguments; unexposed by default. |
| **Host Service Plugin (`tauri-plugin-slopity-host`)** | Operational | Android `POST_NOTIFICATIONS` permission handling, foreground service lifecycle, persistent notification control. |
| **Tauri 2 Frontend (`apps/slopity`)** | Operational | Mobile-first responsive UI, drawer navigation, server cards, bottom sheets, profile editor, resource gauges. |

---

## 3. Explicit Runtime Boundaries

Only `BuiltInHttp` is registered and runnable.

- **Unavailable runtimes:** Java/JVM, Minecraft (Paper/Bedrock), Jellyfin, Node.js, Python, PHP, arbitrary native binaries, custom scripts, and remote downloads.
- Profiles configured with other runtime types are flagged as `runtime-not-installed` until verified providers and physical hardware testing are established.

---

## 4. Validation Status

| Validation Suite | Environment | Result |
| --- | --- | --- |
| `cargo fmt --all -- --check` | Linux x86_64 | PASS (0 diffs) |
| `cargo test --workspace --all-features --locked` | Linux x86_64 | PASS (60 tests passed) |
| `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings` | Linux x86_64 | PASS (0 warnings) |
| `cargo metadata --locked` | Linux x86_64 | PASS (5/5 crates inherit PolyForm-Noncommercial-1.0.0) |
| `cargo audit` | Linux x86_64 | PASS (0 vulnerabilities across 434 crates) |
| `npm ci` & `npm audit` | Linux x86_64 | PASS (0 vulnerabilities) |
| `npm run tauri:check` | Linux x86_64 | PASS (Linux binary compiled cleanly) |
| Android Rust Compilation (`aarch64-linux-android`) | Linux x86_64 + NDK 27.3 | PASS (`libslopity_lib.so` compiled cleanly) |
| Gitleaks History Scan | Repository History | PASS (0 leaks found) |
| Privacy & Path Audit | Repository History | PASS (0 maintainer private emails, 0 local machine paths) |

---

## 5. Open Proof Items

1. **Android Physical Device Testing:** Physical on-device proof of persistent notification behavior, background reachability, thermal throttling handling, and OEM battery-killer survival over a continuous multi-hour test.
2. **Windows CI:** Windows compilation and packaging remained deferred until core features and mobile durability mature.
