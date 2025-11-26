$ErrorActionPreference = 'Stop'
$projRoot = Split-Path -Parent $MyInvocation.MyCommand.Path
Push-Location $projRoot\..

function Find-Cargo {
  try { $null = & cargo --version 2>$null; if ($LASTEXITCODE -eq 0) { return 'cargo' } } catch {}
  $userCargo = Join-Path $env:USERPROFILE ".cargo\bin\cargo.exe"
  if (Test-Path $userCargo) { return $userCargo }
  return $null
}

$dist = "dist\win-x64\AdbAudit"
New-Item -Force -ItemType Directory $dist | Out-Null

${cargoPath} = Find-Cargo
if ($cargoPath) {
  Write-Host "Building release..." -ForegroundColor Cyan
  & $cargoPath build --release
  Copy-Item -Force target\release\androidchecker.exe "$dist\AndroidChecker.exe"
} else {
  Write-Host "cargo not found, using existing binaries in dist." -ForegroundColor Yellow
  if (-not (Test-Path "$dist\AndroidChecker.exe")) {
    throw "No cargo and no existing $dist\AndroidChecker.exe to package."
  }
}

function Copy-Adb($targetDir) {
  $candidates = @()
  $sdkLocal = Join-Path $env:LOCALAPPDATA 'Android\Sdk\platform-tools'
  $sdkUser = Join-Path $env:USERPROFILE 'AppData\Local\Android\Sdk\platform-tools'
  if (Test-Path $sdkLocal) { $candidates += $sdkLocal }
  if (Test-Path $sdkUser) { $candidates += $sdkUser }
  try {
    $where = & where adb 2>$null
    if ($LASTEXITCODE -eq 0 -and $where) {
      $wherePath = Split-Path -Parent ($where -split "\r?\n")[0]
      if ($wherePath) { $candidates += $wherePath }
    }
  } catch {}
  $candidates += (Join-Path $PSScriptRoot '..\dist\win-x64\AdbAudit')

  foreach ($dir in $candidates) {
    $adb = Join-Path $dir 'adb.exe'
    if (Test-Path $adb) {
      $resolvedSource = (Resolve-Path $dir).Path
      $resolvedTarget = (Resolve-Path $targetDir).Path
      if ($resolvedSource -eq $resolvedTarget) { continue }
      
      Write-Host "Using adb from: $dir" -ForegroundColor Cyan
      Copy-Item -Force $adb $targetDir
      foreach ($dll in @('AdbWinApi.dll','AdbWinUsbApi.dll')) {
        $dllPath = Join-Path $dir $dll
        if (Test-Path $dllPath) { Copy-Item -Force $dllPath $targetDir }
      }
      return
    }
  }
  Write-Host "adb not found; skipped copying adb." -ForegroundColor Yellow
}

Copy-Adb -targetDir $dist

$releaseDir = "dist\releases"
New-Item -Force -ItemType Directory $releaseDir | Out-Null

$timestamp = Get-Date -Format "yyyy.MM.dd-HHmmss"
$zipName = "AndroidChecker-$timestamp-win-x64.zip"
$zipPath = Join-Path $releaseDir $zipName

Write-Host "`nCreating release package..." -ForegroundColor Cyan
Compress-Archive -Path $dist -DestinationPath $zipPath -Force

Write-Host "`n========================================" -ForegroundColor Cyan
Write-Host "Build Complete!" -ForegroundColor Green
Write-Host "========================================" -ForegroundColor Cyan
Write-Host "`nOutput directory: $dist" -ForegroundColor Yellow
Write-Host "Executable: .\$dist\AndroidChecker.exe" -ForegroundColor White
Write-Host "Release package: $zipPath" -ForegroundColor Magenta
Write-Host "========================================`n" -ForegroundColor Cyan
Pop-Location
