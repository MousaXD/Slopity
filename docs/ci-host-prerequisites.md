# GitHub-hosted Linux CI prerequisites

Slopity's active CI and release workflows use GitHub-hosted runners. No self-hosted runner is required for normal repository validation.

The hosted Linux jobs install Tauri's operating-system packages non-interactively for each clean runner, then execute `scripts/check-linux-prerequisites.sh` so a missing package fails with a clear diagnostic before compilation.

## Local Pop!_OS / Ubuntu setup

Contributors who want to reproduce the Linux Tauri checks locally can install the same packages once from an administrator terminal:

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

Do not substitute `libappindicator3-dev`. Current Ubuntu/Pop!_OS environments use the Ayatana application-indicator package family and mixing the legacy package can create package conflicts.

`check-linux-prerequisites.sh` itself never invokes `sudo`; privilege escalation is confined to the explicit hosted-runner installation step or to a contributor's one-time local setup.
