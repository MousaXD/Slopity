# ADR 0001: Rust core with Tauri shells

**Status:** Accepted  
**Date:** 2026-08-05

## Context

Slopity needs one lightweight codebase for Windows, Linux, and Android while retaining access to Android lifecycle APIs. Reimplementing profile semantics separately in Kotlin and desktop code would create drift.

## Decision

Use a platform-neutral Rust workspace for domain and orchestration contracts. Use Tauri 2 for the shared system-WebView shell. Keep unavoidable Android lifecycle code behind a narrow mobile plugin boundary.

Desktop process execution and Android hosting are separate adapters. Sharing Rust types does not imply that one operating system's process strategy is valid on another.

## Consequences

- Most behavior is portable and unit-testable without a UI.
- The application remains smaller than a bundled-Chromium architecture.
- Kotlin is still required for Android foreground-service integration.
- iOS can reuse the core and UI as a controller, but reliable arbitrary server hosting is not promised.
