#!/usr/bin/env python3
"""Migrate positional lyric lines to lyric parts in the production D1 database.

Reads every `docs` and `files` row through `wrangler d1 execute --json`, runs
each `content` through `jianpu migrate-lyrics` (stdin -> stdout), and writes
one SQL file of `UPDATE`s for the rows that changed. Nothing is written to the
database unless `--apply` is passed; the default is a dry run that only
writes the SQL file and prints a summary.

    python3 scripts/migrate-lyrics-d1.py [--database <name-or-id>] [--remote] [--apply]

Each updated row's `revision` is bumped so open editors see a conflict instead
of silently overwriting the migrated content. Back the database up first
(`wrangler d1 export`).
"""

from __future__ import annotations

import argparse
import json
import subprocess
import sys
from pathlib import Path

TABLES = {"docs": "share_id", "files": "id"}


WORKER_DIR = Path(__file__).resolve().parent.parent / "crates" / "live-share-worker"


def run(command: list[str], stdin: str | None = None, cwd: Path | None = None) -> str:
    result = subprocess.run(command, input=stdin, capture_output=True, text=True, cwd=cwd)
    if result.returncode != 0:
        sys.exit(f"{' '.join(command)} failed:\n{result.stderr}")
    return result.stdout


def fetch_rows(database: str, table: str, key: str, remote: bool) -> list[dict]:
    location = "--remote" if remote else "--local"
    out = run(
        [
            "pnpm", "exec", "wrangler", "d1", "execute", database, location, "--json",
            "--command", f"SELECT {key} AS id, content FROM {table}",
        ],
        cwd=WORKER_DIR,
    )
    return json.loads(out)[0]["results"]


def sql_string(text: str) -> str:
    return "'" + text.replace("'", "''") + "'"


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__.split("\n")[0])
    parser.add_argument("--database", default=json.loads((WORKER_DIR / "deploy.json").read_text())["d1DatabaseId"])
    parser.add_argument("--remote", action="store_true", help="use the remote database (default: local)")
    parser.add_argument("--apply", action="store_true", help="execute the generated SQL")
    parser.add_argument("--jianpu", default="target/debug/jianpu", help="path to the jianpu binary")
    parser.add_argument("--output", default="migrate-lyrics.sql")
    args = parser.parse_args()

    statements: list[str] = []
    for table, key in TABLES.items():
        rows = fetch_rows(args.database, table, key, args.remote)
        changed = 0
        for row in rows:
            migrated = run([args.jianpu, "migrate-lyrics"], stdin=row["content"])
            if migrated == row["content"]:
                continue
            changed += 1
            statements.append(
                f"UPDATE {table} SET content = {sql_string(migrated)}, "
                f"revision = revision + 1 WHERE {key} = {sql_string(row['id'])};"
            )
        print(f"{table}: {changed} of {len(rows)} rows need migrating")

    Path(args.output).write_text("\n".join(statements) + ("\n" if statements else ""))
    print(f"wrote {len(statements)} statements to {args.output}")

    if args.apply and statements:
        location = "--remote" if args.remote else "--local"
        output = Path(args.output).resolve()
        run(
            ["pnpm", "exec", "wrangler", "d1", "execute", args.database, location, "--file", str(output)],
            cwd=WORKER_DIR,
        )
        print("applied")


if __name__ == "__main__":
    main()
