$ErrorActionPreference = "Stop"
$Root = Split-Path -Parent $PSScriptRoot
$env:CARGO_TARGET_DIR = Join-Path $Root "target-guest"

Write-Host "Building guest-core (core wasm)..."
cargo build --release -p guest-core --target wasm32-unknown-unknown
Set-Location $Root

Write-Host "Building guest-component (WIT component)..."
$componentOk = $false
try {
    cargo component build --release -p guest-component
    if ($LASTEXITCODE -eq 0) { $componentOk = $true }
} catch {
    $componentOk = $false
}

if (-not $componentOk) {
    Write-Host "cargo component unavailable; using wasm-tools component new"
    cargo build --release -p guest-component --target wasm32-unknown-unknown
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
    $coreWasm = Join-Path $env:CARGO_TARGET_DIR "wasm32-unknown-unknown/release/guest_component.wasm"
    $out = Join-Path $env:CARGO_TARGET_DIR "wasm32-unknown-unknown/release/guest_component.component.wasm"
    wasm-tools component new $coreWasm -o $out
    if ($LASTEXITCODE -ne 0) { exit $LASTEXITCODE }
}

Write-Host "Done. Artifacts under $env:CARGO_TARGET_DIR"
