#!/bin/sh
set -eu

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit
echo "Installed repository hooks from .githooks"
