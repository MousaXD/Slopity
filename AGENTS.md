# Slopity agent operating contract

These rules apply to every coding agent and human contributor.

## Read-first order

1. `AGENTS.md`
2. `TASK.md`
3. `PROGRESS.md`
4. Relevant files under `docs/`
5. Tests around the area being changed

## Mandatory two-commit protocol

Every meaningful implementation step uses two commits.

### Commit A: declare the step

Before changing application, runtime, build, or workflow code:

1. Add an `IN PROGRESS` entry to `PROGRESS.md`.
2. Record scope, non-goals, risks, intended files, and acceptance checks.
3. Commit only planning and documentation changes.

Use `docs(step-NNN): plan <description>`.

### Commit B: implement and close

1. Implement only the declared scope.
2. Run the declared checks.
3. Update the same entry with files changed, commands run, results, limitations, and follow-up work.
4. Mark it `DONE`, `PARTIAL`, or `BLOCKED` truthfully.
5. Commit implementation, tests, and the completed progress entry together.

Never reconstruct the plan after writing code.

## Branch safety

- Never implement directly on `main`.
- Use `feat/`, `fix/`, `docs/`, `test/`, or `spike/` branches.
- Do not merge, force-push, rewrite shared history, or delete branches without explicit owner instruction.
- Open a draft pull request for each coherent vertical slice.

## Architecture guardrails

- `slopity-core` must remain independent of Tauri, WebViews, Kotlin, and UI code.
- Runtime execution stays behind traits.
- Desktop, Android, and future platforms may implement different adapters without changing profile semantics.
- Never label a runtime supported until it starts, remains alive, accepts commands when applicable, stops cleanly, and survives a real-device or real-platform test.
- A UI button or mocked response is not runtime support.

## Security guardrails

- Never execute profile data through `sh -c`, `bash -c`, PowerShell command strings, or `cmd /C`.
- Use explicit executable paths and structured argument arrays.
- Bind new test services to loopback by default.
- Validate ports, paths, memory budgets, and runtime availability before launch.
- Treat plugins, mods, scripts, JARs, archives, and server packages as executable code.
- Require source, version, license, and checksum metadata before downloads are implemented.
- Never commit runtimes, worlds, server packages, credentials, signing keys, private logs, APKs, or generated Tauri mobile projects.

## Required checks

Record the applicable results in `PROGRESS.md`:

```bash
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

Run platform-specific Tauri builds when the required toolchain is available. Record unavailable checks rather than pretending they passed.
