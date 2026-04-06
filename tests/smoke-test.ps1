$ErrorActionPreference = "Continue"

$BIN = if ($env:WEOLINE_BIN) { $env:WEOLINE_BIN } else { ".\target\release\weoline.exe" }
$Pass = 0
$Fail = 0

function Test-Case($Name, [scriptblock]$Check) {
    if (& $Check) {
        Write-Host "  PASS  $Name"
        $script:Pass++
    } else {
        Write-Host "  FAIL  $Name"
        $script:Fail++
    }
}

if (-not (Test-Path $BIN)) {
    Write-Host "Building release..."
    cargo build --release --quiet
}

# --- Pipe Mode ---
Write-Host ""
Write-Host "=== Pipe Mode ==="

$json = '{"context_window":{"used_percentage":45,"context_window_size":200000}}'

$out = $json | & $BIN
Test-Case "pipe: context bar"  { $out -match "45%" }
Test-Case "pipe: token counts" { $out -match "90k/200k" }

$out = & { $env:SL_MODE = "compact"; $json | & $BIN; Remove-Item Env:\SL_MODE }
Test-Case "pipe: compact mode" { $out -match "45%" }

$out = & { $env:SL_MODE = "minimal"; $json | & $BIN; Remove-Item Env:\SL_MODE }
Test-Case "pipe: minimal mode" { $out -match "45%" }

$out = & { $env:SL_SHOW_LIMITS = "0"; '{}' | & $BIN; Remove-Item Env:\SL_SHOW_LIMITS }
Test-Case "pipe: empty input" { -not $out }

$out = & { $env:SL_SHOW_LIMITS = "0"; 'not json' | & $BIN; Remove-Item Env:\SL_SHOW_LIMITS }
Test-Case "pipe: invalid json" { -not $out }

# --- Version ---
Write-Host ""
Write-Host "=== Version ==="

$out = & $BIN --version
Test-Case "version: shows name"    { $out -match "weoline" }
Test-Case "version: shows version" { $out -match "0\." }

$out = & $BIN -v
Test-Case "version: short flag" { $out -match "weoline" }

# --- Help ---
Write-Host ""
Write-Host "=== Help ==="

$out = & $BIN --help
Test-Case "help: shows usage" { $out -match "USAGE" }
Test-Case "help: shows query" { $out -match "--query" }
Test-Case "help: shows env"   { $out -match "SL_MODE" }

$out = & $BIN -h
Test-Case "help: short flag" { $out -match "USAGE" }

Test-Case "help: shows 5h timer env"     { $out -match "SL_SHOW_5H_TIMER" }
Test-Case "help: shows weekly timer env"  { $out -match "SL_SHOW_WEEKLY_TIMER" }
Test-Case "help: shows sonnet timer env"  { $out -match "SL_SHOW_SONNET_TIMER" }

# --- Query Mode ---
Write-Host ""
Write-Host "=== Query Mode ==="

$hasCache = $false
try {
    $probe = & $BIN --query -f json 2>$null
    if ($probe -match '{') { $hasCache = $true }
} catch {}

if ($hasCache) {
    $out = (& $BIN --query -f json) -join "`n"
    Test-Case "query: json valid" {
        try { $null = $out | ConvertFrom-Json; $true } catch { $false }
    }
    Test-Case "query: json has meta"      { $out -match '"meta"' }
    Test-Case "query: json has five_hour" { $out -match '"five_hour"' }

    $out = (& $BIN --query -f json -d minimal) -join "`n"
    Test-Case "query: minimal has pct" { $out -match "five_hour_pct" }
    Test-Case "query: minimal no meta" { $out -notmatch '"meta"' }

    $out = (& $BIN --query -f json --filter sonnet) -join "`n"
    Test-Case "query: filter excludes five_hour" { $out -notmatch '"five_hour"' }
    Test-Case "query: filter includes sonnet"    { $out -match "seven_day_sonnet" }

    $out = (& $BIN --query -f json -d minimal --filter five-hour) -join "`n"
    Test-Case "query: minimal+filter has 5h" { $out -match "five_hour_pct" }
    Test-Case "query: minimal+filter no 7d"  { $out -notmatch "seven_day_pct" }

    $out = (& $BIN --query) -join "`n"
    Test-Case "query: toon has 5h"     { $out -match "5h:" }
    Test-Case "query: toon has 7d"     { $out -match "7d:" }
    Test-Case "query: toon has sonnet" { $out -match "sonnet:" }

    $out = (& $BIN --query -d minimal) -join "`n"
    Test-Case "query: toon minimal has pct"  { $out -match "5h:" }
    Test-Case "query: toon minimal no timer" { $out -notmatch "↻" }

    $out = (& $BIN --query -d full) -join "`n"
    Test-Case "query: toon full has timer" { $out -match "↻" }
} else {
    Write-Host "  SKIP  query tests (no cache data available)"
}

# --- Error Cases ---
Write-Host ""
Write-Host "=== Error Cases ==="

$env:SL_CACHE_FILE = "C:\nonexistent-weoline-cache"
& $BIN --query 2>$null
$exitCode = $LASTEXITCODE
Remove-Item Env:\SL_CACHE_FILE
Test-Case "query: missing cache exits 1" { $exitCode -eq 1 }

# --- Results ---
Write-Host ""
Write-Host "=== Results ==="
Write-Host "  $Pass passed, $Fail failed"
exit $(if ($Fail -eq 0) { 0 } else { 1 })
