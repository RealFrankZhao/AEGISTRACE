$RootDir = Split-Path -Parent (Split-Path -Parent $PSScriptRoot)
$Platform = "windows"
$AppVersion = "0.1.0"

Set-Location $RootDir

$server = Start-Process -FilePath "cargo" -ArgumentList @("run","-p","aegis-core-server","--",$Platform,$AppVersion) -NoNewWindow -PassThru
Start-Sleep -Seconds 1

$tmpScreen = Join-Path $env:TEMP "aegis_screen.mp4"
Set-Content -Path $tmpScreen -Value "AEGIS DEMO SCREEN" -NoNewline

& cargo run -p aegis-collector-cli -- focus "windows.explorer" "Explorer" "Desktop"
& cargo run -p aegis-collector-cli -- file $tmpScreen "files/screen.mp4" "screen_recording"
$tmpShot = Join-Path $env:TEMP "aegis_shot.jpg"
Set-Content -Path $tmpShot -Value "AEGIS DEMO SHOT" -NoNewline
& cargo run -p aegis-collector-cli -- shot $tmpShot "files/shots/000001.jpg"
& cargo run -p aegis-collector-cli -- input "10000" "42" "3" "1"
& cargo run -p aegis-collector-cli -- stop "demo"

Wait-Process -Id $server.Id
