#!/bin/sh
set -eu

git config core.hooksPath .githooks
chmod +x .githooks/pre-commit scripts/*.sh
echo "Installed Slopity Git hooks from .githooks"
