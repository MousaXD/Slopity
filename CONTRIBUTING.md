# Contributing

Follow `AGENTS.md` even when working manually. The repository treats the progress ledger as part of the implementation contract.

## Local setup

```bash
./scripts/install-git-hooks.sh
./gradlew testDebugUnitTest lintDebug assembleDebug
```

## Pull requests

A pull request should include:

- The step number from `PROGRESS.md`.
- Scope and non-goals.
- Test commands and results.
- Android devices used for runtime work.
- Security or architecture changes.
- Screenshots only when UI behavior changed.

Do not attach server software, worlds, APKs, signing material, or private logs to the repository.
