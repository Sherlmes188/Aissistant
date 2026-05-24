$ErrorActionPreference = "Stop"

$mingwBin = $env:MINGW_BIN
if ($mingwBin -and (Test-Path $mingwBin) -and (($env:Path -split ";") -notcontains $mingwBin)) {
    $env:Path = "$env:Path;$mingwBin"
}

if (-not (Get-Command windres -ErrorAction SilentlyContinue)) {
    Write-Warning "windres was not found. Set MINGW_BIN to your MinGW bin directory if you want the exe icon embedded."
}

cargo build --release

$exe = Join-Path $PSScriptRoot "..\target\release\aissistant.exe"
if (Test-Path $exe) {
    Write-Host "Built: $exe"
} else {
    throw "Build finished but exe was not found."
}
