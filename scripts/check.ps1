# Runs every check the CI runs, in the same order. Usage, from the repo root:
#   ./scripts/check.ps1            # cargo's own parallelism
#   ./scripts/check.ps1 -Jobs 2    # fewer builds at once: a full parallel build can run out of the page file
# cargo-deny and slint-lsp are optional locally (see slint-format.ps1); the CI always runs them.

param([int]$Jobs = 0)

$ErrorActionPreference = 'Stop'
$jobArgs = if ($Jobs -gt 0) { @('-j', $Jobs) } else { @() }

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
Step 'Clippy' { cargo clippy --workspace --all-targets --all-features @jobArgs -- -D warnings }
Step 'Test' { cargo test --workspace --all-features @jobArgs }

if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Step 'Licences and advisories' { cargo deny check }
} else {
    Write-Host '==> Licences and advisories: skipped (cargo-deny not installed)' -ForegroundColor Yellow
}

Write-Host 'All checks passed.' -ForegroundColor Green
