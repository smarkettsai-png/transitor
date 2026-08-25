[CmdletBinding()]
param(
    [string]$RuntimePath,
    [string]$OutputPath = "portable-dist\pot-3.0.7-win-x64",
    [string]$ECDictPath,
    [switch]$SkipECDict,
    [string]$TatoebaPath,
    [switch]$SkipTatoeba
)

$ErrorActionPreference = "Stop"
$repo = (Resolve-Path (Join-Path $PSScriptRoot "..")).Path
$out = Join-Path $repo $OutputPath
$target = "x86_64-pc-windows-msvc"
$runtimeName = "Microsoft.WebView2.FixedVersionRuntime.109.0.1518.78.x64"
$runtimeInRepo = Join-Path $repo "src-tauri\$runtimeName"
$windowsConfig = Join-Path $repo "src-tauri\tauri.windows.conf.json"
$fixedRuntimeConfig = Join-Path $repo "src-tauri\webview.x64.json"
$windowsConfigBackup = Join-Path ([System.IO.Path]::GetTempPath()) "pot-tauri-windows-conf-$PID.json"
$runtimeWasCopied = $false
$ecdictSource = $null
$tatoebaSource = $null
$tatoebaPluginId = "plugin.com.pot-app.tatoeba"

function Fail([string]$Message) {
    throw "Portable staging failed: $Message"
}

function Test-X64Executable([string]$Path) {
    $bytes = [System.IO.File]::ReadAllBytes($Path)
    if ($bytes.Length -lt 64 -or $bytes[0] -ne 0x4d -or $bytes[1] -ne 0x5a) {
        return $false
    }
    $peOffset = [System.BitConverter]::ToInt32($bytes, 0x3c)
    if ($peOffset -lt 0 -or $peOffset + 6 -gt $bytes.Length) {
        return $false
    }
    if ($bytes[$peOffset] -ne 0x50 -or $bytes[$peOffset + 1] -ne 0x45 -or
        $bytes[$peOffset + 2] -ne 0 -or $bytes[$peOffset + 3] -ne 0) {
        return $false
    }
    return [System.BitConverter]::ToUInt16($bytes, $peOffset + 4) -eq 0x8664
}

try {
    if (-not $SkipECDict) {
        $defaultECDictPath = Join-Path $repo "portable-assets\ecdict\stardict.db"
        if ([string]::IsNullOrWhiteSpace($ECDictPath)) {
            if (Test-Path $defaultECDictPath -PathType Leaf) {
                $ecdictSource = (Resolve-Path $defaultECDictPath).Path
            } else {
                Fail "No ECDict database found at $defaultECDictPath. Run scripts\prepare-ecdict-db.py first, or pass -SkipECDict explicitly."
            }
        } else {
            $candidate = if ([System.IO.Path]::IsPathRooted($ECDictPath)) {
                $ECDictPath
            } else {
                Join-Path $repo $ECDictPath
            }
            if (-not (Test-Path $candidate -PathType Leaf)) {
                Fail "ECDict database not found: $candidate. Run scripts\prepare-ecdict-db.py first."
            }
            $ecdictSource = (Resolve-Path $candidate).Path
        }
    }

    if (-not $SkipTatoeba) {
        $defaultTatoebaPath = Join-Path $repo "portable-assets\tatoeba\$tatoebaPluginId"
        if ([string]::IsNullOrWhiteSpace($TatoebaPath)) {
            if (-not (Test-Path $defaultTatoebaPath -PathType Container)) {
                if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
                    Fail "Python is required to prepare the Tatoeba plugin. Run scripts\prepare-tatoeba-plugin.py or install Python."
                }
                $preparer = Join-Path $repo "scripts\prepare-tatoeba-plugin.py"
                & python $preparer
                if ($LASTEXITCODE -ne 0) {
                    Fail "Tatoeba plugin preparation failed."
                }
            }
            $tatoebaSource = (Resolve-Path $defaultTatoebaPath).Path
        } else {
            $candidate = if ([System.IO.Path]::IsPathRooted($TatoebaPath)) {
                $TatoebaPath
            } else {
                Join-Path $repo $TatoebaPath
            }
            if (-not (Test-Path $candidate -PathType Container)) {
                Fail "Tatoeba plugin directory not found: $candidate. Run scripts\prepare-tatoeba-plugin.py first."
            }
            $tatoebaSource = (Resolve-Path $candidate).Path
        }
        foreach ($required in @("info.json", "main.js", "tatoeba.svg", "LICENSE", "SOURCE.txt")) {
            if (-not (Test-Path (Join-Path $tatoebaSource $required) -PathType Leaf)) {
                Fail "Tatoeba plugin directory is missing ${required}: $tatoebaSource"
            }
        }
    }

    if ($ecdictSource) {
        if (-not (Get-Command python -ErrorAction SilentlyContinue)) {
            Fail "Python is required to validate the ECDict database. Run scripts\prepare-ecdict-db.py or install Python."
        }
        $validator = Join-Path $repo "scripts\validate-ecdict-db.py"
        & python $validator $ecdictSource
        if ($LASTEXITCODE -ne 0) {
            Fail "ECDict database validation failed: $ecdictSource"
        }
    }

    if (-not (Get-Command corepack -ErrorAction SilentlyContinue)) {
        Fail "corepack is required; install Node.js and enable Corepack first."
    }
    & corepack pnpm --version *> $null
    if ($LASTEXITCODE -ne 0) {
        Fail "Corepack could not invoke pnpm; install Node.js and enable Corepack first."
    }
    if ($RuntimePath) {
        $runtime = (Resolve-Path $RuntimePath -ErrorAction Stop).Path
        if (-not (Test-Path $runtime -PathType Container)) {
            Fail "-RuntimePath must be an extracted WebView2 Fixed Version Runtime directory."
        }
        $runtimeExe = Join-Path $runtime "msedgewebview2.exe"
        if (-not (Test-Path $runtimeExe -PathType Leaf)) {
            Fail "-RuntimePath must contain msedgewebview2.exe at its root; provide the complete x64 fixed runtime directory."
        }
        if (-not (Test-X64Executable $runtimeExe)) {
            Fail "-RuntimePath contains a non-x64 msedgewebview2.exe; provide the x64 fixed runtime directory."
        }
        $runtimeVersion = (Get-Item $runtimeExe).VersionInfo.FileVersion
        if (-not $runtimeVersion.StartsWith("109.0.1518.78")) {
            Fail "-RuntimePath contains WebView2 version $runtimeVersion; expected 109.0.1518.78."
        }
        $requiredRuntimeFiles = @(
            "msedge.dll",
            "icudtl.dat",
            "resources.pak",
            "msedge_100_percent.pak",
            "msedge_200_percent.pak",
            "v8_context_snapshot.bin"
        )
        $missingRuntimeFiles = $requiredRuntimeFiles | Where-Object { -not (Test-Path (Join-Path $runtime $_) -PathType Leaf) }
        if ($missingRuntimeFiles) {
            Fail "-RuntimePath is incomplete; missing WebView2 files: $($missingRuntimeFiles -join ", ")"
        }
        $locales = Join-Path $runtime "Locales"
        if (-not (Test-Path $locales -PathType Container) -or
            -not (Get-ChildItem $locales -Filter "*.pak" -File -ErrorAction SilentlyContinue | Select-Object -First 1)) {
            Fail "-RuntimePath is incomplete; the Locales directory must contain language .pak files."
        }
        if (Test-Path $runtimeInRepo) {
            Fail "$runtimeInRepo already exists; remove it before supplying -RuntimePath so the requested runtime is staged reproducibly."
        }
        Copy-Item $runtime $runtimeInRepo -Recurse
        $runtimeWasCopied = $true
    }

    Push-Location $repo
    if ($RuntimePath) {
        if (-not (Test-Path $windowsConfig -PathType Leaf)) {
            Fail "Could not find the Windows Tauri override at $windowsConfig."
        }
        if (-not (Test-Path $fixedRuntimeConfig -PathType Leaf)) {
            Fail "Could not find the fixed-runtime Tauri config at $fixedRuntimeConfig."
        }
        # Tauri's config merge retains the embedBootstrapper `silent` property
        # from tauri.windows.conf.json, which is invalid for fixedRuntime. Swap
        # the platform override for the repository's fixed-runtime override for
        # the duration of the build, matching the upstream CI workflow.
        Copy-Item $windowsConfig $windowsConfigBackup -Force
        try {
            Copy-Item $fixedRuntimeConfig $windowsConfig -Force
            & corepack pnpm tauri build --bundles none --target $target
            if ($LASTEXITCODE -ne 0) { Fail "Tauri build exited with code $LASTEXITCODE." }
        }
        finally {
            if (Test-Path $windowsConfigBackup -PathType Leaf) {
                Copy-Item $windowsConfigBackup $windowsConfig -Force
                Remove-Item $windowsConfigBackup -Force
            }
        }
    } else {
        Write-Warning "No -RuntimePath supplied. The staged app requires the target machine's installed WebView2 runtime."
        & corepack pnpm tauri build --bundles none --target $target
        if ($LASTEXITCODE -ne 0) { Fail "Tauri build exited with code $LASTEXITCODE." }
    }

    $exe = Join-Path $repo "src-tauri\target\$target\release\pot.exe"
    if (-not (Test-Path $exe)) { $exe = Join-Path $repo "src-tauri\target\release\pot.exe" }
    if (-not (Test-Path $exe)) { Fail "Could not find the release executable under src-tauri\target. Check the build output above." }

    if (Test-Path $out) { Fail "Output directory already exists; choose a new -OutputPath instead of overwriting it." }
    New-Item $out -ItemType Directory -Force | Out-Null
    Copy-Item $exe (Join-Path $out "pot.exe")
    $license = Join-Path $repo "LICENSE"
    if (Test-Path $license -PathType Leaf) {
        Copy-Item $license (Join-Path $out "LICENSE")
    }
    $openccNotice = Join-Path $repo "docs\opencc-source.txt"
    if (Test-Path $openccNotice -PathType Leaf) {
        Copy-Item $openccNotice (Join-Path $out "OPENCC-SOURCE.txt")
    }
    New-Item (Join-Path $out "data\config") -ItemType Directory -Force | Out-Null
    New-Item (Join-Path $out "data\cache") -ItemType Directory -Force | Out-Null
    New-Item (Join-Path $out "portable.marker") -ItemType File -Force | Out-Null

    if ($ecdictSource) {
        $ecdictOut = Join-Path $out "data\ecdict"
        New-Item $ecdictOut -ItemType Directory -Force | Out-Null
        Copy-Item $ecdictSource (Join-Path $ecdictOut "stardict.db")
        $notice = Join-Path $repo "docs\ecdict-source.txt"
        if (Test-Path $notice -PathType Leaf) {
            Copy-Item $notice (Join-Path $ecdictOut "SOURCE.txt")
        }
        Write-Host "Staged local ECDict database: $ecdictSource"
    }

    if ($tatoebaSource) {
        $pluginOut = Join-Path $out "data\config\com.pot-app.desktop\plugins\translate\$tatoebaPluginId"
        New-Item $pluginOut -ItemType Directory -Force | Out-Null
        Copy-Item (Join-Path $tatoebaSource "*") $pluginOut -Recurse -Force
        $package = Join-Path (Split-Path $tatoebaSource -Parent) "$tatoebaPluginId.potext"
        if (Test-Path $package -PathType Leaf) {
            Copy-Item $package (Join-Path $out "$tatoebaPluginId.potext")
        }
        Write-Host "Staged Tatoeba plugin: $tatoebaPluginId"
    }

    if ($RuntimePath) {
        Copy-Item $runtimeInRepo (Join-Path $out $runtimeName) -Recurse
        Write-Host "Staged fixed WebView2 runtime: $runtimeName"
    } else {
        Write-Warning "No fixed runtime was staged; do not claim offline/no-WebView2 support for this output."
    }
    Write-Host "Portable folder staged at: $out"
}
finally {
    Pop-Location -ErrorAction SilentlyContinue
    if ($runtimeWasCopied -and (Test-Path $runtimeInRepo)) {
        Remove-Item $runtimeInRepo -Recurse -Force
    }
}
