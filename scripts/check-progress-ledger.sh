#!/bin/sh
set -eu

BASE=${1:-HEAD^}
HEAD_REF=${2:-HEAD}
CHANGED=$(git diff --name-only "$BASE" "$HEAD_REF")

if printf '%s\n' "$CHANGED" | grep -Eq '^(crates/|apps/|plugins/|Cargo\.toml$|rust-toolchain\.toml$|\.github/workflows/|scripts/)'; then
    if ! printf '%s\n' "$CHANGED" | grep -qx 'PROGRESS.md'; then
        echo "Implementation/build changes require PROGRESS.md in the same implementation commit." >&2
        exit 1
    fi
fi
