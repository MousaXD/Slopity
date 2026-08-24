# Contributing to Slopity

Thank you for your interest in Slopity!

## Source-Available Licensing and Contribution Terms

Slopity is released as a **source-available** project under the [PolyForm Noncommercial License 1.0.0](LICENSE.md).

- The project is **not** licensed under an OSI-approved open-source license.
- Noncommercial personal, educational, research, and noncommercial organizational use is permitted.
- Commercial use, paid hosting, commercial distribution, and commercial forks require a separate commercial license from the copyright holder (see [COMMERCIAL-LICENSE.md](COMMERCIAL-LICENSE.md)).
- The copyright holder retains all rights to commercially license and distribute Slopity.

### Code Contributions

Because the repository owner maintains commercial dual-licensing rights, direct code contributions from third parties cannot be merged without an explicit contributor agreement assigning or licensing full commercial rights to the maintainer.

We warmly welcome:
- Bug reports and reproduction steps
- Architecture and security feedback
- Documentation clarifications
- Feature requests and use case discussions

## Development Workflow

Contributors and agents working on Slopity must follow the **Two-Commit Protocol** specified in [AGENTS.md](AGENTS.md). The progress ledger in [PROGRESS.md](PROGRESS.md) is part of the implementation contract.

### Prerequisites & Setup

Install Rust (1.82+), Node.js (v22+), and the platform dependencies required by Tauri 2.

```bash
# Install git hooks
./scripts/install-git-hooks.sh

# Run Rust quality checks
cargo fmt --all -- --check
cargo test --workspace --all-features --locked
cargo clippy --workspace --all-targets --all-features --locked -- -D warnings
```

### UI & Tauri Shell

```bash
cd apps/slopity
npm ci
npm run tauri:dev
```

### Linux Build Prerequisites

On Debian/Ubuntu-based systems:
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

## Pull Request Guidelines

- All PRs must target a feature or fix branch, never `main` directly.
- Include the step number, scope, non-goals, checks and results, platforms tested, and any architecture/security changes.
- Android claims must specify physical device model, Android version, ABI, and test duration.
- **Never attach or commit**: server JARs, game worlds, third-party runtimes, APKs/AABs, signing keystores/keys, credentials, or private logs.
