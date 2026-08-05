# Agent operating contract

These rules apply to every coding agent and human contributor working in this repository.

## Read-first order

Before modifying anything, read:

1. `AGENTS.md`
2. `TASK.md`
3. `PROGRESS.md`
4. Relevant files in `docs/`
5. Existing tests around the area being changed

## Mandatory step and commit protocol

Every implementation step uses two commits. Do not squash these while actively developing.

### Commit A: declare the step first

Before changing application or build code:

1. Add a new entry to `PROGRESS.md`.
2. Mark it `IN PROGRESS`.
3. Record scope, non-goals, risks, acceptance checks, and intended files.
4. Commit only documentation and planning changes.

Commit format:

```text
docs(step-NNN): plan <short description>
```

### Commit B: implement and close the step

1. Implement only the declared scope.
2. Run the declared checks.
3. Update the same `PROGRESS.md` entry with files changed, commands run, results, limitations, and follow-up work.
4. Mark it `DONE`, `PARTIAL`, or `BLOCKED` truthfully.
5. Commit implementation, tests, and the completed progress entry together.

Commit format:

```text
feat(step-NNN): <short description>
fix(step-NNN): <short description>
test(step-NNN): <short description>
refactor(step-NNN): <short description>
```

Never write code first and reconstruct the plan afterward. The Markdown declaration must exist in Git history before implementation.

## Branch and merge rules

- Never work directly on `main`.
- Use `feat/`, `fix/`, `docs/`, `test/`, or `spike/` branches.
- Do not merge, force-push, rewrite shared history, or delete branches unless the user explicitly requests it.
- Open a draft pull request when a coherent vertical slice is ready.
- Keep commits narrow and independently understandable.

## Truthfulness requirements

- Never label a runtime as supported until it starts, remains alive, accepts commands when applicable, stops cleanly, and survives an on-device test.
- Never substitute a desktop Linux binary and call it Android-compatible.
- Record unexecuted tests as unexecuted.
- Record device model, Android version, ABI, and test duration for runtime validation.
- A UI mock is not runtime support.

## Security boundaries

- Do not execute untrusted command strings through `sh -c`.
- Prefer structured argument arrays and fixed runtime entry points.
- Keep server data in app-private storage by default.
- Require checksums and provenance metadata for downloaded artifacts.
- Never commit server JARs, runtimes, secrets, signing keys, worlds, user data, or generated APKs.
- Do not expose remote control without authentication, authorization, rate limits, and an explicit threat-model update.
- Bind to loopback by default during early runtime work.
- Treat imported plugins, mods, scripts, and server packages as executable code.

## Android constraints

- Server hosting must remain user-initiated and visibly controlled through the foreground service.
- Do not silently auto-start hosting after boot.
- Do not bypass Android background, notification, storage, or foreground-service rules.
- Keep the foreground-service declaration and Play policy explanation aligned with actual behavior.
- Resource recommendations must reserve memory for Android and the app itself.

## Definition of done

A step is not done until:

- The intended behavior exists.
- Tests or verification commands have run, or the inability to run them is recorded.
- `PROGRESS.md` is updated.
- Documentation reflects changed architecture or security assumptions.
- No generated artifacts or credentials are staged.
