@echo off
setlocal

REM =====================================
REM Config
REM =====================================

set "SCRIPT_DIR=%~dp0"

REM target/release/bundle/nsis
set "BUNDLE=%SCRIPT_DIR%..\target\release\bundle\nsis"

set "GITHUB_USER=minhdc-react-native"
set "REPO=vaOne-update"

powershell -NoProfile -ExecutionPolicy Bypass ^
"$bundle='%BUNDLE%';" ^
"$user='%GITHUB_USER%';" ^
"$repo='%REPO%';" ^
"" ^
"$exe=Get-ChildItem $bundle -Filter '*-setup.exe' | Select-Object -First 1;" ^
"if(-not $exe){ throw 'NSIS installer not found.' }" ^
"" ^
"$sigFile=$exe.FullName+'.sig';" ^
"if(-not (Test-Path $sigFile)){ throw '.sig file not found.' }" ^
"" ^
"$version=[regex]::Match($exe.Name,'_([0-9]+\.[0-9]+\.[0-9]+)_').Groups[1].Value;" ^
"if([string]::IsNullOrWhiteSpace($version)){ throw 'Cannot detect version from filename.' }" ^
"" ^
"$sig=(Get-Content $sigFile -Raw).Trim();" ^
"$url='https://github.com/'+$user+'/'+$repo+'/releases/download/v'+$version+'/'+$exe.Name;" ^
"" ^
"$platform=@{" ^
"    signature=$sig;" ^
"    url=$url;" ^
"};" ^
"" ^
"$obj=@{" ^
"    version=$version;" ^
"    platform='windows-x86_64';" ^
"    data=$platform;" ^
"};" ^
"" ^
"$output=Join-Path $bundle 'windows-update.json';" ^
"$obj | ConvertTo-Json -Depth 10 | Out-File $output -Encoding utf8;" ^
"" ^
"Write-Host '';" ^
"Write-Host '====================================';" ^
"Write-Host 'Windows updater info created';" ^
"Write-Host $output;" ^
"Write-Host '====================================';"

pause