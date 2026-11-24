# Runs every check the CI runs, in the same order. Usage, from the repo root:
#   ./scripts/check.ps1
# cargo-deny is optional locally (cargo install cargo-deny --locked); the CI always runs it.

$ErrorActionPreference = 'Stop'

function Step($name, [scriptblock]$command) {
    Write-Host "==> $name" -ForegroundColor Cyan
    & $command
    if ($LASTEXITCODE -ne 0) { throw "$name failed" }
}

Step 'Format' { cargo fmt --all -- --check }
Step 'Clippy' { cargo clippy --workspace --all-targets -- -D warnings }
Step 'Test' { cargo test --workspace }

if (Get-Command cargo-deny -ErrorAction SilentlyContinue) {
    Step 'Licences and advisories' { cargo deny check }
} else {
    Write-Host '==> Licences and advisories: skipped (cargo-deny not installed)' -ForegroundColor Yellow
}

Write-Host 'All checks passed.' -ForegroundColor Green
