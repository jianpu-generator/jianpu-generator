#!/usr/bin/env python3
"""Migrate production's `files` rows from section syntax to anyfix.

    python3 scripts/migrate-anyfix-d1.py [--include <file-id> ...] [--apply]

`tools/migrate-anyfix` converts each file and compares its SVGs and errors under
every preset. A file that differs is held back and listed, and stays as it is
unless named with `--include`. Without `--apply` this is a dry run: it writes
the SQL and changes nothing. `--sqlite <path>` works on a local copy instead.

Each update also writes a `file_versions` row, because a new revision with no
row leaves a gap that hides the file's older versions. It only applies if the
revision is unchanged, so a save made during the migration wins and that file
can be picked up by running this again.
"""

from __future__ import annotations

import argparse
import json
import sqlite3
import subprocess
import sys
from pathlib import Path

ROOT = Path(__file__).resolve().parent.parent
WORKER_DIR = ROOT / "crates" / "live-share-worker"
SELECT_FILES = "SELECT id, name, content, revision FROM files"
# D1 rejects a longer statement.
MAX_STATEMENT_BYTES = 100_000


def run(command: list[str], stdin: str | None = None, cwd: Path = ROOT) -> str:
    result = subprocess.run(command, input=stdin, capture_output=True, text=True, cwd=cwd)
    if result.returncode != 0:
        sys.exit(f"{' '.join(command)} failed:\n{result.stderr}")
    return result.stdout


def wrangler_execute(*arguments: str) -> str:
    return run(
        ["pnpm", "exec", "wrangler", "d1", "execute", "jianpu-live-share", "--remote", *arguments],
        cwd=WORKER_DIR,
    )


def fetch_rows(sqlite_path: str | None) -> list[dict]:
    if sqlite_path is None:
        return json.loads(wrangler_execute("--json", f"--command={SELECT_FILES}"))[0]["results"]
    with sqlite3.connect(sqlite_path) as database:
        database.row_factory = sqlite3.Row
        return [dict(row) for row in database.execute(SELECT_FILES)]


def sql_string(text: str) -> str:
    return "'" + text.replace("'", "''") + "'"


def statements_for(migration: dict) -> list[str]:
    file_id = sql_string(migration["id"])
    revision = migration["revision"]
    unchanged = f"id = {file_id} AND revision = {revision}"
    return [
        "INSERT INTO file_versions (file_id, revision, newer_revision, reverse_patch, saved_at) "
        f"SELECT id, revision, revision + 1, {sql_string(migration['reverse_patch'])}, updated_at "
        f"FROM files WHERE {unchanged};",
        f"UPDATE files SET content = {sql_string(migration['converted'])}, revision = revision + 1 "
        f"WHERE {unchanged};",
    ]


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--apply", action="store_true", help="execute the generated SQL")
    parser.add_argument("--include", action="append", default=[], help="migrate this held file anyway")
    parser.add_argument("--sqlite", help="use this local copy instead of production")
    parser.add_argument("--output", default="target/migrate-anyfix.sql")
    args = parser.parse_args()

    rows = fetch_rows(args.sqlite)
    output = run(
        ["cargo", "run", "--quiet", "-p", "migrate-anyfix"],
        stdin=json.dumps(rows),
    )
    migrations = json.loads(output)

    by_outcome = {
        outcome: [migration for migration in migrations if migration["outcome"] == outcome]
        for outcome in ("skipped", "verified", "held")
    }
    for migration in by_outcome["held"]:
        included = " (included)" if migration["id"] in args.include else ""
        print(f"HELD{included} {migration['name']} [{migration['id']}]")
        for difference in migration["differences"]:
            print(f"    {difference}")
    print({outcome: len(group) for outcome, group in by_outcome.items()})

    selected = by_outcome["verified"] + [
        migration for migration in by_outcome["held"] if migration["id"] in args.include
    ]
    statements = [statement for migration in selected for statement in statements_for(migration)]
    too_long = [s for s in statements if len(s.encode()) > MAX_STATEMENT_BYTES]
    if too_long:
        sys.exit(f"{len(too_long)} statements are over D1's {MAX_STATEMENT_BYTES} byte limit")

    sql_path = ROOT / args.output
    sql_path.write_text("\n".join(statements) + ("\n" if statements else ""))
    print(f"wrote {len(selected)} files to {sql_path}")

    if not args.apply or not statements:
        return
    if args.sqlite is None:
        wrangler_execute("--yes", f"--file={sql_path}")
    else:
        with sqlite3.connect(args.sqlite) as database:
            database.executescript(sql_path.read_text())
    print("applied")


if __name__ == "__main__":
    main()
