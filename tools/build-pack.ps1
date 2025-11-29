$ErrorActionPreference = 'Stop'
$projRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $projRoot\..

function Find-Cargo {
  try { $null = & cargo --version 2>$null; if ($LASTEXITCODE -eq 0) { return 'cargo' } } catch {}
  $userCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
  if (Test-Path $userCargo) { return $userCargo }
  return $null
}

$distRoot = "dist\win-x64\AndroidChecker"
if (Test-Path $distRoot) { Remove-Item -Recurse -Force $distRoot }
New-Item -Force -ItemType Directory "$distRoot\vendor\platform-tools" | Out-Null
New-Item -Force -ItemType Directory "$distRoot\logs" | Out-Null

$cargoPath = Find-Cargo
if ($cargoPath) {
  Write-Host "Building release..." -ForegroundColor Cyan
  & $cargoPath build --release
  if ($LASTEXITCODE -ne 0) { throw "Build failed" }
  Copy-Item -Force target\release\androidchecker.exe "$distRoot\AndroidChecker.exe"
} else {
  Write-Host "Cargo not found. Please install Rust." -ForegroundColor Red
  exit 1
}

$adbSrc = "vendor\platform-tools"
if (Test-Path $adbSrc) {
    Write-Host "Bundling ADB..." -ForegroundColor Cyan
    Copy-Item -Force "$adbSrc\*" "$distRoot\vendor\platform-tools\"
} else {
    Write-Host "Local bundled ADB not found in vendor/platform-tools. Skipping." -ForegroundColor Yellow
}

$releaseDir = "dist\releases"
New-Item -Force -ItemType Directory $releaseDir | Out-Null
$timestamp = Get-Date -Format "yyyyMMdd_HHmmss"
$zipName = "AndroidChecker_$timestamp.zip"
$zipPath = Join-Path $releaseDir $zipName

Write-Host "Zipping..." -ForegroundColor Cyan
Compress-Archive -Path "$distRoot\*" -DestinationPath $zipPath -Force

Write-Host "Build Complete." -ForegroundColor Green
Write-Host "Output: $distRoot"
Write-Host "Zip: $zipPath"
Pop-Location
