"""Extract and compact the official Pot ECDict plugin database for portable mode."""

from __future__ import annotations

import argparse
import shutil
import sqlite3
import sys
import tempfile
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

DEFAULT_PLUGIN_URL = (
    "https://github.com/pot-app/pot-app-translate-plugin-ecdict/"
    "releases/download/2.0.1/plugin.com.pot-app.ecdict.potext"
)
DEFAULT_OUTPUT = Path("portable-assets/ecdict/stardict.db")
REQUIRED_COLUMNS = {"word", "phonetic", "definition", "translation", "tag", "exchange"}


def download(url: str, target: Path) -> None:
    print(f"Downloading ECDict plugin from {url}")
    with urllib.request.urlopen(url, timeout=60) as response, target.open("wb") as output:
        total = int(response.headers.get("Content-Length", "0"))
        copied = 0
        while True:
            chunk = response.read(1024 * 1024)
            if not chunk:
                break
            output.write(chunk)
            copied += len(chunk)
            if total:
                print(f"\r  {copied / total:.0%}", end="", flush=True)
    if total:
        print()


def extract_database(plugin: Path, target: Path) -> None:
    with zipfile.ZipFile(plugin) as archive:
        entries = [entry for entry in archive.infolist() if entry.filename.rstrip("/").endswith("stardict.db")]
        if len(entries) != 1:
            raise RuntimeError("The plugin archive must contain exactly one stardict.db")
        with archive.open(entries[0]) as source, target.open("wb") as output:
            shutil.copyfileobj(source, output, length=1024 * 1024)


def compact(source_path: Path, output_path: Path) -> int:
    source = sqlite3.connect(source_path)
    try:
        columns = {row[1] for row in source.execute("PRAGMA table_info(stardict)")}
        missing = REQUIRED_COLUMNS - columns
        if missing:
            raise RuntimeError(f"ECDict database is missing columns: {', '.join(sorted(missing))}")
        source_count = source.execute("SELECT COUNT(*) FROM stardict").fetchone()[0]
    finally:
        source.close()

    output_path.parent.mkdir(parents=True, exist_ok=True)
    temporary_output = output_path.with_suffix(output_path.suffix + ".tmp")
    if temporary_output.exists():
        temporary_output.unlink()
    destination = sqlite3.connect(temporary_output)
    try:
        destination.executescript(
            """
            PRAGMA journal_mode=OFF;
            PRAGMA synchronous=OFF;
            CREATE TABLE stardict (
                word TEXT PRIMARY KEY COLLATE NOCASE,
                phonetic TEXT,
                definition TEXT,
                translation TEXT,
                tag TEXT,
                exchange TEXT
            );
            """
        )
        destination.execute("ATTACH DATABASE ? AS source", (str(source_path),))
        destination.execute(
            """
            INSERT INTO stardict (word, phonetic, definition, translation, tag, exchange)
            SELECT word, phonetic, definition, translation, tag, exchange
            FROM source.stardict
            """
        )
        destination.commit()
        destination.execute("DETACH DATABASE source")
        destination.commit()
        destination.execute("VACUUM")
        destination.commit()
        result_count = destination.execute("SELECT COUNT(*) FROM stardict").fetchone()[0]
    finally:
        destination.close()

    if result_count != source_count:
        temporary_output.unlink(missing_ok=True)
        raise RuntimeError(f"ECDict row count changed: source={source_count}, output={result_count}")
    temporary_output.replace(output_path)
    return result_count


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument(
        "--plugin",
        default=DEFAULT_PLUGIN_URL,
        help="Path or URL to a Pot .potext ECDict plugin (default: pinned Pot plugin release 2.0.1)",
    )
    parser.add_argument("--output", type=Path, default=DEFAULT_OUTPUT, help="Output SQLite path")
    args = parser.parse_args()

    with tempfile.TemporaryDirectory(prefix="pot-ecdict-") as temporary_directory:
        temporary = Path(temporary_directory)
        plugin_path = temporary / "ecdict.potext"
        plugin = Path(args.plugin)
        if plugin.is_file():
            plugin_path = plugin
        else:
            download(args.plugin, plugin_path)

        source_db = temporary / "stardict.db"
        extract_database(plugin_path, source_db)
        count = compact(source_db, args.output)

    print(f"Prepared {count:,} ECDict entries at {args.output} ({args.output.stat().st_size:,} bytes)")
    return 0


if __name__ == "__main__":
    try:
        raise SystemExit(main())
    except (OSError, RuntimeError, urllib.error.URLError, zipfile.BadZipFile, sqlite3.Error) as error:
        print(f"ECDict preparation failed: {error}", file=sys.stderr)
        raise SystemExit(1)
