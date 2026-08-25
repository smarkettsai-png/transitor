#!/usr/bin/env python3
"""Fetch and package the pinned Pot Tatoeba plugin with a runtime fix."""

from __future__ import annotations

import argparse
import hashlib
import json
import shutil
import sys
import urllib.error
import urllib.request
import zipfile
from pathlib import Path

PLUGIN_ID = "plugin.com.pot-app.tatoeba"
SOURCE_REPOSITORY = "https://github.com/pot-app/pot-app-translate-plugin-tatoeba"
SOURCE_COMMIT = "0864ca679fd4038d588f6f94afde5a1c33085de4"
RAW_BASE = f"https://raw.githubusercontent.com/pot-app/pot-app-translate-plugin-tatoeba/{SOURCE_COMMIT}"
EXPECTED_SHA256 = {
    "info.json": "963b2908d410062d35ff9bcd5ccfb7fa21d28129d086c2a54b2fba7aac99972f",
    "main.js": "63eb8af1ed2c3797fb6dd0b4bb058bff5b5019b3da7a6b00cbfc209af3137829",
    "tatoeba.svg": "bdfb38216a1d9d0834a8e250bd320360d44583e884b63468aa1e18ae8cb8ac5c",
    "LICENSE": "3972dc9744f6499f0f9b2dbf76696f2ae7ad8af9b23dde66d6af86c9dfb36986",
}

PATCHED_MAIN = r'''const DETECTED_LANGUAGE_MAP = {
    ar: 'ara',
    de: 'deu',
    en: 'eng',
    es: 'spa',
    fr: 'fra',
    hi: 'hin',
    id: 'ind',
    it: 'ita',
    ja: 'jpn',
    km: 'khm',
    ko: 'kor',
    ms: 'zsm',
    mn_cy: 'mon',
    pt_br: 'por',
    pt_pt: 'por',
    ru: 'rus',
    th: 'tha',
    tr: 'tur',
    vi: 'vie',
    yue: 'yue',
    zh_cn: 'cmn',
    zh_tw: 'cmn',
};

async function translate(text, from, to, options) {
    const { utils, detect } = options;
    const { tauriFetch: fetch } = utils;
    const sourceLanguage = from || DETECTED_LANGUAGE_MAP[detect];

    if (!sourceLanguage || !to) {
        return { sentence: [] };
    }

    const res = await fetch('https://api.tatoeba.org/v1/sentences', {
        method: 'GET',
        query: {
            lang: sourceLanguage,
            q: text,
            sort: 'relevance',
            limit: '10',
            'trans:lang': to,
            'showtrans:lang': to,
        },
    });

    if (!res.ok) {
        throw `Http Request Error\nHttp Status: ${res.status}\n${JSON.stringify(res.data)}`;
    }

    const result = res.data;
    const final = { sentence: [], speechText: '', speechLanguage: 'target' };
    const results = Array.isArray(result?.data) ? result.data : [];
    const escapeHtml = (value) =>
        String(value)
            .replaceAll('&', '&amp;')
            .replaceAll('<', '&lt;')
            .replaceAll('>', '&gt;')
            .replaceAll('"', '&quot;')
            .replaceAll("'", '&#39;');

    for (const sentence of results) {
        const translations = Array.isArray(sentence?.translations) ? sentence.translations : [];
        const targetTexts = translations
            .map((translation) => (Array.isArray(translation) ? translation : [translation]))
            .flat()
            .filter((translation) => typeof translation?.text === 'string' && translation.text.trim() !== '')
            .map((translation) => translation.text);
        if (final.speechText === '' && targetTexts.length > 0) {
            final.speechText = targetTexts[0];
        }
        final.sentence.push({
            source: escapeHtml(sentence?.text ?? ''),
            target: targetTexts.map(escapeHtml).join('<br>'),
        });
    }
    if (final.speechText === '' && final.sentence.length > 0) {
        final.speechText = String(results[0]?.text ?? '').trim();
        final.speechLanguage = 'source';
    }
    return final;
}
'''

SOURCE_NOTICE = f"""Pot Tatoeba plugin source notice

Plugin: {PLUGIN_ID}
Repository: {SOURCE_REPOSITORY}
Pinned source commit: {SOURCE_COMMIT} (tag 2.0.1)

This portable build carries a local runtime fix for the upstream plugin's
sentence-result parser. The upstream parser referenced an undefined variable
and attempted to push into a string. The patched parser uses Tatoeba API v1,
handles automatic language detection, reads translations safely, and escapes
sentence text before the Pot UI renders it.

The plugin is GPL-3.0. The Tatoeba example-sentence data is provided by the
Tatoeba service; respect its attribution and Creative Commons licensing terms.
"""


def sha256(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def download(name: str) -> bytes:
    request = urllib.request.Request(
        f"{RAW_BASE}/{name}",
        headers={"User-Agent": "pot-desktop-portable-builder"},
    )
    with urllib.request.urlopen(request, timeout=60) as response:
        return response.read()


def load_sources(source_dir: Path | None) -> dict[str, bytes]:
    sources: dict[str, bytes] = {}
    if source_dir is None:
        for name in EXPECTED_SHA256:
            data = download(name)
            actual = sha256(data)
            if actual != EXPECTED_SHA256[name]:
                raise RuntimeError(
                    f"{name} checksum mismatch for pinned commit: {actual} != {EXPECTED_SHA256[name]}"
                )
            sources[name] = data
    else:
        for name in EXPECTED_SHA256:
            path = source_dir / name
            if not path.is_file():
                raise RuntimeError(f"Tatoeba source directory is missing {path}")
            sources[name] = path.read_bytes()
    return sources


def validate_info(data: bytes) -> dict:
    try:
        info = json.loads(data.decode("utf-8"))
    except (UnicodeDecodeError, json.JSONDecodeError) as error:
        raise RuntimeError(f"invalid Tatoeba info.json: {error}") from error
    if info.get("id") != PLUGIN_ID or info.get("plugin_type") != "translate":
        raise RuntimeError("Tatoeba info.json has an unexpected plugin id or type")
    if info.get("icon") != "tatoeba.svg":
        raise RuntimeError("Tatoeba info.json has an unexpected icon")
    if not isinstance(info.get("language"), dict) or info["language"].get("en") != "eng":
        raise RuntimeError("Tatoeba info.json is missing the English language mapping")
    return info


def write_output(output: Path, sources: dict[str, bytes]) -> None:
    validate_info(sources["info.json"])
    upstream_main = sources["main.js"].decode("utf-8")
    if "o.translations" not in upstream_main or "target.push" not in upstream_main:
        raise RuntimeError("pinned Tatoeba main.js no longer matches the expected source")

    if output.exists():
        shutil.rmtree(output)
    output.mkdir(parents=True)
    (output / "info.json").write_bytes(sources["info.json"])
    (output / "tatoeba.svg").write_bytes(sources["tatoeba.svg"])
    (output / "main.js").write_text(PATCHED_MAIN, encoding="utf-8", newline="\n")
    (output / "LICENSE").write_bytes(sources["LICENSE"])
    (output / "SOURCE.txt").write_text(SOURCE_NOTICE, encoding="utf-8", newline="\n")

    archive = output.parent / f"{PLUGIN_ID}.potext"
    if archive.exists():
        archive.unlink()
    with zipfile.ZipFile(archive, "w", compression=zipfile.ZIP_DEFLATED) as package:
        for name in ("info.json", "tatoeba.svg", "main.js", "LICENSE", "SOURCE.txt"):
            package.write(output / name, name)


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument(
        "--output",
        default="portable-assets/tatoeba/plugin.com.pot-app.tatoeba",
        help="directory for the prepared plugin",
    )
    parser.add_argument(
        "--source-dir",
        help="use a checked-out pinned source directory instead of downloading",
    )
    args = parser.parse_args()
    repo = Path(__file__).resolve().parent.parent
    output = Path(args.output)
    if not output.is_absolute():
        output = repo / output
    source_dir = Path(args.source_dir).resolve() if args.source_dir else None

    try:
        write_output(output, load_sources(source_dir))
    except (OSError, RuntimeError, urllib.error.URLError) as error:
        print(f"Tatoeba plugin preparation failed: {error}", file=sys.stderr)
        return 1
    print(f"Prepared Tatoeba plugin: {output}")
    print(f"Prepared Tatoeba package: {output.parent / (PLUGIN_ID + '.potext')}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
