$ErrorActionPreference = "Stop"

$mingwBin = "D:\code_tool\x86_64-15.2.0-release-win32-seh-ucrt-rt_v13-rev0\mingw64\bin"
if ((Test-Path $mingwBin) -and (($env:Path -split ";") -notcontains $mingwBin)) {
    $env:Path = "$env:Path;$mingwBin"
}

cargo build --release

$exe = Join-Path $PSScriptRoot "..\target\release\aissistant.exe"
if (Test-Path $exe) {
    Write-Host "Built: $exe"
} else {
    throw "Build finished but exe was not found."
}
