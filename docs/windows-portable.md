# Windows x64 folder-portable build

Portable mode is opt-in. Set `POT_PORTABLE=1`, or place an empty `portable.marker` beside `pot.exe`. In portable mode settings, history, plugins, backups, and cache are stored below `data/` beside the executable. The Tauri filesystem and asset scopes explicitly allow `$EXE/data/**`. Installed mode continues to use the normal Windows AppData locations.

From a Windows 11 x64 developer shell:

```powershell
corepack pnpm install
python scripts/prepare-ecdict-db.py
corepack pnpm run portable:windows
```

`prepare-ecdict-db.py` downloads the official Pot ECDict plugin release, extracts its
SQLite database, and creates a compact `portable-assets/ecdict/stardict.db`. The
unpacked database is about 300 MB and is staged at `data/ecdict/stardict.db`; the
large database is intentionally ignored by Git. The staging script also prepares
the pinned Tatoeba translation plugin at
`data/config/com.pot-app.desktop/plugins/translate/plugin.com.pot-app.tatoeba/`.
It includes a local parser fix using Tatoeba API v1; use `-SkipTatoeba` only
when deliberately producing a build without Tatoeba. The plugin is visible under
Settings > Services > Translation > Add external service and is automatically added to
the active service list when it is installed or detected. To use a local ECDict plugin archive instead:

```powershell
python scripts/prepare-ecdict-db.py --plugin C:\path\to\plugin.com.pot-app.ecdict.potext
```

To prepare the pinned Tatoeba plugin separately:

```powershell
python scripts/prepare-tatoeba-plugin.py
```

The staging command validates the SQLite schema before copying it and validates the pinned Tatoeba plugin files before staging them. It fails if either source is missing; use `-SkipECDict` or `-SkipTatoeba` only when deliberately producing a build without that feature. The command builds with `--bundles none` (it does not produce NSIS/MSI) and stages `portable-dist/pot-3.0.7-win-x64`. Without `-RuntimePath`, the folder relies on the target machine's installed WebView2 runtime. It is not an offline/no-WebView2 package.

For a folder that carries the complete Microsoft WebView2 Fixed Version Runtime, supply an already extracted x64 runtime directory:

```powershell
corepack pnpm run portable:windows -RuntimePath C:\path\to\Microsoft.WebView2.FixedVersionRuntime.109.0.1518.78.x64
```

The directory must contain `msedgewebview2.exe` and all accompanying runtime files. The script accepts an existing runtime path because downloading the roughly 180 MB runtime is intentionally not part of the reproducible build command. The fixed-runtime configuration follows the repository's existing `src-tauri/webview.x64.json`.

Portable mode disables automatic updater checks and the autostart setting is unavailable, because replacing a moving executable or registering a removable path is unsafe. Diagnostic logs continue to use the normal OS log directory (and the About page opens that directory); settings, history, plugins, backups, cache, and the local ECDict database remain beside the executable. Portable builds automatically add the bundled ECDict service and Tatoeba plugin to the active translation list; both can still be removed from Settings > Services > Translation. ECDict has no service options. Tatoeba uses API v1, queries the online service, and requires network access. DeepL uses its `zh-Hant` target for Traditional Chinese; Yandex only offers `zh`, so `zh_tw` responses are converted with bundled opencc-js. MyMemory is available as a no-key online fallback, subject to its 500-byte request and daily quota limits. Translation providers, Windows OCR language packs, native plugin dependencies, and network access remain external requirements. This repository remains GPLv3-only; retain `LICENSE`, `OPENCC-SOURCE.txt`, `data/ecdict/SOURCE.txt`, the staged Tatoeba plugin's `SOURCE.txt`/`LICENSE`, and other source-notice obligations when distributing modified builds.
