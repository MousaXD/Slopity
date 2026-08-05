# Self-hosted Linux runner prerequisites

Slopity's GitHub Actions runner is intentionally unprivileged. Install operating-system packages once from an administrator terminal; CI only verifies them.

## Pop!_OS / Ubuntu 24.04

```bash
sudo apt-get update
sudo apt-get install -y \
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

Do not substitute `libappindicator3-dev`. Current Pop!_OS and Ubuntu installations use the Ayatana application-indicator package family, and mixing the legacy package can produce package conflicts.

The workflow fails with a copyable command when a prerequisite is missing. It never invokes `sudo` and never waits for an interactive password.
