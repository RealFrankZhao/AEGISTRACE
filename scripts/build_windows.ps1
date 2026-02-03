$RootDir = Split-Path -Parent $PSScriptRoot
$DistDir = Join-Path $RootDir "dist/windows"

New-Item -ItemType Directory -Force -Path $DistDir | Out-Null

Set-Location $RootDir
cargo build --release -p aegis-core-server -p aegis-collector-cli -p aegis-verifier

Copy-Item -Force "target/release/aegis-core-server.exe" $DistDir
Copy-Item -Force "target/release/aegis-collector-cli.exe" $DistDir
Copy-Item -Force "target/release/aegis-verifier.exe" $DistDir

Write-Host "Artifacts in $DistDir"
