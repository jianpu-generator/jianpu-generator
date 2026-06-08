#!/bin/sh
set -e

repo_root="$(cd "$(dirname "$0")/.." && pwd)"
git -C "$repo_root" config core.hooksPath scripts/git-hooks
chmod +x "$repo_root/scripts/git-hooks/pre-commit"
echo "Installed git hooks from scripts/git-hooks/"
