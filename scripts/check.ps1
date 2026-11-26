# Runs every check the CI runs, in the same order. Usage, from the repo root:
#   ./scripts/check.ps1
# cargo-deny and slint-lsp are optional locally (see slint-format.ps1); the CI always runs them.

$ErrorActionPreference = 'Stop'

function Step($name, [scriptblock]$command) {
    Write-Host "==> $name" -ForegroundColor Cyan
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$name failed" }
}

Step 'Format' { cargo fmt --all -- --check }
if (Get-Command slint-lsp -ErrorAction SilentlyContinue) {
    Step 'Slint format' { & (Join-Path $PSScriptRoot 'slint-format.ps1') -Check }
} else {
    Write-Host '==> Slint format: skipped (slint-lsp not installed, see scripts/slint-format.ps1)' -ForegroundColor Yellow
}
Step 'Clippy' { cargo clippy --workspace --all-targets --all-features -- -D warnings }
Step 'Test' { cargo test --workspace --all-features }

if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Step 'Licences and advisories' { cargo deny check }
} else {
    Write-Host '==> Licences and advisories: skipped (cargo-deny not installed)' -ForegroundColor Yellow
}

Write-Host 'All checks passed.' -ForegroundColor Green
