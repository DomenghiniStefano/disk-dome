# Step 3's go / no-go on this PC's drive: reads the $MFT, scans it, exports the folder totals and
# compares them with WizTree's. Usage, from the repo root, in a normal (not elevated) PowerShell:
#   ./scripts/proof.ps1                 # drive C:, with the WizTree comparison
#   ./scripts/proof.ps1 -Drive D        # another NTFS drive
#   ./scripts/proof.ps1 -SkipWizTree    # DiskDome only
#   ./scripts/proof.ps1 -Dump           # also save the $MFT to a file, so the parser can be debugged
#                                       # later without administrator rights (it holds every file
#                                       # name of the drive: it stays on this PC)
# It builds the CLI, then asks for administrator rights once (UAC) to read the volume. It only
# reads: nothing on the drive is written, moved or deleted. Results go to
# %TEMP%\diskdome-proof\<date-time>\report.txt.

param(
    [string]$Drive = 'C',
    [switch]$SkipWizTree,
    [switch]$Dump,
    [string]$OutDir,
    [switch]$Elevated
)

$ErrorActionPreference = 'Stop'
$Drive = $Drive.TrimEnd(':', '\').ToUpper()
$repo = Split-Path -Parent $PSScriptRoot
$cli = Join-Path $repo 'target\release\diskdome-cli.exe'

if (-not $Elevated) {
    Write-Host '==> Building diskdome-cli (release)' -ForegroundColor Cyan
    cargo build --release -p diskdome-cli
    if ($LASTEXITCODE -ne 0) { throw 'build failed' }
    $OutDir = Join-Path $env:TEMP ("diskdome-proof\" + (Get-Date -Format 'yyyyMMdd-HHmmss'))
    New-Item -ItemType Directory -Force $OutDir | Out-Null
    $psArgs = @('-NoProfile', '-ExecutionPolicy', 'Bypass', '-File', "`"$PSCommandPath`"",
        '-Drive', $Drive, '-OutDir', "`"$OutDir`"", '-Elevated')
    if ($SkipWizTree) { $psArgs += '-SkipWizTree' }
    if ($Dump) { $psArgs += '-Dump' }
    Write-Host '==> Asking for administrator rights to read the volume' -ForegroundColor Cyan
    Start-Process powershell -Verb RunAs -Wait -ArgumentList $psArgs
    $report = Join-Path $OutDir 'report.txt'
    if (Test-Path $report) {
        Get-Content $report
        Write-Host "`nReport: $report" -ForegroundColor Green
    } else {
        Write-Host 'No report: the elevated run did not start or failed.' -ForegroundColor Red
    }
    return
}

# --- elevated part -------------------------------------------------------------------------------
$report = Join-Path $OutDir 'report.txt'
Start-Transcript -Path $report | Out-Null
try {
    $os = Get-CimInstance Win32_OperatingSystem
    $cpu = (Get-CimInstance Win32_Processor | Select-Object -First 1).Name
    $part = Get-Partition -DriveLetter $Drive
    $disk = Get-PhysicalDisk | Where-Object DeviceId -eq $part.DiskNumber
    Write-Host "PC: $cpu, $([math]::Round($os.TotalVisibleMemorySize / 1MB)) GB RAM; drive $($Drive): $($disk.FriendlyName) ($($disk.BusType))"

    Write-Host "`n==> 1. Reading the `$MFT only (3a), three runs" -ForegroundColor Cyan
    foreach ($i in 1..3) { & $cli mft-read "$($Drive):" }

    Write-Host "`n==> 2. Full scan with folder totals (3b), three runs; the last one exports" -ForegroundColor Cyan
    $ours = Join-Path $OutDir 'diskdome-folders.csv'
    & $cli scan "$($Drive):"
    & $cli scan "$($Drive):"
    & $cli scan "$($Drive):" --export $ours

    if (-not $SkipWizTree) {
        $wiz = @("$env:ProgramFiles\WizTree\WizTree64.exe", "${env:ProgramFiles(x86)}\WizTree\WizTree64.exe") |
            Where-Object { Test-Path $_ } | Select-Object -First 1
        if ($wiz) {
            Write-Host "`n==> 3. WizTree's export of the same drive (folders only)" -ForegroundColor Cyan
            $theirs = Join-Path $OutDir 'wiztree-folders.csv'
            # Its memory is sampled every 100 ms while it runs: the peaks are not readable after it exits.
            $clock = [Diagnostics.Stopwatch]::StartNew()
            $p = Start-Process -PassThru -FilePath $wiz -ArgumentList "`"$($Drive):`"", "/export=`"$theirs`"",
                '/admin=1', '/exportfiles=0', '/exportdrivecapacity=0'
            $peakWs = 0; $peakPrivate = 0
            while (-not $p.HasExited) {
                try {
                    $p.Refresh()
                    $peakWs = [math]::Max($peakWs, $p.PeakWorkingSet64)
                    $peakPrivate = [math]::Max($peakPrivate, $p.PrivateMemorySize64)
                } catch {}
                Start-Sleep -Milliseconds 100
            }
            $clock.Stop()
            Write-Host ("WizTree scan + export: {0:N1} s; memory: peak working set {1:N0} MB, peak private {2:N0} MB (sampled)" -f `
                $clock.Elapsed.TotalSeconds, ($peakWs / 1MB), ($peakPrivate / 1MB))
            Write-Host "`n==> 4. Folder totals, DiskDome vs WizTree" -ForegroundColor Cyan
            & $cli compare $ours $theirs --top 25
        } else {
            Write-Host 'WizTree64.exe not found: comparison skipped.' -ForegroundColor Yellow
        }
    }

    if ($Dump) {
        Write-Host "`n==> 5. Saving the `$MFT for offline debugging" -ForegroundColor Cyan
        & $cli mft-dump "$($Drive):" (Join-Path $OutDir 'mft.ddmft')
    }
} finally {
    Stop-Transcript | Out-Null
}
Write-Host "`nDone. This window can be closed." -ForegroundColor Green
Start-Sleep -Seconds 3
