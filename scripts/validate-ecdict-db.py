"""Validate the compact SQLite database expected by the built-in ECDict service."""

from __future__ import annotations

import sqlite3
import sys
from pathlib import Path

REQUIRED_COLUMNS = {"word", "phonetic", "definition", "translation", "tag", "exchange"}


def validate(path: Path) -> int:
    if not path.is_file():
        raise RuntimeError(f"ECDict database does not exist: {path}")

    connection = sqlite3.connect(path)
    try:
        columns = {row[1] for row in connection.execute("PRAGMA table_info(stardict)")}
        missing = REQUIRED_COLUMNS - columns
        if missing:
            raise RuntimeError(f"missing stardict columns: {', '.join(sorted(missing))}")
        if connection.execute("PRAGMA integrity_check").fetchone()[0] != "ok":
            raise RuntimeError("SQLite integrity check failed")
        count = connection.execute("SELECT COUNT(*) FROM stardict").fetchone()[0]
        if count == 0:
            raise RuntimeError("stardict table is empty")
    finally:
        connection.close()

    print(f"ECDict database: PASS ({count:,} entries)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(validate(Path(sys.argv[1])))
    except (IndexError, OSError, RuntimeError, sqlite3.Error) as error:
        print(f"ECDict database validation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
