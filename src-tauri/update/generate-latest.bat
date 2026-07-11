@echo off
setlocal

REM =====================================
REM Config
REM =====================================

set "SCRIPT_DIR=%~dp0"

REM target/release/bundle/updater
set "BUNDLE=%SCRIPT_DIR%..\target\release\bundle\updater"

set "GITHUB_USER=minhdc-react-native"
set "REPO=vaOne-update"


powershell -NoProfile -ExecutionPolicy Bypass ^
"$bundle='%BUNDLE%';" ^
"$user='%GITHUB_USER%';" ^
"$repo='%REPO%';" ^
"" ^
"Write-Host 'Scanning updater artifacts...';" ^
"" ^
"$platforms=@{};" ^
"" ^
"# ==============================" ^
"# Windows NSIS" ^
"# ==============================" ^
"$win=Get-ChildItem $bundle -Filter '*.nsis.zip' | Select-Object -First 1;" ^
"if($win){" ^
"    $sigFile=$win.FullName+'.sig';" ^
"    if(Test-Path $sigFile){" ^
"        $version=[regex]::Match($win.Name,'_([0-9]+\.[0-9]+\.[0-9]+)_x64\.nsis\.zip').Groups[1].Value;" ^
"        $sig=Get-Content $sigFile -Raw;" ^
"        $sig=$sig.Trim();" ^
"        $url='https://github.com/'+$user+'/'+$repo+'/releases/download/v'+$version+'/'+$win.Name;" ^
"        $platforms['windows-x86_64']=@{" ^
"            signature=$sig;" ^
"            url=$url;" ^
"        };" ^
"        Write-Host 'Windows artifact:' $win.Name;" ^
"    }" ^
"}" ^
"" ^
"# ==============================" ^
"# macOS Apple Silicon" ^
"# ==============================" ^
"$macArm=Get-ChildItem $bundle -Filter '*_aarch64.app.tar.gz' | Select-Object -First 1;" ^
"if($macArm){" ^
"    $sigFile=$macArm.FullName+'.sig';" ^
"    if(Test-Path $sigFile){" ^
"        $version=[regex]::Match($macArm.Name,'_([0-9]+\.[0-9]+\.[0-9]+)_aarch64\.app\.tar\.gz').Groups[1].Value;" ^
"        $sig=Get-Content $sigFile -Raw;" ^
"        $sig=$sig.Trim();" ^
"        $url='https://github.com/'+$user+'/'+$repo+'/releases/download/v'+$version+'/'+$macArm.Name;" ^
"        $platforms['darwin-aarch64']=@{" ^
"            signature=$sig;" ^
"            url=$url;" ^
"        };" ^
"        Write-Host 'macOS ARM artifact:' $macArm.Name;" ^
"    }" ^
"}" ^
"" ^
"# ==============================" ^
"# macOS Intel" ^
"# ==============================" ^
"$macIntel=Get-ChildItem $bundle -Filter '*_x86_64.app.tar.gz' | Select-Object -First 1;" ^
"if($macIntel){" ^
"    $sigFile=$macIntel.FullName+'.sig';" ^
"    if(Test-Path $sigFile){" ^
"        $version=[regex]::Match($macIntel.Name,'_([0-9]+\.[0-9]+\.[0-9]+)_x86_64\.app\.tar\.gz').Groups[1].Value;" ^
"        $sig=Get-Content $sigFile -Raw;" ^
"        $sig=$sig.Trim();" ^
"        $url='https://github.com/'+$user+'/'+$repo+'/releases/download/v'+$version+'/'+$macIntel.Name;" ^
"        $platforms['darwin-x86_64']=@{" ^
"            signature=$sig;" ^
"            url=$url;" ^
"        };" ^
"        Write-Host 'macOS Intel artifact:' $macIntel.Name;" ^
"    }" ^
"}" ^
"" ^
"if($platforms.Count -eq 0){throw 'No updater artifacts found'};" ^
"" ^
"$obj=@{" ^
"version=$version;" ^
"notes='Update vaOne plugin';" ^
"pub_date=(Get-Date).ToUniversalTime().ToString('yyyy-MM-ddTHH:mm:ssZ');" ^
"platforms=$platforms;" ^
"};" ^
"" ^
"$output=Join-Path $bundle 'latest.json';" ^
"$obj | ConvertTo-Json -Depth 10 | Out-File $output -Encoding utf8;" ^
"" ^
"Write-Host '';" ^
"Write-Host '================================';" ^
"Write-Host 'latest.json created';" ^
"Write-Host 'Output:' $output;" ^
"Write-Host '================================';"


pause