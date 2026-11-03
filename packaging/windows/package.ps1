# Builds UMLCraft for Windows and writes to dist\release:
#   UMLCraft-<version>-windows-<arch>.msi            installer (WiX Toolset v5: dotnet tool install --global wix)
#   UMLCraft-<version>-windows-<arch>-portable.zip   no installation needed
# Signing: with WINDOWS_CERTIFICATE (base64 .pfx) and WINDOWS_CERTIFICATE_PASSWORD set, the exe and
# the msi are signed with signtool; otherwise they are left unsigned.
param([ValidateSet('x64', 'arm64')][string]$Arch = 'x64')
$ErrorActionPreference = 'Stop'
Set-Location (Join-Path $PSScriptRoot '..\..')

$version = $env:VERSION
if (-not $version) {
  $version = (Select-String -Path Cargo.toml -Pattern '^version = "(.*)"' | Select-Object -First 1).Matches[0].Groups[1].Value
}
$msiVersion = ($version -split '[-+]')[0]   # MSI versions are numbers only
$target = if ($Arch -eq 'arm64') { 'aarch64-pc-windows-msvc' } else { 'x86_64-pc-windows-msvc' }

rustup target add $target
cargo build --release --target $target
if ($LASTEXITCODE) { throw "cargo build failed" }
$exe = "target\$target\release\umlcraft.exe"
New-Item -ItemType Directory -Force dist\release | Out-Null

function Find-SignTool {
  $kits = Join-Path ${env:ProgramFiles(x86)} 'Windows Kits\10\bin'
  $tool = Get-ChildItem $kits -Recurse -Filter signtool.exe -ErrorAction SilentlyContinue |
    Where-Object { $_.FullName -match '\\x64\\' } | Sort-Object FullName -Descending | Select-Object -First 1
  if (-not $tool) { throw "signtool.exe not found" }
  $tool.FullName
}
$pfx = $null
if ($env:WINDOWS_CERTIFICATE) {
  $pfx = Join-Path $env:RUNNER_TEMP 'umlcraft-signing.pfx'
  [IO.File]::WriteAllBytes($pfx, [Convert]::FromBase64String($env:WINDOWS_CERTIFICATE))
  $signtool = Find-SignTool
}
function Sign-File([string]$path) {
  if (-not $pfx) { Write-Host "no WINDOWS_CERTIFICATE: $path stays unsigned"; return }
  & $signtool sign /f $pfx /p $env:WINDOWS_CERTIFICATE_PASSWORD /fd sha256 /tr http://timestamp.digicert.com /td sha256 /d UMLCraft $path
  if ($LASTEXITCODE) { throw "signing $path failed" }
}

Sign-File $exe

# portable zip
$portable = "dist\portable\UMLCraft"
Remove-Item -Recurse -Force dist\portable -ErrorAction SilentlyContinue
New-Item -ItemType Directory -Force $portable | Out-Null
Copy-Item $exe "$portable\umlcraft.exe"
Copy-Item README.md "$portable\README.md"
$zip = "dist\release\UMLCraft-$version-windows-$Arch-portable.zip"
Compress-Archive -Path $portable -DestinationPath $zip -Force

# msi installer
$msi = "dist\release\UMLCraft-$version-windows-$Arch.msi"
wix build -arch $Arch -d "Version=$msiVersion" -d "ExePath=$exe" -d "IconFile=packaging\icons\umlcraft.ico" -d "ReadmePath=README.md" -o $msi packaging\windows\umlcraft.wxs
if ($LASTEXITCODE) { throw "wix build failed" }
Remove-Item dist\release\*.wixpdb -ErrorAction SilentlyContinue   # WiX debug data, not for users
Sign-File $msi

if ($pfx) { Remove-Item $pfx }
Get-ChildItem dist\release
