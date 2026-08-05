# Contributing

Follow `AGENTS.md`. The progress ledger is part of the implementation contract, not release decoration.

## Setup

```bash
./scripts/install-git-hooks.sh
cargo fmt --all -- --check
cargo test --workspace --all-features
cargo clippy --workspace --all-targets --all-features -- -D warnings
```

For the UI shell:

```bash
cd apps/slopity
npm install
npm run tauri:dev
```

## Pull requests

Include the step number, scope, non-goals, checks and results, platforms tested, and any architecture/security changes. Android runtime claims must include device model, Android version, ABI, and test duration.

Do not attach server software, worlds, runtimes, APKs, signing material, credentials, or private logs.
