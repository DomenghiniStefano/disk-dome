# Adds the interface's new @tr strings to the Italian translation, keeping the ones already translated.
# Then translate the new entries (empty msgstr) by hand; crates/ui/tests/translations.rs checks the result.
# Usage, from the repo root:
#   ./scripts/translations.ps1
# Needs the extractor, the same version as Slint: cargo install slint-tr-extractor --version 1.19.1 --locked

$ErrorActionPreference = 'Stop'

$ui = Join-Path $PSScriptRoot '..\crates\ui'
$po = Join-Path $ui 'lang\it\LC_MESSAGES\diskdome-ui.po'

Push-Location $ui
try {
    $sources = Get-ChildItem -Path ui -Recurse -Filter *.slint | ForEach-Object { Resolve-Path -Relative $_.FullName }
    & slint-tr-extractor --join-existing --default-domain diskdome-ui -o $po @sources
    if ($LASTEXITCODE -ne 0) { throw 'slint-tr-extractor failed' }
} finally {
    Pop-Location
}

# Source references (`#: file:line`) would change the file at every edit of a .slint: keep only the strings.
$lines = Get-Content -Path $po -Encoding UTF8 | Where-Object { -not $_.StartsWith('#: ') }
[IO.File]::WriteAllText($po, (($lines -join "`n") + "`n"), [Text.UTF8Encoding]::new($false))
Write-Host "Updated $po. Translate the entries with an empty msgstr." -ForegroundColor Green
