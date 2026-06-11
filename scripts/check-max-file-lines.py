#!/usr/bin/env python3
"""Reject files whose line count exceeds MAX_LINES."""

from __future__ import annotations

import sys
from pathlib import Path

MAX_LINES = 600


def main() -> int:
    violations: list[tuple[str, int]] = []

    for arg in sys.argv[1:]:
        path = Path(arg)
        count = len(path.read_text(encoding="utf-8", errors="replace").splitlines())
        if count > MAX_LINES:
            violations.append((path.as_posix(), count))

    if not violations:
        return 0

    print(f"Files must be at most {MAX_LINES} lines:")
    for rel, count in sorted(violations):
        print(f"  {rel}: {count} lines")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
