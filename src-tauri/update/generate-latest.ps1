$ErrorActionPreference = "Stop"

# =====================================
# Config
# =====================================

$ScriptDir = Split-Path -Parent $MyInvocation.MyCommand.Path
$Bundle = Join-Path $ScriptDir "..\target\release\bundle\nsis"

$GithubUser = "minhdc-react-native"
$Repo = "vaOne-update"

# =====================================
# Find installer
# =====================================

$exe = Get-ChildItem $Bundle -Filter "*-setup.exe" |
    Sort-Object LastWriteTime -Descending |
    Select-Object -First 1

if (-not $exe) {
    throw "NSIS installer not found."
}

$sigFile = "$($exe.FullName).sig"

if (-not (Test-Path $sigFile)) {
    throw ".sig file not found."
}

# =====================================
# Detect version
# =====================================

$version = [regex]::Match(
    $exe.Name,
    "_([0-9]+\.[0-9]+\.[0-9]+)_"
).Groups[1].Value

if ([string]::IsNullOrWhiteSpace($version)) {
    throw "Cannot detect version from filename."
}

# =====================================
# Read signature
# =====================================

$signature = (Get-Content $sigFile -Raw).Trim()

$url = "https://github.com/$GithubUser/$Repo/releases/download/v$version/$($exe.Name)"

# =====================================
# Generate JSON
# =====================================

$json = @{
    version = $version
    platform = "windows-x86_64"
    data = @{
        signature = $signature
        url = $url
    }
}

$output = Join-Path $Bundle "windows-update.json"

$json |
    ConvertTo-Json -Depth 10 |
    Set-Content -Encoding UTF8 $output

Write-Host ""
Write-Host "===================================="
Write-Host "Windows updater info created"
Write-Host $output
Write-Host "===================================="