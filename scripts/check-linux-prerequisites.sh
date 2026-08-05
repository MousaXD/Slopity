#!/usr/bin/env bash
set -euo pipefail

packages=(
  libwebkit2gtk-4.1-dev
  build-essential
  curl
  wget
  file
  libxdo-dev
  libssl-dev
  libayatana-appindicator3-dev
  librsvg2-dev
  patchelf
)

missing=()
for package in "${packages[@]}"; do
  dpkg -s "$package" >/dev/null 2>&1 || missing+=("$package")
done

if ((${#missing[@]} == 0)); then
  echo "All Linux Tauri prerequisites are installed."
  exit 0
fi

printf 'Missing Linux Tauri prerequisites:'
printf ' %s' "${missing[@]}"
printf '\n\nInstall them once outside GitHub Actions:\n\n'
printf 'sudo apt-get update\nsudo apt-get install -y'
printf ' %q' "${missing[@]}"
printf '\n'
exit 1
