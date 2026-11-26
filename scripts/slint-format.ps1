# Formats the interface's .slint files with Slint's own formatter, as cargo fmt does for Rust.
# Usage, from the repo root:
#   ./scripts/slint-format.ps1          # rewrites the files
#   ./scripts/slint-format.ps1 -Check   # changes nothing, fails if a file is not formatted (CI, check.ps1)
# Needs the formatter, the same version as Slint (the formatter alone, no live preview):
#   cargo install slint-lsp --version 1.19.1 --locked --no-default-features

param([switch]$Check)

$ErrorActionPreference = 'Stop'

# The app's interface and the design system (Dome UI); paths in messages are relative to the crates folder.
$ui = (Resolve-Path (Join-Path $PSScriptRoot '..\crates')).Path
$files = Get-ChildItem -Path (Join-Path $ui 'ui\ui'), (Join-Path $ui 'dome-ui\ui') -Recurse -Filter *.slint | ForEach-Object { $_.FullName }

if (-not $Check) {
    & slint-lsp format -i @files
    if ($LASTEXITCODE -ne 0) { throw 'slint-lsp format failed' }
    exit 0
}

# The formatter has no check mode: format copies in a temporary folder and compare them with the originals.
$copy = Join-Path ([IO.Path]::GetTempPath()) ("diskdome-slint-format-" + [Guid]::NewGuid())
try {
    $copies = foreach ($file in $files) {
        $target = Join-Path $copy $file.Substring($ui.Length).TrimStart('\', '/')
        New-Item -ItemType Directory -Force -Path (Split-Path $target) | Out-Null
        Copy-Item -Path $file -Destination $target
        $target
    }
    & slint-lsp format -i @copies
    if ($LASTEXITCODE -ne 0) { throw 'slint-lsp format failed' }
    $unformatted = for ($i = 0; $i -lt $files.Count; $i++) {
        $before = [IO.File]::ReadAllText($files[$i])
        $after = [IO.File]::ReadAllText($copies[$i])
        if ($before -ne $after) { $files[$i].Substring($ui.Length).TrimStart('\', '/') }
    }
} finally {
    Remove-Item -Recurse -Force -Path $copy -ErrorAction SilentlyContinue
}

if ($unformatted) {
    Write-Host 'Not formatted (run ./scripts/slint-format.ps1):' -ForegroundColor Red
    $unformatted | ForEach-Object { Write-Host "  $_" }
    exit 1
}
Write-Host 'Every .slint file is formatted.' -ForegroundColor Green
