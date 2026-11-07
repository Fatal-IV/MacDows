# A quick local build for trying changes: stops this folder's MacDows (only the one built here, found by its path), builds with the
# "fast" profile (no LTO, many code units, incremental: the first build takes a while, later ones are much shorter), starts it again.
# Releases are NOT made with this; `node scripts/release.mjs` still uses the full release profile.
#
#   npm run fast
$ErrorActionPreference = "Stop"
$root = Split-Path -Parent $PSScriptRoot
Set-Location $root
$target = Join-Path $root "src-tauri\target"

Get-Process macdows -ErrorAction SilentlyContinue | Where-Object { $_.Path -and $_.Path.StartsWith($target, [StringComparison]::OrdinalIgnoreCase) } | ForEach-Object {
    Write-Host "stopping MacDows (PID $($_.Id))"
    Stop-Process -Id $_.Id
}
Start-Sleep -Seconds 1

$watch = [Diagnostics.Stopwatch]::StartNew()
npm run tauri -- build --no-bundle -- --profile fast
if ($LASTEXITCODE -ne 0) { throw "build failed" }
Write-Host ("built in {0:n0} s" -f $watch.Elapsed.TotalSeconds)

$exe = Join-Path $target "fast\macdows.exe"
if (-not (Test-Path $exe)) { throw "not found: $exe" }
Start-Process $exe
Write-Host "started $exe"
