param(
    [string]$UdlPath = "crates/crypnotes-ffi/src/crypnotes.udl",
    [string]$OutDir = "android/core/bridge/src/main/kotlin/com/crypnotes/core/bridge"
)

$ErrorActionPreference = "Stop"

$repoRoot = Resolve-Path (Join-Path $PSScriptRoot "..")
$udlAbs = Join-Path $repoRoot $UdlPath
$outAbs = Join-Path $repoRoot $OutDir

if (-not (Test-Path -LiteralPath $udlAbs)) {
    throw "UDL file not found: $udlAbs"
}

$bindgen = Get-Command "uniffi-bindgen" -ErrorAction SilentlyContinue
if (-not $bindgen) {
    throw "uniffi-bindgen is not installed. Install with: cargo install uniffi_bindgen --version 0.29.5"
}

$tempDir = Join-Path ([System.IO.Path]::GetTempPath()) ("crypnotes-uniffi-kotlin-" + [guid]::NewGuid().ToString("N"))
New-Item -ItemType Directory -Path $tempDir -Force | Out-Null

try {
    & $bindgen.Source generate $udlAbs --language kotlin --out-dir $tempDir
    if ($LASTEXITCODE -ne 0) {
        throw "uniffi-bindgen generation failed with exit code $LASTEXITCODE"
    }

    $generated = Get-ChildItem -Path $tempDir -Recurse -File -Filter *.kt
    if (-not $generated) {
        throw "No Kotlin files were generated."
    }

    New-Item -ItemType Directory -Path $outAbs -Force | Out-Null

    # Remove old generated bindings while preserving handwritten package files.
    Get-ChildItem -Path $outAbs -File -Filter "Crypnotes*.kt" -ErrorAction SilentlyContinue | Remove-Item -Force

    foreach ($file in $generated) {
        $content = Get-Content -Raw -LiteralPath $file.FullName
        $content = [regex]::Replace($content, "(?m)^package\s+[A-Za-z0-9_.]+", "package com.crypnotes.core.bridge")
        $target = Join-Path $outAbs $file.Name
        Set-Content -LiteralPath $target -Value $content -NoNewline
    }

    Write-Host "Generated $($generated.Count) Kotlin binding file(s) to $outAbs"
}
finally {
    if (Test-Path -LiteralPath $tempDir) {
        Remove-Item -Recurse -Force -LiteralPath $tempDir
    }
}
