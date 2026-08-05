#!/bin/sh
set -eu

BASE_SHA=${1:?base SHA required}
HEAD_SHA=${2:?head SHA required}
CHANGED=$(git diff --name-only "$BASE_SHA" "$HEAD_SHA")

if printf '%s\n' "$CHANGED" | grep -Eq '^(app/|gradle/|build\.gradle\.kts$|settings\.gradle\.kts$|gradle\.properties$|scripts/|\.github/workflows/)'; then
    if ! printf '%s\n' "$CHANGED" | grep -qx 'PROGRESS.md'; then
        echo "Implementation or build files changed without PROGRESS.md." >&2
        exit 1
    fi
fi
