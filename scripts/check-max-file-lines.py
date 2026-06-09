#!/usr/bin/env python3
"""Reject source files whose production line count exceeds MAX_LINES."""

from __future__ import annotations

import re
import sys
from pathlib import Path

MAX_LINES = 600

RUST_TEST_MODULE = re.compile(
    r"#\[cfg\(test\)\]\s*\n\s*mod\s+\w+\s*\{",
    re.MULTILINE,
)


def production_line_count(path: Path) -> int:
    text = path.read_text(encoding="utf-8", errors="replace")
    if path.suffix == ".rs":
        match = RUST_TEST_MODULE.search(text)
        if match:
            text = text[: match.start()]
    return len(text.splitlines())


def main() -> int:
    violations: list[tuple[str, int]] = []

    for arg in sys.argv[1:]:
        path = Path(arg)
        rel = path.as_posix()

        count = production_line_count(path)
        if count > MAX_LINES:
            violations.append((rel, count))

    if not violations:
        return 0

    print(f"Source files must be at most {MAX_LINES} lines (Rust #[cfg(test)] modules excluded):")
    for rel, count in sorted(violations):
        print(f"  {rel}: {count} lines")
    return 1


if __name__ == "__main__":
    raise SystemExit(main())
