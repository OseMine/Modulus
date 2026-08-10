# Build script for Modulus on Windows (PowerShell 7+).
#
# Usage:
#   .\scripts\build.ps1            # checks + release build + bundles
#   .\scripts\build.ps1 -SkipChecks
#
# Output: target\bundled\*.vst3 and *.clap

param(
    [switch]$SkipChecks
)

$ErrorActionPreference = "Stop"
$root = Split-Path -Parent (Split-Path -Parent $MyInvocation.MyCommand.Path)

Push-Location $root
try {
    if (-not $SkipChecks) {
        Write-Host "==> fmt" -ForegroundColor Cyan
        cargo fmt --all --check

        Write-Host "==> clippy (-D warnings)" -ForegroundColor Cyan
        cargo clippy --workspace --all-targets -- -D warnings

        Write-Host "==> building demo-module" -ForegroundColor Cyan
        cargo build -p demo-module

        Write-Host "==> tests" -ForegroundColor Cyan
        $env:MODULUS_DEMO_MODULE = Join-Path $root "target\debug\demo_module.dll"
        cargo test --workspace
    }

    Write-Host "==> release build" -ForegroundColor Cyan
    cargo build --release -p modulus-synth -p modulus-fx

    Write-Host "==> bundle VST3/CLAP" -ForegroundColor Cyan
    cargo run -p xtask --release bundle

    Write-Host "Bundles:" -ForegroundColor Green
    Get-ChildItem -Path (Join-Path $root "target\bundled") -Recurse -File |
        Select-Object -ExpandProperty FullName
}
finally {
    Pop-Location
}