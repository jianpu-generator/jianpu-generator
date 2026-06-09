#!/bin/sh
set -e

repo_root="$(cd "$(dirname "$0")/.." && pwd)"

if ! command -v prek >/dev/null 2>&1; then
  echo "prek is not installed. Install it with: brew install prek" >&2
  exit 1
fi

git -C "$repo_root" config --unset core.hooksPath 2>/dev/null || true
prek -C "$repo_root" install --overwrite
echo "Installed prek git hooks from .pre-commit-config.yaml"
